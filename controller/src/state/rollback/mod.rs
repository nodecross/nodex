use crate::managers::{
    resource::{ResourceError, ResourceManagerTrait},
    runtime::{RuntimeError, RuntimeManager},
};

#[cfg(unix)]
pub use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error("Failed to find backup")]
    BackupNotFound,
    #[error("resource operation failed: {0}")]
    ResourceError(#[from] ResourceError),
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("failed to kill process: {0}")]
    FailedKillOwnProcess(String),
    #[error("Failed to get current executable path: {0}")]
    CurrentExecutablePathError(#[source] std::io::Error),
}

pub async fn execute<'a, R, T>(
    resource_manager: &'a R,
    runtime_manager: &'a mut T,
) -> Result<(), RollbackError>
where
    R: ResourceManagerTrait,
    T: RuntimeManager,
{
    log::info!("Starting rollback");

    let latest_backup = resource_manager.get_latest_backup();
    match latest_backup {
        Some(backup_file) => {
            let agent_path = runtime_manager.get_runtime_info()?.exec_path;
            log::info!("Found backup: {}", backup_file.display());
            resource_manager.rollback(&backup_file)?;
            if let Err(err) = resource_manager.remove() {
                log::error!("Failed to remove files {}", err);
            }
            runtime_manager.update_state_without_send(crate::managers::runtime::State::Init)?;
            runtime_manager.launch_controller(agent_path)?;
            log::info!("Rollback completed");

            #[cfg(not(test))] // failed test by kill own process
            {
                log::info!("Restarting controller by SIGTERM");
                let runtime_info = runtime_manager.get_runtime_info()?;
                let self_info = runtime_info.find_process_info(std::process::id()).ok_or(
                    RollbackError::FailedKillOwnProcess("Failed to find self info".into()),
                )?;
                runtime_manager.kill_process(self_info)?;
            }
            Ok(())
        }
        None => Err(RollbackError::BackupNotFound),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::super::tests::{MockResourceManager, MockRuntimeManager};
    use super::*;
    use crate::managers::runtime::{RuntimeInfo, RuntimeManagerWithoutAsync, State};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_execute_with_backup() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let backup_file = temp_dir.path().join("backup.tar.gz");
        let resource = MockResourceManager::new(vec![backup_file]);
        let runtime_info = RuntimeInfo {
            state: State::Rollback,
            process_infos: [None, None, None, None],
            exec_path: "".into(),
        };
        let mut runtime = MockRuntimeManager::new(runtime_info);

        let result = execute(&resource, &mut runtime).await;
        assert!(result.is_ok());

        let state = runtime.get_runtime_info().unwrap().state;
        assert_eq!(
            state,
            State::Init,
            "update_state should be called with State::Init"
        );

        {
            let rollback_called = *resource.rollback_called.lock().unwrap();
            assert!(rollback_called, "rollback should be called");
            let remove_called = *resource.remove_called.lock().unwrap();
            assert!(remove_called, "remove should be called");
        }
    }

    #[tokio::test]
    async fn test_execute_without_backup() {
        let resource = MockResourceManager::new(vec![]);
        let runtime_info = RuntimeInfo {
            state: State::Rollback,
            process_infos: [None, None, None, None],
            exec_path: "".into(),
        };
        let mut runtime = MockRuntimeManager::new(runtime_info);

        let result = execute(&resource, &mut runtime).await;
        assert!(result.is_err());

        match result {
            Err(RollbackError::BackupNotFound) => {}
            _ => panic!("Should return BackupNotFound error"),
        }
    }
}
