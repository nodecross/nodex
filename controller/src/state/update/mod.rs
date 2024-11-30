pub mod tasks;

use crate::managers::{
    agent::{AgentManager, AgentManagerError},
    resource::{ResourceError, ResourceManager},
    runtime::{FeatType, RuntimeError, RuntimeManager, State},
};
use crate::state::update::tasks::{UpdateAction, UpdateActionError};
#[cfg(unix)]
use crate::validator::agent::is_latest_version;
use semver::Version;
use serde_yaml::Error as SerdeYamlError;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{self, Instant};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Failed to find bundle")]
    BundleNotFound,
    #[error("Invalid version format")]
    InvalidVersionFormat,
    #[error("update action error: {0}")]
    UpdateActionFailed(#[from] UpdateActionError),
    #[error("Failed to read YAML file: {0}")]
    YamlReadFailed(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlParseFailed(#[source] SerdeYamlError),
    #[error("Failed to update state: {0}")]
    UpdateStateFailed(#[source] RuntimeError),
    #[error("Failed to Agent version check: {0}")]
    AgentVersionCheckFailed(String),
    #[error("agent operation failed: {0}")]
    AgentError(#[from] AgentManagerError),
    #[error("runtime operation failed: {0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("resource operation failed: {0}")]
    ResourceError(#[from] ResourceError),
    #[error("Agent not running")]
    AgentNotRunning,
}

impl UpdateError {
    pub fn required_restore_state(&self) -> bool {
        !matches!(self, UpdateError::AgentNotRunning)
    }

    pub fn requires_rollback(&self) -> bool {
        !matches!(
            self,
            UpdateError::ResourceError(ResourceError::RemoveFailed(_))
        )
    }
}

pub struct UpdateState<'a> {
    agent_manager: &'a Arc<Mutex<AgentManager>>,
    resource_manager: ResourceManager,
    runtime_manager: &'a RuntimeManager,
}

impl<'a> UpdateState<'a> {
    pub fn new(
        agent_manager: &'a Arc<Mutex<AgentManager>>,
        resource_manager: ResourceManager,
        runtime_manager: &'a RuntimeManager,
    ) -> Self {
        Self {
            agent_manager,
            resource_manager,
            runtime_manager,
        }
    }

    pub async fn execute(&self) -> Result<(), UpdateError> {
        log::info!("Starting update");

        if self
            .runtime_manager
            .filter_process_infos(FeatType::Agent)?
            .is_empty()
        {
            return Err(UpdateError::AgentNotRunning);
        }

        self.runtime_manager
            .update_state(State::Updating)
            .map_err(UpdateError::UpdateStateFailed)?;

        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|_| UpdateError::InvalidVersionFormat)?;

        let bundles = self.resource_manager.collect_downloaded_bundles();
        let update_actions = self.parse_bundles(&bundles)?;
        let pending_update_actions =
            self.extract_pending_update_actions(&update_actions, &current_version)?;
        for action in pending_update_actions {
            action.handle()?;
        }

        self.launch_new_version_agent().await?;
        self.monitor_agent_version(self.agent_manager, &current_version)
            .await?;
        self.terminate_old_version_agent(current_version.to_string())
            .await?;

        self.resource_manager.remove()?;

        log::info!("Update completed");

        Ok(())
    }

    fn parse_bundles(&self, bundles: &[PathBuf]) -> Result<Vec<UpdateAction>, UpdateError> {
        bundles
            .iter()
            .map(|bundle| {
                let yaml_content = fs::read_to_string(bundle)?;
                let update_action: UpdateAction =
                    serde_yaml::from_str(&yaml_content).map_err(UpdateError::YamlParseFailed)?;
                Ok(update_action)
            })
            .collect()
    }

    fn extract_pending_update_actions<'b>(
        &'b self,
        update_actions: &'b [UpdateAction],
        current_version: &Version,
    ) -> Result<Vec<&'b UpdateAction>, UpdateError> {
        let pending_actions: Vec<&'b UpdateAction> = update_actions
            .iter()
            .filter_map(|action| {
                let target_version = Version::parse(&action.version).ok()?;
                if target_version > *current_version {
                    Some(action)
                } else {
                    None
                }
            })
            .collect();

        Ok(pending_actions)
    }

    async fn launch_new_version_agent(&self) -> Result<(), UpdateError> {
        let agent_manager = self.agent_manager.lock().await;
        let process_info = agent_manager.launch_agent()?;
        self.runtime_manager.add_process_info(process_info)?;

        Ok(())
    }

    async fn monitor_agent_version(
        &self,
        agent_manager: &'a Arc<Mutex<AgentManager>>,
        expected_version: &Version,
    ) -> Result<(), UpdateError> {
        let timeout = Duration::from_secs(180);
        let interval = Duration::from_secs(3);

        let start = Instant::now();
        let mut interval_timer = time::interval(interval);

        while start.elapsed() < timeout {
            interval_timer.tick().await;

            match self.check_version(agent_manager, expected_version).await {
                Ok(true) => {
                    log::info!("Expected version received: {}", expected_version);
                    return Ok(());
                }
                Ok(false) => {
                    log::info!("Version did not match expected value.");
                }
                Err(e) => {
                    log::error!("Error occurred during version check: {}", e);
                }
            }
        }

        Err(UpdateError::AgentVersionCheckFailed(format!(
            "Expected version '{}' was not received within {:?}.",
            expected_version, timeout
        )))
    }

    #[cfg(unix)]
    async fn check_version(
        &self,
        agent_manager: &'a Arc<Mutex<AgentManager>>,
        expected_version: &Version,
    ) -> Result<bool, UpdateError> {
        let manager = agent_manager.lock().await;
        is_latest_version(&manager, expected_version.to_string())
            .await
            .map_err(|e| UpdateError::AgentVersionCheckFailed(e.to_string()))
    }
    #[cfg(windows)]
    async fn check_version(
        &self,
        agent_manager: &'a Arc<Mutex<AgentManager>>,
        expected_version: &Version,
    ) -> Result<bool, UpdateError> {
    }

    async fn terminate_old_version_agent(
        &self,
        current_version: String,
    ) -> Result<(), UpdateError> {
        let agent_processes = self.runtime_manager.filter_process_infos(FeatType::Agent)?;

        for agent_process in agent_processes {
            if agent_process.version == current_version {
                continue;
            }
            let agent_manager = self.agent_manager.lock().await;
            agent_manager.terminate_agent(agent_process.process_id)?;
            self.runtime_manager
                .remove_process_info(agent_process.process_id)?;
        }

        Ok(())
    }
}
