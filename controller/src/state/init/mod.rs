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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::managers::{
        agent::{AgentManagerError, AgentManagerTrait},
        runtime::{FeatType, FileHandler, ProcessInfo},
    };
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::sync::Mutex;

    struct TestAgentManager;

    #[async_trait::async_trait]
    impl AgentManagerTrait for TestAgentManager {
        fn launch_agent(&self) -> Result<ProcessInfo, AgentManagerError> {
            Ok(ProcessInfo::new(1, FeatType::Agent))
        }
        fn terminate_agent(&self, _process_id: u32) -> Result<(), AgentManagerError> {
            Ok(())
        }

        async fn get_request<T>(&self, _endpoint: &str) -> Result<T, AgentManagerError>
        where
            T: serde::de::DeserializeOwned + Send,
        {
            Err(AgentManagerError::RequestFailed("Invalid request".into()))
        }

        fn cleanup(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_execute_with_no_running_agents() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_file_path = temp_dir.path().join("runtime_info.json");
        let file_handler = FileHandler::new(temp_file_path.clone());
        let runtime_manager = RuntimeManager::new(file_handler);

        let agent_manager = Arc::new(Mutex::new(TestAgentManager));

        let default_state = DefaultState::new(&agent_manager, &runtime_manager);

        let result = default_state.execute().await;
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);

        let runtime_info = runtime_manager.read_runtime_info().unwrap();
        assert_eq!(runtime_info.process_infos.len(), 1);
        assert_eq!(runtime_info.process_infos[0].feat_type, FeatType::Agent);
        assert_eq!(runtime_info.process_infos[0].process_id, 1);
    }

    #[tokio::test]
    async fn test_execute_with_one_running_agent() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_file_path = temp_dir.path().join("runtime_info.json");
        let file_handler = FileHandler::new(temp_file_path.clone());
        let runtime_manager = RuntimeManager::new(file_handler);
        runtime_manager.add_process_info(ProcessInfo::new(12345, FeatType::Agent));

        let agent_manager = Arc::new(Mutex::new(TestAgentManager));

        let default_state = DefaultState::new(&agent_manager, &runtime_manager);

        let result = default_state.execute().await;
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
    }
}
