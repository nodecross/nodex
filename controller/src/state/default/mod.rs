use crate::managers::{
    agent::{AgentProcessManager, AgentProcessManagerError},
    runtime::{RuntimeError, RuntimeManager},
};
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum DefaultError {
    #[error("agent process failed: {0}")]
    AgentProcess(#[from] AgentProcessManagerError),
    #[error("failed to get runtime info: {0}")]
    RuntimeInfo(#[from] RuntimeError),
}

pub struct DefaultState<'a> {
    agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
    runtime_manager: &'a RuntimeManager,
}

impl<'a> DefaultState<'a> {
    pub fn new(
        agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
        runtime_manager: &'a RuntimeManager,
    ) -> Self {
        DefaultState {
            agent_process_manager,
            runtime_manager,
        }
    }

    pub fn handle(&self) -> Result<(), DefaultError> {
        let agent_processes = self.runtime_manager.clean_and_get_running_agents()?;
        if agent_processes.len() > 1 {
            log::error!("Agent already running");
            return Ok(());
        }

        let agent_manager = self.agent_process_manager.lock().unwrap();
        let process_info = agent_manager.launch_agent()?;
        self.runtime_manager.add_process_info(process_info)?;

        Ok(())
    }
}
