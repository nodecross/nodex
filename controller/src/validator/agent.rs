use crate::managers::agent::{AgentManagerError, AgentManagerTrait};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VersionResponse {
    pub version: String,
}

pub async fn is_latest_version<A>(
    agent_manager: &A,
    expected_version: String,
) -> Result<bool, AgentManagerError>
where
    A: AgentManagerTrait,
{
    let version_response: VersionResponse =
        agent_manager.get_request("/internal/version/get").await?;

    Ok(version_response.version == expected_version)
}
