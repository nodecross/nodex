use crate::managers::{
    agent::AgentProcessManager,
    resource::ResourceManager,
    runtime::{RuntimeError, RuntimeManager, State},
};
use crate::state::{
    default::{DefaultError, DefaultState},
    rollback::{RollbackError, RollbackState},
    update::{UpdateError, UpdateState},
};
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum StateHandlerError {
    #[error("updating failed: {0}")]
    Updating(#[from] UpdateError),
    #[error("rollback failed: {0}")]
    Rollback(#[from] RollbackError),
    #[error("default failed: {0}")]
    Default(#[from] DefaultError),
    #[error("failed to get runtime info: {0}")]
    RuntimeInfo(#[from] RuntimeError),
}

pub struct StateHandler;

impl StateHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle(
        &self,
        runtime_manager: &RuntimeManager,
        agent_process_manager: &Arc<Mutex<AgentProcessManager>>,
    ) -> Result<(), StateHandlerError> {
        match runtime_manager.get_state()? {
            State::Update => {
                let resource_manager = ResourceManager::new();
                let update_state =
                    UpdateState::new(agent_process_manager, resource_manager, runtime_manager);
                update_state.handle()?
            }
            State::Rollback => {
                let resource_manager = ResourceManager::new();
                let rollback_state =
                    RollbackState::new(agent_process_manager, &resource_manager, runtime_manager);
                rollback_state.handle()?
            }
            State::Default => {
                let default_state = DefaultState::new(agent_process_manager, runtime_manager);
                default_state.handle()?
            }
            _ => {
                log::info!("No state change required.");
            }
        }

        Ok(())
    }
}

impl Default for StateHandler {
    fn default() -> Self {
        Self::new()
    }
}
