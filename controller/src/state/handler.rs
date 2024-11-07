use crate::process::agent::{AgentProcessManager, AgentProcessManagerError};
use crate::runtime::State;
use crate::state::{
    default::DefaultState,
    updating::{UpdatingError, UpdatingState},
    resource::ResourceManager,
    rollback::{RollbackState, RollbackError}
};
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum StateHandlerError {
    #[error("updating failed: {0}")]
    Updating(#[from] UpdatingError),
    #[error("rollback failed: {0}")]
    Rollback(#[from] RollbackError),
    #[error("agent process failed: {0}")]
    AgentProcess(#[from] AgentProcessManagerError),
}

pub struct StateHandler;

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
                let resource_manager = ResourceManager::new();
                let updating_state = UpdatingState::new(resource_manager);
                updating_state.handle()?
            }
            State::Rollback => {
                let resource_manager = ResourceManager::new();
                let rollback_state = RollbackState::new(resource_manager);
                rollback_state.handle()?                
            }
            State::Default => {
                let default_state = DefaultState {};
                default_state.handle(agent_process_manager)
            }?,
        }

        Ok(())
    }
}
