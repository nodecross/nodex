use crate::process::agent::AgentProcessManager;
use crate::runtime::{RuntimeInfo, State};
use crate::state::{default::DefaultState, updating::UpdatingState};
use std::sync::{Arc, Mutex};

pub struct StateHandler;

impl StateHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle(
        &self,
        current_state: &State,
        agent_process_manager: &Arc<std::sync::Mutex<AgentProcessManager>>,
    ) {
        match current_state {
            State::Updating => {
                let updating_state = UpdatingState {};
                updating_state.handle();
            }
            State::Default => {
                let default_state = DefaultState {};
                default_state.handle(agent_process_manager);
            }
        }
    }
}
