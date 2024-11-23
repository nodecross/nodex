use crate::managers::{
    agent::{AgentManager, AgentManagerError},
    resource::{ResourceError, ResourceManager},
    runtime::{FeatType, RuntimeError, RuntimeManager},
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
}

pub struct RollbackState<'a> {
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

                let mut agent_processes =
                    self.runtime_manager.filter_process_infos(FeatType::Agent)?;
                agent_processes.retain(|agent_process| {
                    self.runtime_manager
                        .is_running_or_remove_if_stopped(agent_process)
                });

                if agent_processes.is_empty() {
                    let agent_manager = self.agent_manager.lock().await;
                    let process_info = agent_manager.launch_agent()?;
                    self.runtime_manager.add_process_info(process_info)?;
                }

                self.resource_manager.remove()?;
                log::info!("Rollback completed");

                Ok(())
            }
            None => Err(RollbackError::BackupNotFound),
        }
    }
}
