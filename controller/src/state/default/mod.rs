use crate::config::get_config;
use crate::process::agent::AgentProcessManager;
use crate::runtime::{AgentInfo, RuntimeInfo};

pub struct DefaultState;

impl DefaultState {
    pub fn handle(&self, runtime_info: &mut RuntimeInfo) {
        let uds_path = {
            let config = get_config().lock().unwrap();
            config.uds_path.clone()
        };
        let agent_manager = AgentProcessManager::new(&uds_path);
        if let Ok(agent_manager) = agent_manager {
            runtime_info.add_agent_info(AgentInfo {
                process_id: agent_manager.pid,
                executed_at: agent_manager.executed_at,
                version: agent_manager.version,
            });
        }
    }
}
