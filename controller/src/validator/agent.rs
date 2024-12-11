use crate::managers::agent::{AgentManagerError, AgentManagerTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionResponse {
    pub version: String,
}

pub async fn is_latest_version<A>(
    agent_manager: &A,
    expected_version: String,
) -> Result<bool, AgentManagerError>
where
    A: AgentManagerTrait + Sync,
{
    println!("Checking version...");
    let version_response: VersionResponse =
        agent_manager.get_request("/internal/version/get").await?;

    Ok(version_response.version == expected_version)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::managers::runtime::{FeatType, ProcessInfo};
    use async_trait::async_trait;

    struct MockAgentManager {
        response_version: String,
    }

    #[async_trait]
    impl AgentManagerTrait for MockAgentManager {
        async fn get_request<T>(&self, _path: &str) -> Result<T, AgentManagerError>
        where
            T: serde::de::DeserializeOwned + Send,
        {
            if _path == "/internal/version/get" {
                let response = VersionResponse {
                    version: self.response_version.clone(),
                };
                let json_response = serde_json::to_string(&response).unwrap();
                let deserialized: T = serde_json::from_str(&json_response).unwrap();
                Ok(deserialized)
            } else {
                Err(AgentManagerError::RequestFailed("Invalid path".into()))
            }
        }

        fn launch_agent(&self) -> Result<ProcessInfo, AgentManagerError> {
            Ok(ProcessInfo::new(1, FeatType::Agent))
        }

        fn terminate_agent(&self, process_id: u32) -> Result<(), AgentManagerError> {
            Ok(())
        }

        fn cleanup(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_is_latest_version_returns_true_for_matching_version() {
        let agent_manager = MockAgentManager {
            response_version: "1.0.0".to_string(),
        };

        let result = is_latest_version(&agent_manager, "1.0.0".to_string()).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_is_latest_version_returns_false_for_different_version() {
        let agent_manager = MockAgentManager {
            response_version: "1.0.0".to_string(),
        };

        let result = is_latest_version(&agent_manager, "2.0.0".to_string()).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_is_latest_version_handles_request_failure() {
        struct FailingAgentManager;

        #[async_trait]
        impl AgentManagerTrait for FailingAgentManager {
            async fn get_request<T>(&self, _path: &str) -> Result<T, AgentManagerError>
            where
                T: serde::de::DeserializeOwned + Send,
            {
                Err(AgentManagerError::RequestFailed(
                    "Request failed".to_string(),
                ))
            }

            fn launch_agent(&self) -> Result<ProcessInfo, AgentManagerError> {
                Ok(ProcessInfo::new(1, FeatType::Agent))
            }

            fn terminate_agent(&self, process_id: u32) -> Result<(), AgentManagerError> {
                Ok(())
            }

            fn cleanup(&self) -> Result<(), std::io::Error> {
                Ok(())
            }
        }

        let agent_manager = FailingAgentManager;

        let result = is_latest_version(&agent_manager, "1.0.0".to_string()).await;
        assert!(result.is_err());
        match result {
            Err(AgentManagerError::RequestFailed(msg)) => {
                assert_eq!(msg, "Request failed");
            }
            _ => panic!("Expected RequestFailed error"),
        }
    }
}
