use crate::config::get_config;
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
    let state_handler = StateHandler::new();

    let should_stop = Arc::new(AtomicBool::new(false));
    let runtime_lock = Arc::new(Mutex::new(()));
    let runtime_info = Arc::new(Mutex::new(RuntimeInfo::load_or_default(
        &runtime_info_path,
        runtime_lock.clone(),
    )));

    let shutdown_handle = start_shutdown_handler(
        should_stop.clone(),
        runtime_info.clone(),
        runtime_info_path.clone(),
    );

    monitoring_loop(
        state_handler,
        runtime_info.clone(),
        runtime_info_path,
        should_stop,
    )
    .await;

    shutdown_handle.await;
    log::info!("Shutdown handler completed successfully.");

    Ok(())
}

fn get_runtime_info_path() -> PathBuf {
    let config = get_config().lock().unwrap();
    config.config_dir.join("runtime_info.json")
}

async fn monitoring_loop(
    state_handler: StateHandler,
    runtime_info: Arc<Mutex<RuntimeInfo>>,
    runtime_info_path: PathBuf,
    should_stop: Arc<AtomicBool>,
) {
    let mut previous_state: Option<State> = None;

    loop {
        if should_stop.load(Ordering::Relaxed) {
            println!("Exiting monitoring loop due to SIGTERM or SIGINT.");
            break;
        }

        {
            let mut runtime_info_guard = runtime_info.lock().unwrap();
            if previous_state.as_ref() != Some(&runtime_info_guard.state) {
                state_handler.handle(&mut runtime_info_guard);
                runtime_info_guard
                    .write(&runtime_info_path)
                    .expect("Failed to write runtime info");

                previous_state = Some(runtime_info_guard.state.clone());
            }
        }

        time::sleep(Duration::from_secs(5)).await;
    }
}

async fn start_shutdown_handler(
    stop_flag: Arc<AtomicBool>,
    runtime_info: Arc<Mutex<RuntimeInfo>>,
    runtime_info_path: PathBuf,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        handle_signals(stop_flag).await;

        let mut runtime_info_guard = runtime_info.lock().unwrap();
        runtime_info_guard.terminate_all_agents();
        runtime_info_guard
            .write(&runtime_info_path)
            .expect("Failed to write runtime info after termination.");

        log::info!("All processes have been successfully terminated.");
    })
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
