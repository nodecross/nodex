use crate::managers::{
    agent::{AgentManagerError, AgentManagerTrait},
    runtime::{FeatType, RuntimeError, RuntimeManager},
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum DefaultError {
    #[error("agent process failed: {0}")]
    AgentError(#[from] AgentManagerError),
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
}

pub struct DefaultState<'a, A>
where
    A: AgentManagerTrait,
{
    agent_manager: &'a Arc<Mutex<A>>,
    runtime_manager: &'a RuntimeManager,
}

impl<'a, A> DefaultState<'a, A>
where
    A: AgentManagerTrait,
{
    pub fn new(agent_manager: &'a Arc<Mutex<A>>, runtime_manager: &'a RuntimeManager) -> Self {
        DefaultState {
            agent_manager,
            runtime_manager,
        }
    }

    pub async fn execute(&self) -> Result<(), DefaultError> {
        let mut agent_processes = self.runtime_manager.filter_process_infos(FeatType::Agent)?;
        agent_processes.retain(|agent_process| {
            self.runtime_manager
                .is_running_or_remove_if_stopped(agent_process)
        });
        if agent_processes.len() > 1 {
            log::error!("Agent already running");
            return Ok(());
        }

        #[cfg(unix)]
        {
            let agent_manager = self.agent_manager.lock().await;
            let process_info = agent_manager.launch_agent()?;
            self.runtime_manager.add_process_info(process_info)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managers::{
        runtime::{ProcessInfo, FeatType, FileHandler},
        agent::{AgentManagerError, AgentManagerTrait},
    };
    use std::fs::File;
    use std::sync::Arc;
    use std::path::PathBuf;
    use tokio::sync::Mutex;
    use tempfile::tempdir;

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
