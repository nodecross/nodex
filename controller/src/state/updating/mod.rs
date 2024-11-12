pub mod action;

use crate::process::agent::{AgentProcessManager, AgentProcessManagerError};
use crate::state::resource::ResourceManager;
use crate::state::updating::action::UpdateAction;
use semver::Version;
use serde_yaml::Error as SerdeYamlError;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum UpdatingError {
    #[error("Failed to find bundle")]
    BundleNotFound,
    #[error("Failed to run actions: {0}")]
    ActionError(#[source] Box<dyn std::error::Error>),
    #[error("Failed to read YAML file: {0}")]
    YamlReadError(#[source] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[source] SerdeYamlError),
    #[error("Invalid version format")]
    InvalidVersionFormat,
    #[error("agent process failed: {0}")]
    AgentProcess(#[from] AgentProcessManagerError),
}

pub struct UpdatingState<'a> {
    resource_manager: ResourceManager,
    agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
}

impl<'a> UpdatingState<'a> {
    pub fn new(
        resource_manager: ResourceManager,
        agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
    ) -> Self {
        Self {
            resource_manager,
            agent_process_manager,
        }
    }

    pub fn handle(&self) -> Result<(), UpdatingError> {
        let bundles = self.resource_manager.collect_downloaded_bundles();
        if bundles.is_empty() {
            return Err(UpdatingError::BundleNotFound);
        }

        let update_actions = self.parse_bundles(&bundles)?;
        let pending_update_actions = self.extract_pending_update_actions(&update_actions)?;
        if pending_update_actions.is_empty() {
            return Ok(());
        }

        for action in pending_update_actions {
            if let Err(e) = action.run() {
                return Err(UpdatingError::ActionError(Box::new(e)));
            }
        }

        self.launch_new_version_agent()?;
        // monitor new version agent
        // self.terminate_old_version_agent()?;

        Ok(())
    }

    pub fn parse_bundles(&self, bundles: &[PathBuf]) -> Result<Vec<UpdateAction>, UpdatingError> {
        bundles
            .iter()
            .map(|bundle| {
                let yaml_content =
                    fs::read_to_string(bundle).map_err(UpdatingError::YamlReadError)?;
                let update_action: UpdateAction =
                    serde_yaml::from_str(&yaml_content).map_err(UpdatingError::YamlParseError)?;
                Ok(update_action)
            })
            .collect()
    }

    pub fn extract_pending_update_actions<'b>(
        &'b self,
        update_actions: &'b [UpdateAction],
    ) -> Result<Vec<&'b UpdateAction>, UpdatingError> {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|_| UpdatingError::InvalidVersionFormat)?;

        let pending_actions: Vec<&'b UpdateAction> = update_actions
            .iter()
            .filter_map(|action| {
                let target_version = Version::parse(&action.version).ok()?;
                if target_version > current_version {
                    Some(action)
                } else {
                    None
                }
            })
            .collect();

        Ok(pending_actions)
    }

    pub fn launch_new_version_agent(&self) -> Result<(), UpdatingError> {
        let agent_process_manager = self.agent_process_manager.lock().unwrap();
        agent_process_manager.launch_agent()?;

        Ok(())
    }

    pub fn terminate_old_version_agent(&self, process_id: u32) -> Result<(), UpdatingError> {
        let agent_process_manager = self.agent_process_manager.lock().unwrap();
        agent_process_manager.terminate_agent(process_id)?;

        Ok(())
    }
}
