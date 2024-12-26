use crate::managers::runtime::{RuntimeError, RuntimeManager};

#[derive(Debug, thiserror::Error)]
pub enum IdleError {
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
}

pub async fn execute<T: RuntimeManager>(runtime_manager: &mut T) -> Result<(), IdleError> {
    if !runtime_manager.get_runtime_info()?.is_agent_running() {
        let _process_info = runtime_manager.launch_agent(true)?;
    } else {
        log::error!("Agent already running");
    }
    log::info!("No state change required.");
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::super::tests::MockRuntimeManager;
    use super::*;
    use crate::managers::runtime::{FeatType, ProcessInfo, RuntimeInfo, State};

    #[tokio::test]
    async fn test_execute_with_no_running_agents() {
        let runtime_info = RuntimeInfo {
            state: State::Idle,
            process_infos: [None, None, None, None],
            exec_path: std::env::current_exe().unwrap(),
        };
        let mut runtime_manager = MockRuntimeManager::new(runtime_info);

        let result = execute(&mut runtime_manager).await;
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);

        let process_infos: Vec<_> = runtime_manager
            .runtime_info
            .process_infos
            .iter()
            .flatten()
            .collect();
        assert_eq!(process_infos.len(), 1);
        assert_eq!(process_infos[0].feat_type, FeatType::Agent);
        assert_eq!(process_infos[0].process_id, 1);
    }

    #[tokio::test]
    async fn test_execute_with_one_running_agent() {
        let runtime_info = RuntimeInfo {
            state: State::Idle,
            process_infos: [
                Some(ProcessInfo::new(12345, FeatType::Agent)),
                None,
                None,
                None,
            ],
            exec_path: std::env::current_exe().unwrap(),
        };
        let mut runtime_manager = MockRuntimeManager::new(runtime_info);

        let result = execute(&mut runtime_manager).await;
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
    }
}
