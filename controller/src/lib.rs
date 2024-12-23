use crate::config::get_config;
use crate::managers::mmap_storage::MmapHandler;
use crate::managers::runtime::{
    ProcessManager, RuntimeError, RuntimeInfoStorage, RuntimeManager, State,
};
use crate::state::handler::handle_state;
use std::sync::Arc;
use tokio::sync::Mutex;
#[cfg(unix)]
type ProcessManagerImpl = crate::managers::unix_process_manager::UnixProcessManager;

#[cfg(unix)]
mod unix_imports {
    pub use tokio::signal::unix::{signal, SignalKind};
}
#[cfg(unix)]
use unix_imports::*;

#[cfg(windows)]
use windows_imports::*;

mod config;
pub mod managers;
pub mod state;
#[cfg(unix)]
pub mod unix_utils;
pub mod validator;

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
    let (runtime_manager, mut state_rx) =
        initialize_runtime_manager().expect("Failed to create RuntimeManager");

    let runtime_manager = Arc::new(Mutex::new(runtime_manager));
    let shutdown_handle = tokio::spawn(handle_signals(runtime_manager.clone()));

    tokio::spawn(async move {
        let mut description = "Initial state";
        while {
            let current_state = *state_rx.borrow();
            log::info!("Worker: {}: {:?}", description, current_state);
            {
                let mut _runtime_manager = runtime_manager.lock().await;
                if let Err(e) = handle_state(current_state, &mut _runtime_manager).await {
                    log::error!("Worker: Failed to handle {}: {}", description, e);
                }
            }
            description = "State change";
            state_rx.changed().await.is_ok()
        } {}
    });

    let _ = shutdown_handle.await;
    log::info!("Shutdown handler completed successfully.");

    Ok(())
}

fn initialize_runtime_manager() -> Result<
    (
        RuntimeManager<MmapHandler, ProcessManagerImpl>,
        tokio::sync::watch::Receiver<State>,
    ),
    RuntimeError,
> {
    let handler = MmapHandler::new("nodex_runtime_info")?;
    let uds_path = get_config().lock().unwrap().uds_path.clone();
    RuntimeManager::new_by_controller(handler, ProcessManagerImpl {}, uds_path)
}

#[cfg(unix)]
pub async fn handle_signals<H, P>(runtime_manager: Arc<Mutex<RuntimeManager<H, P>>>)
where
    H: RuntimeInfoStorage + Send + Sync + 'static,
    P: ProcessManager + Send + Sync + 'static,
{
    let ctrl_c = tokio::signal::ctrl_c();
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to bind to SIGTERM");
    let mut sigabrt = signal(SignalKind::user_defined1()).expect("Failed to bind to SIGABRT");
    let mut sigint = signal(SignalKind::quit()).expect("Failed to bind to SIGINT");

    // We have the following as a convention.
    // - Only the controller terminates with SIGTERM.
    // - SIGUSR1 is sent to the Agent by SIGINT etc. The Agent that receives SIGUSR1 sends fd of the Unix domain socket.
    tokio::select! {
        _ = sigint.recv() => {
            if let Err(e) = runtime_manager.lock().await.cleanup_all() {
                log::error!("Failed to handle sigint: {}", e);
            }
        },
        _ = ctrl_c => {
            if let Err(e) = runtime_manager.lock().await.cleanup_all() {
                log::error!("Failed to handle CTRL+C: {}", e);
            }
        },
        _ = sigterm.recv() => {
            log::info!("Received SIGTERM. Gracefully stopping application.");
            if let Err(e) = runtime_manager.lock().await.cleanup() {
                log::error!("Failed to handle sigterm: {}", e);
            }
        },
        _ = sigabrt.recv() => {
            if let Err(e) = runtime_manager.lock().await.cleanup_all() {
                log::error!("Failed to handle SIGABRT: {}", e);
            }
        }
    }
    log::info!("All processes have been successfully terminated.");
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
