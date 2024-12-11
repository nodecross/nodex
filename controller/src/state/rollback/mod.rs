use crate::managers::{
    agent::{AgentManagerError, AgentManagerTrait},
    resource::{ResourceError, ResourceManagerTrait},
    runtime::{RuntimeError, RuntimeInfoStorage, RuntimeManager},
};
#[cfg(unix)]
pub use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error("agent process failed: {0}")]
    AgentError(#[from] AgentManagerError),
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

pub struct RollbackState<'a, A, R, H>
where
    A: AgentManagerTrait,
    R: ResourceManagerTrait,
    H: RuntimeInfoStorage,
{
    #[allow(dead_code)]
    agent_manager: &'a Arc<tokio::sync::Mutex<A>>,
    resource_manager: &'a R,
    runtime_manager: &'a Arc<tokio::sync::Mutex<RuntimeManager<H>>>,
}

impl<'a, A, R, H> RollbackState<'a, A, R, H>
where
    A: AgentManagerTrait,
    R: ResourceManagerTrait,
    H: RuntimeInfoStorage,
{
    pub fn new(
        agent_manager: &'a Arc<tokio::sync::Mutex<A>>,
        resource_manager: &'a R,
        runtime_manager: &'a Arc<tokio::sync::Mutex<RuntimeManager<H>>>,
    ) -> Self {
        RollbackState {
            agent_manager,
            resource_manager,
            runtime_manager,
        }
    }

    pub async fn execute(&self) -> Result<(), RollbackError> {
        log::info!("Starting rollback");

        let latest_backup = self.resource_manager.get_latest_backup();
        match latest_backup {
            Some(backup_file) => {
                let mut runtime_manager = self.runtime_manager
                    .lock()
                    .await;
                let agent_path = std::env::current_exe().map_err(RollbackError::CurrentExecutablePathError)?;
                log::info!("Found backup: {}", backup_file.display());
                self.resource_manager.rollback(&backup_file)?;
                if let Err(err) = self.resource_manager.remove() {
                    log::error!("Failed to remove files {}", err);
                }
                runtime_manager.run_controller(agent_path)?; // TODO: Care about UDS
                runtime_manager.update_state(crate::managers::runtime::State::Default)?;
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
}
