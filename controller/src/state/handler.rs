use crate::managers::{
    agent::AgentManagerTrait,
    runtime::{RuntimeError, RuntimeManager, State},
};
use crate::state::{
    default::{DefaultError, DefaultState},
    rollback::{RollbackError, RollbackState},
    update::{UpdateError, UpdateState},
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(unix)]
use crate::managers::resource::UnixResourceManager;

#[cfg(windows)]
use crate::managers::resource::WindowsResourceManager;


#[derive(Debug, thiserror::Error)]
pub enum StateHandlerError {
    #[error("update failed: {0}")]
    Update(#[from] UpdateError),
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

    pub async fn handle<A>(
        &self,
        runtime_manager: &Arc<RuntimeManager>,
        agent_manager: &Arc<Mutex<A>>,
    ) -> Result<(), StateHandlerError>
    where
        A: AgentManagerTrait + Sync + Send,
    {
        #[cfg(unix)]
        let resource_manager = UnixResourceManager::new();

        #[cfg(windows)]
        let resource_manager = WindowsResourceManager::new();

        match runtime_manager.get_state()? {
            State::Update => {
                {
                    let update_state =
                        UpdateState::new(agent_manager, resource_manager, runtime_manager);

                    if let Err(e) = update_state.execute().await {
                        self.handle_update_failed(runtime_manager, e)?;
                    }
                }
            }
            State::Rollback => {
                let rollback_state =
                    RollbackState::new(agent_manager, &resource_manager, runtime_manager);
                rollback_state.execute().await?;
            }
            State::Default => {
                let default_state = DefaultState::new(agent_manager, runtime_manager);
                default_state.execute().await?;
            }
            _ => {
                log::info!("No state change required.");
            }
        }

        Ok(())
    }

    fn handle_update_failed(
        &self,
        runtime_manager: &Arc<RuntimeManager>,
        update_error: UpdateError,
    ) -> Result<(), StateHandlerError> {
        log::error!("Failed to update state: {}", update_error);
        if let Some(target_state) = self.get_target_state(&update_error) {
            self.transition_to_state(runtime_manager, target_state)?;
        } else {
            log::warn!(
                "Skipping rollback state transition due to ignored update error: {}",
                update_error
            );
        }

        Err(StateHandlerError::Update(update_error))
    }

    fn get_target_state(&self, update_error: &UpdateError) -> Option<State> {
        if update_error.requires_rollback() {
            Some(State::Rollback)
        } else if update_error.required_restore_state() {
            Some(State::Default)
        } else {
            None
        }
    }

    fn transition_to_state(
        &self,
        runtime_manager: &Arc<RuntimeManager>,
        target_state: State,
    ) -> Result<(), StateHandlerError> {
        runtime_manager
            .update_state(target_state)
            .map_err(|runtime_err| {
                log::error!("Failed to transition to state: {}", runtime_err,);
                StateHandlerError::RuntimeInfo(runtime_err)
            })?;

        Ok(())
    }
}

impl Default for StateHandler {
    fn default() -> Self {
        Self::new()
    }
}
