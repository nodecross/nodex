use crate::managers::resource::ResourceManagerTrait;
use crate::managers::{
    agent::{AgentManagerError, AgentManagerTrait},
    runtime::{FeatType, RuntimeError, RuntimeInfoStorage, RuntimeManager},
};

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("agent process failed: {0}")]
    AgentError(#[from] AgentManagerError),
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
}

pub async fn execute<'a, A, R, H>(
    agent_manager: &'a mut A,
    _resource_manager: &'a R,
    runtime_manager: &'a mut RuntimeManager<H>,
) -> Result<(), InitError>
where
    A: AgentManagerTrait,
    R: ResourceManagerTrait,
    H: RuntimeInfoStorage,
{
    {
        let mut agent_processes = runtime_manager.filter_process_infos(FeatType::Agent)?;
        agent_processes
            .retain(|agent_process| runtime_manager.is_running_or_remove_if_stopped(agent_process));
        if !agent_processes.is_empty() {
            log::error!("Agent already running");
            runtime_manager.update_state(crate::managers::runtime::State::Idle)?;
            return Ok(());
        }
    }

    let process_info = agent_manager.launch_agent(true)?;
    runtime_manager.add_process_info(process_info)?;
    runtime_manager.update_state(crate::managers::runtime::State::Idle)?;
    Ok(())
}
