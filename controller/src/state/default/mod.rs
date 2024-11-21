use crate::managers::{
    agent::{AgentManager, AgentManagerError},
    runtime::{FeatType, RuntimeError, RuntimeManager},
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum DefaultError {
    #[error("agent process failed: {0}")]
    AgentProcess(#[from] AgentManagerError),
    #[error("failed to get runtime info: {0}")]
    RuntimeInfo(#[from] RuntimeError),
}

pub struct DefaultState<'a> {
    agent_manager: &'a Arc<Mutex<AgentManager>>,
    runtime_manager: &'a RuntimeManager,
}

impl<'a> DefaultState<'a> {
    pub fn new(
        agent_manager: &'a Arc<Mutex<AgentManager>>,
        runtime_manager: &'a RuntimeManager,
    ) -> Self {
        DefaultState {
            agent_manager,
            runtime_manager,
        }
    }

    pub async fn execute(&self) -> Result<(), DefaultError> {
        let mut agent_processes = self.runtime_manager.filter_process_infos(FeatType::Agent)?;
        agent_processes.retain(|agent_process| {
            self.runtime_manager
                .remove_and_filter_running_process(agent_process)
        });
        if agent_processes.len() > 1 {
            log::error!("Agent already running");
            return Ok(());
        }

        let agent_manager = self.agent_manager.lock().await;
        let process_info = agent_manager.launch_agent()?;
        self.runtime_manager.add_process_info(process_info)?;

        Ok(())
    }
}
