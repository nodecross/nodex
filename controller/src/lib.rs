use crate::config::get_config;
use crate::managers::agent::AgentManagerTrait;
use crate::managers::mmap_storage::MmapHandler;
use crate::managers::runtime::{
    FeatType, ProcessInfo, RuntimeError, RuntimeInfoStorage, RuntimeManager, State,
};
use crate::state::handler::StateHandler;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(unix)]
mod unix_imports {
    pub use crate::managers::agent::UnixAgentManager;
    pub use tokio::signal::unix::{signal, SignalKind};
}
#[cfg(unix)]
use unix_imports::*;

#[cfg(windows)]
mod windows_imports {
    pub use crate::managers::agent::WindowsAgentManager;
}

#[cfg(windows)]
use windows_imports::*;

mod config;
pub mod managers;
pub mod state;
pub mod validator;

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
    let runtime_manager = initialize_runtime_manager().expect("Failed to create RuntimeManager");
    let should_stop = Arc::new(AtomicBool::new(false));

    {
        let mut _runtime_manager = runtime_manager.lock().await;
        let process_infos = _runtime_manager
            .get_process_infos()
            .expect("Failed to read runtime_info");

        let controller_processes = process_infos
            .iter()
            .filter(|process_info| {
                _runtime_manager.is_running_or_remove_if_stopped(process_info)
                    && process_info.feat_type == FeatType::Controller
            })
            .collect::<Vec<&ProcessInfo>>();
        if !controller_processes.is_empty() {
            log::error!("Controller already running");
            return Ok(());
        }
        on_controller_started(&mut _runtime_manager)
            .expect("Failed to record controller start in runtime manager");
    }

    let uds_path = get_config().lock().unwrap().uds_path.clone();

    #[cfg(unix)]
    let agent_manager = Arc::new(Mutex::new(
        UnixAgentManager::new(uds_path).expect("Failed to create AgentManager"),
    ));

    #[cfg(windows)]
    let agent_manager = Arc::new(Mutex::new(
        WindowsAgentManager::new().expect("Failed to create AgentManager"),
    ));

    let shutdown_handle = tokio::spawn({
        let agent_manager = agent_manager.clone();
        let runtime_manager = runtime_manager.clone();

        async move {
            handle_signals(runtime_manager, agent_manager).await;
            log::info!("All processes have been successfully terminated.");
        }
    });

    state_monitoring_worker(runtime_manager, agent_manager).await;

    let _ = shutdown_handle.await;
    log::info!("Shutdown handler completed successfully.");

    Ok(())
}

async fn state_monitoring_worker<A, H>(
    runtime_manager: Arc<Mutex<RuntimeManager<H>>>,
    agent_manager: Arc<tokio::sync::Mutex<A>>,
) where
    A: AgentManagerTrait + Send + Sync + 'static,
    H: RuntimeInfoStorage + Send + Sync + 'static,
{
    let mut state_rx = runtime_manager.lock().await.get_state_receiver();

    tokio::spawn(async move {
        let state_handler = StateHandler::new();
        let mut description = "Initial state";

        while {
            let current_state = *state_rx.borrow();
            log::info!("Worker: {}: {:?}", description, current_state);

            if let Err(e) = state_handler.handle(current_state, &runtime_manager, &agent_manager).await {
                log::error!("Worker: Failed to handle {}: {}", description, e);
            }
            description = "State change";
            state_rx.changed().await.is_ok()
        } {}
    });
}

fn initialize_runtime_manager() -> Result<Arc<Mutex<RuntimeManager<MmapHandler>>>, RuntimeError> {
    let handler = MmapHandler::new("runtime_info", core::num::NonZero::new(10000).unwrap())?;
    std::env::set_var("MMAP_SIZE", 10000.to_string());
    Ok(Arc::new(Mutex::new(RuntimeManager::new(handler))))
}

fn get_runtime_info_path() -> PathBuf {
    get_config()
        .lock()
        .unwrap()
        .runtime_dir
        .join("runtime_info.json")
}

fn on_controller_started<H: RuntimeInfoStorage>(
    runtime_manager: &mut RuntimeManager<H>,
) -> Result<(), RuntimeError> {
    let process_info = ProcessInfo::new(std::process::id(), FeatType::Controller);
    runtime_manager.add_process_info(process_info)
}

#[cfg(unix)]
pub async fn handle_signals<A, H>(
    runtime_manager: Arc<Mutex<RuntimeManager<H>>>,
    agent_manager: Arc<Mutex<A>>,
) where
    A: AgentManagerTrait + Sync + Send + 'static,
    H: RuntimeInfoStorage + Send + Sync + 'static,
{
    let ctrl_c = tokio::signal::ctrl_c();
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to bind to SIGTERM");
    let mut sigabrt = signal(SignalKind::user_defined1()).expect("Failed to bind to SIGABRT");
    let mut sigint = signal(SignalKind::quit()).expect("Failed to bind to SIGINT");

    tokio::select! {
        _ = sigint.recv() => {
            if let Err(e) = handle_cleanup(&agent_manager, &runtime_manager).await {
                log::error!("Failed to handle CTRL+C: {}", e);
            }
        },
        _ = ctrl_c => {
            if let Err(e) = handle_cleanup(&agent_manager, &runtime_manager).await {
                log::error!("Failed to handle CTRL+C: {}", e);
            }
        },
        _ = sigterm.recv() => {
            log::info!("Received SIGTERM. Gracefully stopping application.");
        },
        _ = sigabrt.recv() => {
            if let Err(e) = handle_cleanup(&agent_manager, &runtime_manager).await {
                log::error!("Failed to handle SIGABRT: {}", e);
            }
        }
    };
}

#[cfg(windows)]
pub async fn handle_signals<A>(
    should_stop: Arc<AtomicBool>,
    agent_manager: Arc<Mutex<A>>,
    runtime_manager: Arc<RuntimeManager>,
) where
    A: AgentManagerTrait + Sync + Send,
{
    unimplemented!("implemented for Windows.");
}

#[cfg(unix)]
async fn handle_cleanup<A, H>(
    agent_manager: &Arc<Mutex<A>>,
    runtime_manager: &Arc<Mutex<RuntimeManager<H>>>,
) -> Result<(), String>
where
    A: AgentManagerTrait + Sync + Send,
    H: RuntimeInfoStorage + Send + Sync + 'static,
{
    log::info!("Received CTRL+C. Initiating shutdown.");

    let current_pid = std::process::id();

    let mut runtime_manager = runtime_manager.lock().await;
    let process_infos = runtime_manager
        .get_process_infos()
        .map_err(|e| e.to_string())?;

    for process_info in process_infos.iter() {
        if process_info.process_id != current_pid {
            log::info!("Terminating process with PID: {}", process_info.process_id);
            let manager = agent_manager.lock().await;
            manager
                .terminate_agent(process_info.process_id)
                .map_err(|e| e.to_string())?;
        }
        runtime_manager
            .remove_process_info(process_info.process_id)
            .map_err(|e| e.to_string())?;
    }

    let manager = agent_manager.lock().await;
    manager.cleanup().map_err(|e| e.to_string())?;

    log::info!("cleanup successfully.");
    Ok(())
}

#[cfg(unix)]
fn handle_sigterm(should_stop: Arc<AtomicBool>) {
    // log::info!("Dropping listener.");
    // std::mem::drop(listener);
    log::info!("Received SIGTERM. Setting stop flag.");
    should_stop.store(true, Ordering::Relaxed);
}
