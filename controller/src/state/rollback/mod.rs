use crate::managers::{
    agent::{AgentProcessManager, AgentProcessManagerError},
    resource::ResourceManager,
    runtime::{RuntimeError, RuntimeManager},
};
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error("agent process failed: {0}")]
    AgentProcess(#[from] AgentProcessManagerError),
    #[error("Failed to find backup")]
    BackupNotFound,
    #[error("Failed to perform rollback: {0}")]
    RollbackFailed(#[from] std::io::Error),
    #[error("failed to get runtime info: {0}")]
    RuntimeInfo(#[from] RuntimeError),
}

pub struct RollbackState<'a> {
    agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
    resource_manager: &'a ResourceManager,
    runtime_manager: &'a RuntimeManager,
}

impl<'a> RollbackState<'a> {
    pub fn new(
        agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
        resource_manager: &'a ResourceManager,
        runtime_manager: &'a RuntimeManager,
    ) -> Self {
        RollbackState {
            agent_process_manager,
            resource_manager,
            runtime_manager,
        }
    }
    pub fn handle(&self) -> Result<(), RollbackError> {
        let latest_backup = self.resource_manager.get_latest_backup();
        match latest_backup {
            Some(backup_file) => {
                self.resource_manager.rollback(&backup_file)?;

                let agent_processes = self.runtime_manager.clean_and_get_running_agents()?;
                if agent_processes.is_empty() {
                    let agent_manager = self.agent_process_manager.lock().unwrap();
                    let process_info = agent_manager.launch_agent()?;
                    self.runtime_manager.add_process_info(process_info)?;
                }
                Ok(())
            }
            None => Err(RollbackError::BackupNotFound),
        }
    }
}
