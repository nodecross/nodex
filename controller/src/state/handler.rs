use crate::process::agent::{AgentProcessManager, AgentProcessManagerError};
use crate::runtime::State;
use crate::state::{
    default::DefaultState,
    updating::{UpdatingError, UpdatingState},
};
use std::sync::{Arc, Mutex};

pub struct StateHandler;

#[derive(Debug, thiserror::Error)]
pub enum StateHandlerError {
    #[error("updating failed: {0}")]
    Updating(#[from] UpdatingError),
    #[error("agent process failed: {0}")]
    AgentProcess(#[from] AgentProcessManagerError),
}

impl StateHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle(
        &self,
        current_state: &State,
        agent_process_manager: &Arc<Mutex<AgentProcessManager>>,
    ) -> Result<(), StateHandlerError> {
        match current_state {
            State::Updating => {
                let updating_state = UpdatingState {};
                updating_state.handle()?
            }
            State::Default => {
                let default_state = DefaultState {};
                default_state.handle(agent_process_manager)
            }?,
        }

        Ok(())
    }
}
