use crate::managers::{
    agent::{AgentManagerError, AgentManagerTrait},
    resource::{ResourceError, ResourceManagerTrait},
    runtime::{RuntimeError, RuntimeManager},
};
#[cfg(unix)]
pub use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error("agent process failed: {0}")]
    AgentError(#[from] AgentManagerError),
    #[error("Failed to find backup")]
    BackupNotFound,
    #[error("resource operation failed: {0}")]
    ResourceError(#[from] ResourceError),
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("failed to kill process: {0}")]
    FailedKillOwnProcess(String),
}

pub struct RollbackState<'a, A, R>
where
    A: AgentManagerTrait,
    R: ResourceManagerTrait,
{
    #[allow(dead_code)]
    agent_manager: &'a Arc<tokio::sync::Mutex<A>>,
    resource_manager: &'a R,
    runtime_manager: &'a RuntimeManager,
}

impl<'a, A, R> RollbackState<'a, A, R>
where
    A: AgentManagerTrait,
    R: ResourceManagerTrait,
{
    pub fn new(
        agent_manager: &'a Arc<tokio::sync::Mutex<A>>,
        resource_manager: &'a R,
        runtime_manager: &'a RuntimeManager,
    ) -> Self {
        RollbackState {
            agent_manager,
            resource_manager,
            runtime_manager,
        }
    }

    pub async fn execute(&self) -> Result<(), RollbackError> {
        log::info!("Starting rollback");

        let latest_backup = self.resource_manager.get_latest_backup();
        match latest_backup {
            Some(backup_file) => {
                log::info!("Found backup: {}", backup_file.display());
                self.resource_manager.rollback(&backup_file)?;
                self.resource_manager.remove()?;
                self.runtime_manager
                    .update_state(crate::managers::runtime::State::Default)?;
                log::info!("Rollback completed");

                log::info!("Restarting controller by SIGINT");
                // #[cfg(unix)]
                // {
                //     let current_pid = std::process::id();
                //     signal::kill(Pid::from_raw(current_pid as i32), Signal::SIGINT)
                //         .map_err(|e| RollbackError::FailedKillOwnProcess(e.to_string()))?;
                // }

                Ok(())
            }
            None => Err(RollbackError::BackupNotFound),
        }
    }
}


#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::managers::{
        agent::AgentManagerTrait,
        resource::{ResourceManagerTrait, ResourceError},
        runtime::{RuntimeManager, ProcessInfo, FeatType, FileHandler, State},
    };
    use tempfile::{tempdir, NamedTempFile};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct MockAgentManager;

    #[async_trait::async_trait]
    impl AgentManagerTrait for MockAgentManager {
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

    pub struct MockResourceManager {
        pub backup_file: Option<PathBuf>,
        pub rollback_called: std::sync::Mutex<bool>,
        pub remove_called: std::sync::Mutex<bool>,
    }
    
    impl MockResourceManager {
        pub fn new(backup_file: Option<PathBuf>) -> Self {
            Self {
                backup_file,
                rollback_called: std::sync::Mutex::new(false),
                remove_called: std::sync::Mutex::new(false),
            }
        }
    }

    impl ResourceManagerTrait for MockResourceManager {
        fn backup(&self) -> Result<(), ResourceError> {
            unimplemented!()
        }

        fn rollback(&self, _backup_file: &std::path::Path) -> Result<(), ResourceError> {
            let mut called = self.rollback_called.lock().unwrap();
            *called = true;
            Ok(())
        }

        fn tmp_path(&self) -> &PathBuf {
            unimplemented!()
        }

        fn get_paths_to_backup(&self) -> Result<Vec<PathBuf>, ResourceError> {
            unimplemented!()
        }

        fn collect_downloaded_bundles(&self) -> Vec<PathBuf> {
            unimplemented!()
        }

        fn get_latest_backup(&self) -> Option<PathBuf> {
            self.backup_file.clone()
        }

        fn extract_zip(&self, _archive_data: bytes::Bytes, _output_path: &std::path::Path) -> Result<(), ResourceError> {
            unimplemented!()
        }

        fn remove_directory(&self, _path: &std::path::Path) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn remove(&self) -> Result<(), ResourceError> {
            let mut called = self.remove_called.lock().unwrap();
            *called = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_execute_with_backup() {
        let agent = Arc::new(Mutex::new(MockAgentManager));
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let backup_file = temp_dir.path().join("backup.tar.gz");
        let resource = MockResourceManager::new(Some(backup_file));
        
        let temp_file_path = temp_dir.path().join("runtime_info.json");
        let file_handler = FileHandler::new(temp_file_path.clone());
        let runtime= RuntimeManager::new(file_handler);


        let state = RollbackState::new(&agent, &resource, &runtime);
        let result = state.execute().await;
        assert!(result.is_ok());

        {
            let rollback_called = *resource.rollback_called.lock().unwrap();
            assert!(rollback_called, "rollback should be called");
            let remove_called = *resource.remove_called.lock().unwrap();
            assert!(remove_called, "remove should be called");
        }

        {
            assert_eq!(runtime.get_state().unwrap(), State::Default, "update_state should be called with State::Default");
        }
    }

    #[tokio::test]
    async fn test_execute_without_backup() {
        let agent = Arc::new(Mutex::new(MockAgentManager));
        let resource = MockResourceManager::new(None);
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_file_path = temp_dir.path().join("runtime_info.json");
        let file_handler = FileHandler::new(temp_file_path.clone());
        let runtime= RuntimeManager::new(file_handler);

        let state = RollbackState::new(&agent, &resource, &runtime);
        let result = state.execute().await;
        assert!(result.is_err());

        match result {
            Err(RollbackError::BackupNotFound) => {},
            _ => panic!("Should return BackupNotFound error"),
        }
    }
}
