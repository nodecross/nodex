use crate::managers::agent::{AgentManager, AgentManagerError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VersionResponse {
    pub version: String,
}

#[cfg(unix)]
pub async fn is_latest_version(
    agent_manager: &AgentManager,
    expected_version: String,
) -> Result<bool, AgentManagerError> {
    let version_response: VersionResponse =
        agent_manager.get_request("/internal/version/get").await?;

    Ok(version_response.version == expected_version)
}
