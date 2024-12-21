use crate::managers::resource::ResourceManagerTrait;
use crate::managers::runtime::{ProcessManager, RuntimeError, RuntimeInfoStorage, RuntimeManager};

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
}

pub async fn execute<'a, R, H, P>(
    _resource_manager: &'a R,
    runtime_manager: &'a mut RuntimeManager<H, P>,
) -> Result<(), InitError>
where
    R: ResourceManagerTrait,
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    if !runtime_manager.is_agent_running()? {
        let _process_info = runtime_manager.launch_agent(true)?;
    } else {
        log::error!("Agent already running");
    }
    runtime_manager.update_state(crate::managers::runtime::State::Idle)?;
    Ok(())
}