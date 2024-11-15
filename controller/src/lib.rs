use crate::config::get_config;
use crate::managers::agent::AgentProcessManager;
use crate::managers::runtime::{
    FeatType, FileHandler, ProcessInfo, RuntimeError, RuntimeManager, State,
};
use crate::state::handler::StateHandler;
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

    let mut controller_processes = runtime_manager
        .filter_process_info(FeatType::Controller)
        .expect("failed filter controller process");

    controller_processes.retain(|controller_process| {
        runtime_manager.remove_and_filter_running_process(controller_process)
    });

    if !controller_processes.is_empty() {
        log::error!("Controller already running");
        return Ok(());
    }

    on_controller_started(&runtime_manager).expect("failed controller start");

    let uds_path = get_config().lock().unwrap().uds_path.clone();
    let agent_process_manager = Arc::new(Mutex::new(AgentProcessManager::new(uds_path)?));

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

        let current_state = runtime_manager.get_state().expect("failed get state");
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

pub async fn handle_signals(
    should_stop: Arc<AtomicBool>,
    agent_process_manager: Arc<Mutex<AgentProcessManager>>,
    runtime_manager: Arc<RuntimeManager>,
) {
    let ctrl_c = tokio::signal::ctrl_c();
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to bind to SIGTERM");
    let mut sigabrt = signal(SignalKind::user_defined1()).expect("Failed to bind to SIGABRT");

    tokio::select! {
        _ = ctrl_c => {
            if let Err(e) = handle_cleanup(agent_process_manager, runtime_manager).await {
                log::error!("Failed to handle CTRL+C: {}", e);
            }
            should_stop.store(true, Ordering::Relaxed);
        },
        _ = sigterm.recv() => {
            log::info!("Received SIGTERM. Gracefully stopping application.");
            handle_sigterm(should_stop.clone());
        },
        _ = sigabrt.recv() => {
            if let Err(e) = handle_cleanup(agent_process_manager, runtime_manager).await {
                log::error!("Failed to handle SIGABRT: {}", e);
            }
            should_stop.store(true, Ordering::Relaxed);
        },
    };
}

async fn handle_cleanup(
    agent_process_manager: Arc<Mutex<AgentProcessManager>>,
    runtime_manager: Arc<RuntimeManager>,
) -> Result<(), String> {
    log::info!("Received CTRL+C. Initiating shutdown.");

    let current_pid = std::process::id();
    let process_infos = runtime_manager
        .get_process_infos()
        .map_err(|e| e.to_string())?;

    for process_info in process_infos.iter() {
        if process_info.process_id != current_pid {
            log::info!("Terminating process with PID: {}", process_info.process_id);
            let manager = agent_process_manager.lock().map_err(|e| e.to_string())?;
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

    let manager = agent_process_manager.lock().map_err(|e| e.to_string())?;
    manager.cleanup_uds_file().map_err(|e| e.to_string())?;

    log::info!("cleanup successfully.");
    Ok(())
}

fn handle_sigterm(should_stop: Arc<AtomicBool>) {
    log::info!("Received SIGTERM. Setting stop flag.");
    should_stop.store(true, Ordering::Relaxed);
}
