use crate::config::get_config;
use crate::process::agent::AgentProcessManager;
use crate::runtime::{AgentInfo, RuntimeInfo};
use std::sync::{Arc, Mutex};

pub struct DefaultState;

impl DefaultState {
    pub fn handle(&self, agent_process_manager: &Arc<std::sync::Mutex<AgentProcessManager>>) {
        let mut manager = agent_process_manager.lock().unwrap();
        if let Err(e) = manager.launch_agent() {
            log::error!("Failed to launch agent: {}", e);
        } else {
            log::info!("Agent successfully launched.");
        }
    }
}
