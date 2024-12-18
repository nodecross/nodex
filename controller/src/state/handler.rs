use crate::managers::{
    agent::AgentManagerTrait,
    runtime::{RuntimeError, RuntimeInfoStorage, RuntimeManager, State},
};
use crate::state::{init, rollback, update};
use std::sync::Arc;
use tokio::sync::Mutex;

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

pub async fn handle_state<A, H>(
    state: State,
    runtime_manager: &Arc<Mutex<RuntimeManager<H>>>,
    agent_manager: &Arc<Mutex<A>>,
) -> Result<(), StateHandlerError>
where
    A: AgentManagerTrait + Sync + Send,
    H: RuntimeInfoStorage + Sync + Send,
{
    let mut runtime_manager = runtime_manager.lock().await;
    let mut agent_manager = agent_manager.lock().await;

    let agent_path = runtime_manager.read_runtime_info()?.exec_path;
    #[cfg(unix)]
    let resource_manager = UnixResourceManager::new(agent_path);
    #[cfg(windows)]
    let resource_manager = WindowsResourceManager::new();

    match state {
        State::Update => {
            update::execute(
                &mut *agent_manager,
                &resource_manager,
                &mut *runtime_manager,
            )
            .await?;
            // ERASE: test for rollback
            // runtime_manager.update_state(crate::managers::runtime::State::Rollback)?;
        }
        State::Rollback => {
            rollback::execute(&*agent_manager, &resource_manager, &mut *runtime_manager).await?;
        }
        State::Init => {
            init::execute(
                &mut *agent_manager,
                &resource_manager,
                &mut *runtime_manager,
            )
            .await?;
        }
        State::Idle => {
            log::info!("No state change required.");
        }
    }

    Ok(())
}
