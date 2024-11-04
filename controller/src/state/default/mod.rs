use crate::config::get_config;
use crate::process::agent::{AgentProcessManager, AgentProcessManagerError};
use crate::runtime::{AgentInfo, RuntimeInfo};
use std::sync::{Arc, Mutex};

pub struct DefaultState;

impl DefaultState {
    pub fn handle(
        &self,
        agent_process_manager: &Arc<std::sync::Mutex<AgentProcessManager>>,
    ) -> Result<(), AgentProcessManagerError> {
        let manager = agent_process_manager.lock().unwrap();
        manager.launch_agent()
    }
}
