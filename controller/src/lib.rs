use crate::config::get_config;
use crate::process::agent::AgentProcessManager;
use crate::runtime::{RuntimeInfo, State};
use crate::state::handler::StateHandler;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{self, Duration};

mod config;
mod process;
mod runtime;
mod state;

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let runtime_info_path = get_runtime_info_path();
    let should_stop = Arc::new(AtomicBool::new(false));
    let runtime_lock = Arc::new(Mutex::new(()));
    let runtime_info = Arc::new(Mutex::new(RuntimeInfo::load_or_default(
        &runtime_info_path,
        runtime_lock.clone(),
    )));

    let uds_path = {
        let config = get_config().lock().unwrap();
        config.uds_path.clone()
    };
    let agent_process_manager = Arc::new(Mutex::new(AgentProcessManager::new(
        &uds_path,
        runtime_info.clone(),
    )?));

    let shutdown_handle = tokio::spawn({
        let should_stop = should_stop.clone();
        // let runtime_info = runtime_info.clone();
        // let runtime_info_path = runtime_info_path.clone();
        // let agent_process_manager = agent_process_manager.clone();

        async move {
            handle_signals(should_stop).await;
            // let mut runtime_info_guard = runtime_info.lock().unwrap();
            // for agent in runtime_info_guard.agent_infos.iter_mut() {
            //     let pid = agent.process_id;
            //     log::info!("Terminating process with PID: {}", pid);
            //     let mut manager = agent_process_manager.lock().unwrap();
            //     manager.terminate_agent(pid);
            // }
            // save_runtime_info(&runtime_info_path, &runtime_info);

            log::info!("All processes have been successfully terminated.");
        }
    });

    monitoring_loop(
        runtime_info_path,
        runtime_info,
        agent_process_manager,
        should_stop,
    )
    .await;

    let _ = shutdown_handle.await;
    log::info!("Shutdown handler completed successfully.");

    Ok(())
}

async fn monitoring_loop(
    runtime_info_path: PathBuf,
    runtime_info: Arc<Mutex<RuntimeInfo>>,
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

        let current_state = get_state(&runtime_info);
        if previous_state.as_ref() != Some(&current_state) {
            state_handler.handle(&current_state, &agent_process_manager);

            save_runtime_info(&runtime_info_path, &runtime_info);

            previous_state = Some(current_state);
        }

        time::sleep(Duration::from_secs(5)).await;
    }
}

fn get_runtime_info_path() -> PathBuf {
    let config = get_config().lock().unwrap();
    config.config_dir.join("runtime_info.json")
}

async fn handle_signals(should_stop: Arc<AtomicBool>) {
    let ctrl_c = tokio::signal::ctrl_c();
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to bind to SIGTERM");

    tokio::select! {
        _ = ctrl_c => {
            log::info!("Received SIGINT");
            should_stop.store(true, Ordering::Relaxed);
        },
        _ = sigterm.recv() => {
            log::info!("Received SIGTERM");
            should_stop.store(true, Ordering::Relaxed);
        },
    }
}

fn get_state(runtime_info: &Arc<Mutex<RuntimeInfo>>) -> State {
    let runtime_info_guard = runtime_info.lock().unwrap();
    runtime_info_guard.state.clone()
}

fn save_runtime_info(runtime_info_path: &PathBuf, runtime_info: &Arc<Mutex<RuntimeInfo>>) {
    let runtime_info_guard = runtime_info.lock().unwrap();
    runtime_info_guard
        .write(runtime_info_path)
        .expect("Failed to write runtime info");
}
