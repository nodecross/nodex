use crate::managers::{
    resource::{ResourceError, ResourceManagerTrait},
    runtime::{ProcessManager, RuntimeError, RuntimeInfoStorage, RuntimeManager},
};

#[cfg(unix)]
pub use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error("Failed to find backup")]
    BackupNotFound,
    #[error("resource operation failed: {0}")]
    ResourceError(#[from] ResourceError),
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("failed to kill process: {0}")]
    FailedKillOwnProcess(String),
    #[error("Failed to get current executable path: {0}")]
    CurrentExecutablePathError(#[source] std::io::Error),
}

pub async fn execute<'a, R, H, P>(
    resource_manager: &'a R,
    runtime_manager: &'a mut RuntimeManager<H, P>,
) -> Result<(), RollbackError>
where
    R: ResourceManagerTrait,
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    log::info!("Starting rollback");

    let latest_backup = resource_manager.get_latest_backup();
    match latest_backup {
        Some(backup_file) => {
            let agent_path = runtime_manager.get_exec_path()?;
            log::info!("Found backup: {}", backup_file.display());
            resource_manager.rollback(&backup_file)?;
            if let Err(err) = resource_manager.remove() {
                log::error!("Failed to remove files {}", err);
            }
            runtime_manager.update_state_without_send(crate::managers::runtime::State::Init)?;
            runtime_manager.launch_controller(agent_path)?;
            log::info!("Rollback completed");

            log::info!("Restarting controller by SIGINT");
            #[cfg(unix)]
            {
                let current_pid = std::process::id();
                signal::kill(Pid::from_raw(current_pid as i32), Signal::SIGINT)
                    .map_err(|e| RollbackError::FailedKillOwnProcess(e.to_string()))?;
            }

            Ok(())
        }
        None => Err(RollbackError::BackupNotFound),
    }
}
