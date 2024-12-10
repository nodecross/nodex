use crate::config::get_config;
use crate::managers::agent::AgentManagerTrait;
use crate::managers::runtime::{
    FeatType, FileHandler, ProcessInfo, RuntimeError, RuntimeInfoStorage, RuntimeManager, State,
};
use crate::state::handler::StateHandler;
use std::path::PathBuf;
use std::process as stdProcess;
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
    let runtime_manager = initialize_runtime_manager();
    let should_stop = Arc::new(AtomicBool::new(false));

    let process_infos = runtime_manager
        .get_process_infos()
        .expect("Failed to read runtime_info.json");

    let controller_processes = process_infos
        .iter()
        .filter(|process_info| {
            runtime_manager.is_running_or_remove_if_stopped(process_info)
                && process_info.feat_type == FeatType::Controller
        })
        .collect::<Vec<&ProcessInfo>>();

    if !controller_processes.is_empty() {
        log::error!("Controller already running");
        return Ok(());
    }

    on_controller_started(&runtime_manager)
        .expect("Failed to record controller start in runtime manager");

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
        let should_stop = Arc::clone(&should_stop);
        let agent_manager = Arc::clone(&agent_manager);
        let runtime_manager = runtime_manager.clone();

        async move {
            handle_signals(should_stop, agent_manager, runtime_manager).await;
            log::info!("All processes have been successfully terminated.");
        }
    });

    state_monitoring_worker(runtime_manager, agent_manager, should_stop).await;

    let _ = shutdown_handle.await;
    log::info!("Shutdown handler completed successfully.");

    Ok(())
}

async fn state_monitoring_worker<A, H>(
    runtime_manager: Arc<RuntimeManager<H>>,
    agent_manager: Arc<tokio::sync::Mutex<A>>,
    should_stop: Arc<AtomicBool>,
) where
    A: AgentManagerTrait + Send + Sync + 'static,
    H: RuntimeInfoStorage + Send + Sync + 'static,
{
    let mut state_rx = runtime_manager.get_state_receiver();

    tokio::spawn(async move {
        let state_handler = StateHandler::new();

        async fn process_state<A, H>(
            state_handler: &StateHandler,
            runtime_manager: &Arc<RuntimeManager<H>>,
            agent_manager: &Arc<tokio::sync::Mutex<A>>,
            state: State,
            description: &str,
        ) where
            A: AgentManagerTrait + Send + Sync + 'static,
            H: RuntimeInfoStorage + Send + Sync + 'static,
        {
            log::info!("Worker: {}: {:?}", description, state);

            let agent_manager = Arc::clone(agent_manager);
            if let Err(e) = state_handler.handle(runtime_manager, &agent_manager).await {
                log::error!("Worker: Failed to handle {}: {}", description, e);
            }
        }

        let initial_state = *state_rx.borrow();
        process_state(
            &state_handler,
            &runtime_manager,
            &agent_manager,
            initial_state,
            "Initial state",
        )
        .await;

        while !should_stop.load(Ordering::Relaxed) {
            if let Ok(()) = state_rx.changed().await {
                let current_state = *state_rx.borrow();
                process_state(
                    &state_handler,
                    &runtime_manager,
                    &agent_manager,
                    current_state,
                    "State change",
                )
                .await;
            }
        }
    });
}

fn initialize_runtime_manager() -> Arc<RuntimeManager<FileHandler>> {
    let file_handler = FileHandler::new(get_runtime_info_path());
    Arc::new(RuntimeManager::new(file_handler))
}

fn get_runtime_info_path() -> PathBuf {
    get_config()
        .lock()
        .unwrap()
        .runtime_dir
        .join("runtime_info.json")
}

fn on_controller_started<H: RuntimeInfoStorage>(
    runtime_manager: &RuntimeManager<H>,
) -> Result<(), RuntimeError> {
    let process_info = ProcessInfo::new(stdProcess::id(), FeatType::Controller);
    runtime_manager.add_process_info(process_info)
}

#[cfg(unix)]
pub async fn handle_signals<A, H>(
    should_stop: Arc<AtomicBool>,
    agent_manager: Arc<Mutex<A>>,
    runtime_manager: Arc<RuntimeManager<H>>,
) where
    A: AgentManagerTrait + Sync + Send + 'static,
    H: RuntimeInfoStorage + Send + Sync + 'static,
{
    let ctrl_c = tokio::signal::ctrl_c();
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to bind to SIGTERM");
    let mut sigabrt = signal(SignalKind::user_defined1()).expect("Failed to bind to SIGABRT");

    tokio::select! {
        _ = ctrl_c => {
            if let Err(e) = handle_cleanup(&agent_manager, &runtime_manager).await {
                log::error!("Failed to handle CTRL+C: {}", e);
            }
            should_stop.store(true, Ordering::Relaxed);
        },
        _ = sigterm.recv() => {
            log::info!("Received SIGTERM. Gracefully stopping application.");
            handle_sigterm(should_stop.clone());
        },
        _ = sigabrt.recv() => {
            if let Err(e) = handle_cleanup(&agent_manager, &runtime_manager).await {
                log::error!("Failed to handle SIGABRT: {}", e);
            }
            should_stop.store(true, Ordering::Relaxed);
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
    runtime_manager: &Arc<RuntimeManager<H>>,
) -> Result<(), String>
where
    A: AgentManagerTrait + Sync + Send,
    H: RuntimeInfoStorage + Send + Sync + 'static,
{
    log::info!("Received CTRL+C. Initiating shutdown.");

    let current_pid = std::process::id();
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

    runtime_manager
        .update_state(crate::managers::runtime::State::Default)
        .map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        let manager = agent_manager.lock().await;
        manager.cleanup().map_err(|e| e.to_string())?;
    }

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
