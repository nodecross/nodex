use crate::process::agent::{AgentProcessManager, AgentProcessManagerError};
use std::sync::{Arc, Mutex};

pub struct DefaultState<'a> {
    agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
}

impl<'a> DefaultState<'a> {
    pub fn new(agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>) -> Self {
        Self {
            agent_process_manager,
        }
    }

    pub fn handle(&self) -> Result<(), AgentProcessManagerError> {
        let manager = self.agent_process_manager.lock().unwrap();
        manager.launch_agent()
    }
}
