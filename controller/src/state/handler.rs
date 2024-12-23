use crate::managers::runtime::{RuntimeError, RuntimeManager, State};
use crate::state::{init, rollback, update};

#[cfg(unix)]
use crate::managers::resource::UnixResourceManager;

#[cfg(windows)]
use crate::managers::resource::WindowsResourceManager;

#[derive(Debug, thiserror::Error)]
pub enum StateHandlerError {
    #[error("update failed: {0}")]
    Update(#[from] update::UpdateError),
    #[error("rollback failed: {0}")]
    Rollback(#[from] rollback::RollbackError),
    #[error("default failed: {0}")]
    Init(#[from] init::InitError),
    #[error("failed to get runtime info: {0}")]
    RuntimeInfo(#[from] RuntimeError),
}

pub async fn handle_state<R: RuntimeManager>(
    state: State,
    runtime_manager: &mut R,
) -> Result<(), StateHandlerError> {
    let agent_path = runtime_manager.get_exec_path()?;
    #[cfg(unix)]
    let resource_manager = UnixResourceManager::new(agent_path);
    #[cfg(windows)]
    let resource_manager = WindowsResourceManager::new();

    match state {
        State::Update => {
            update::execute(&resource_manager, runtime_manager).await?;
            // ERASE: test for rollback
            // runtime_manager.update_state(crate::managers::runtime::State::Rollback)?;
        }
        State::Rollback => {
            rollback::execute(&resource_manager, runtime_manager).await?;
        }
        State::Init => {
            init::execute(&resource_manager, runtime_manager).await?;
        }
        State::Idle => {
            log::info!("No state change required.");
        }
    }

    Ok(())
}
