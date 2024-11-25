use crate::managers::{
    agent::{AgentManager, AgentManagerError},
    resource::{ResourceError, ResourceManager},
    runtime::{RuntimeError, RuntimeManager},
};
pub use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use std::sync::Arc;
use tokio::sync::Mutex;

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
}

pub struct RollbackState<'a> {
    #[allow(dead_code)]
    agent_manager: &'a Arc<Mutex<AgentManager>>,
    resource_manager: &'a ResourceManager,
    runtime_manager: &'a RuntimeManager,
}

impl<'a> RollbackState<'a> {
    pub fn new(
        agent_manager: &'a Arc<Mutex<AgentManager>>,
        resource_manager: &'a ResourceManager,
        runtime_manager: &'a RuntimeManager,
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
                log::info!("Found backup: {}", backup_file.display());
                self.resource_manager.rollback(&backup_file)?;
                self.resource_manager.remove()?;
                self.runtime_manager
                    .update_state(crate::managers::runtime::State::Default)?;
                log::info!("Rollback completed");

                log::info!("Restarting controller by SIGINT");
                let current_pid = std::process::id();
                signal::kill(Pid::from_raw(current_pid as i32), Signal::SIGINT)
                    .map_err(|e| RollbackError::FailedKillOwnProcess(e.to_string()))?;

                Ok(())
            }
            None => Err(RollbackError::BackupNotFound),
        }
    }
}
