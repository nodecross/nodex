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
    let should_stop = Arc::new(AtomicBool::new(false));
    let file_handler = FileHandler::new(get_runtime_info_path());
    let runtime_manager = RuntimeManager::new(file_handler);
    on_controller_started(&runtime_manager).unwrap();

    let uds_path = {
        let config = get_config().lock().unwrap();
        config.uds_path.clone()
    };
    let agent_process_manager = Arc::new(Mutex::new(AgentProcessManager::new(&uds_path)?));

    let shutdown_handle = tokio::spawn({
        let should_stop = should_stop.clone();

        async move {
            handle_signals(should_stop).await;
            // If we make it possible to shutdown, won't that conflict with the kill by agent that assumes a reboot by systemd?
            // let mut runtime_info_guard = runtime_info.lock().unwrap();
            // for process_info in runtime_info_guard.process_infos.iter_mut() {
            //     let pid = process_info.process_id;
            //     log::info!("Terminating process with PID: {}", pid);
            //     let mut manager = agent_process_manager.lock().unwrap();
            //     manager.terminate_agent(pid);
            // }
            // save_runtime_info(&runtime_info_path, &runtime_info);

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
    // Maybe instead of looping based on state, it would be better to loop if none of the running agents exist in runtime_info?
    let state_handler = StateHandler::new();
    let mut previous_state: Option<State> = None;

    loop {
        if should_stop.load(Ordering::Relaxed) {
            log::info!("Exiting monitoring loop due to SIGTERM or SIGINT.");
            break;
        }

        let current_state = runtime_manager.get_state().unwrap();
        if previous_state.as_ref() != Some(&current_state) {
            let _ = state_handler.handle(runtime_manager, &agent_process_manager);
            previous_state = Some(current_state);
        }

        time::sleep(Duration::from_secs(5)).await;
    }
}

fn get_runtime_info_path() -> PathBuf {
    let config = get_config().lock().unwrap();
    config.config_dir.join("runtime_info.json")
}

fn on_controller_started(runtime_manager: &RuntimeManager) -> Result<(), RuntimeError> {
    let process_info = ProcessInfo::new(stdProcess::id(), FeatType::Controller);
    runtime_manager.add_process_info(process_info)
}

async fn handle_signals(should_stop: Arc<AtomicBool>) {
    let ctrl_c = tokio::signal::ctrl_c();
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to bind to SIGTERM");

    tokio::select! {
        _ = ctrl_c => log::info!("Received SIGINT"),
        _ = sigterm.recv() => log::info!("Received SIGTERM"),
    };

    should_stop.store(true, Ordering::Relaxed);
}
