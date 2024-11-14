use crate::config::get_config;
use crate::managers::agent::AgentProcessManager;
use crate::managers::runtime::{
    FeatType, FileHandler, ProcessInfo, RuntimeError, RuntimeManager, State,
};
use crate::state::handler::StateHandler;
use nix::sys::signal::{self as nix_signal, Signal};
use nix::unistd::Pid;
use std::path::PathBuf;
use std::process as stdProcess;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{self, Duration};

mod config;
pub mod managers;
pub mod state;
pub mod validator;

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let runtime_manager = initialize_runtime_manager();
    let should_stop = Arc::new(AtomicBool::new(false));

    if !runtime_manager
        .filter_process_info(FeatType::Controller)?
        .is_empty()
    {
        log::error!("Controller already running");
        return Ok(());
    }

    on_controller_started(&runtime_manager)?;

    let uds_path = get_config().lock().unwrap().uds_path.clone();
    let agent_process_manager = Arc::new(Mutex::new(AgentProcessManager::new(&uds_path)?));

    let shutdown_handle = tokio::spawn({
        let should_stop = Arc::clone(&should_stop);
        let agent_process_manager = Arc::clone(&agent_process_manager);
        let runtime_manager = runtime_manager.clone();

        async move {
            handle_signals(should_stop, agent_process_manager, runtime_manager).await;
            log::info!("All processes have been successfully terminated.");
        }
    });

    monitoring_loop(&runtime_manager, agent_process_manager, should_stop).await;

    let _ = shutdown_handle.await;
    log::info!("Shutdown handler completed successfully.");

    Ok(())
}

async fn monitoring_loop(
    runtime_manager: &RuntimeManager,
    agent_process_manager: Arc<Mutex<AgentProcessManager>>,
    should_stop: Arc<AtomicBool>,
) {
    let state_handler = StateHandler::new();
    let mut previous_state: Option<State> = None;

    loop {
        if should_stop.load(Ordering::Relaxed) {
            log::info!("Exiting monitoring loop due to SIGTERM or SIGINT.");
            break;
        }

        let current_state = runtime_manager.get_state().unwrap();
        if previous_state.as_ref() != Some(&current_state) {
            if let Err(e) = state_handler.handle(runtime_manager, &agent_process_manager) {
                log::error!("Failed to handle state change: {}", e);
            }
            previous_state = Some(current_state);
        }

        time::sleep(Duration::from_secs(5)).await;
    }
}

fn initialize_runtime_manager() -> Arc<RuntimeManager> {
    let file_handler = FileHandler::new(get_runtime_info_path());
    Arc::new(RuntimeManager::new(file_handler))
}

fn get_runtime_info_path() -> PathBuf {
    get_config()
        .lock()
        .unwrap()
        .config_dir
        .join("runtime_info.json")
}

fn on_controller_started(runtime_manager: &RuntimeManager) -> Result<(), RuntimeError> {
    let process_info = ProcessInfo::new(stdProcess::id(), FeatType::Controller);
    runtime_manager.add_process_info(process_info)
}

async fn handle_signals(
    should_stop: Arc<AtomicBool>,
    agent_process_manager: Arc<Mutex<AgentProcessManager>>,
    runtime_manager: Arc<RuntimeManager>,
) {
    let ctrl_c = tokio::signal::ctrl_c();
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to bind to SIGTERM");

    tokio::select! {
        _ = ctrl_c => {
            if let Err(e) = handle_ctrl_c(agent_process_manager.clone(), runtime_manager.clone()).await {
                log::error!("Failed to handle CTRL+C: {}", e);
            }
        },
        _ = sigterm.recv() => log::info!("Received SIGTERM"),
    };

    should_stop.store(true, Ordering::Relaxed);
}

async fn handle_ctrl_c(
    agent_process_manager: Arc<Mutex<AgentProcessManager>>,
    runtime_manager: Arc<RuntimeManager>,
) -> Result<(), RuntimeError> {
    let current_pid = std::process::id();
    let process_infos = runtime_manager.get_process_infos()?;

    for process_info in process_infos.iter() {
        if process_info.process_id != current_pid {
            log::info!("Terminating process with PID: {}", process_info.process_id);
            let manager = agent_process_manager.lock().unwrap();
            if let Err(e) = manager.terminate_agent(process_info.process_id) {
                log::error!(
                    "Failed to terminate agent process {}: {}",
                    process_info.process_id,
                    e
                );
            }

            runtime_manager.remove_process_info(process_info.process_id)?;
        }
    }

    runtime_manager.update_state(State::Default)?;

    Ok(())
}
