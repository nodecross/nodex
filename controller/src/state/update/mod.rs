pub mod tasks;

use crate::managers::{
    agent::{AgentProcessManager, AgentProcessManagerError},
    resource::ResourceManager,
    runtime::{RuntimeError, RuntimeManager, State},
};
use crate::state::update::tasks::UpdateAction;
use semver::Version;
use serde_yaml::Error as SerdeYamlError;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Failed to find bundle")]
    BundleNotFound,
    #[error("Invalid version format")]
    InvalidVersionFormat,
    #[error("Failed to run actions: {0}")]
    ActionError(#[source] Box<dyn std::error::Error>),
    #[error("Failed to read YAML file: {0}")]
    YamlReadError(#[source] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[source] SerdeYamlError),
    #[error("Failed to update state: {0}")]
    UpdateStateFailed(#[source] RuntimeError),
    #[error("agent process failed: {0}")]
    AgentProcess(#[from] AgentProcessManagerError),
    #[error("failed to get runtime info: {0}")]
    RuntimeInfo(#[from] RuntimeError),
}

pub struct UpdateState<'a> {
    agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
    resource_manager: ResourceManager,
    runtime_manager: &'a RuntimeManager,
}

impl<'a> UpdateState<'a> {
    pub fn new(
        agent_process_manager: &'a Arc<Mutex<AgentProcessManager>>,
        resource_manager: ResourceManager,
        runtime_manager: &'a RuntimeManager,
    ) -> Self {
        Self {
            agent_process_manager,
            resource_manager,
            runtime_manager,
        }
    }

    pub fn execute(&self) -> Result<(), UpdateError> {
        self.runtime_manager
            .update_state(State::Updating)
            .map_err(UpdateError::UpdateStateFailed)?;

        let bundles = self.resource_manager.collect_downloaded_bundles();
        let update_actions = self.parse_bundles(&bundles)?;
        let pending_update_actions = self.extract_pending_update_actions(&update_actions)?;
        if pending_update_actions.is_empty() {
            return Ok(());
        }

        for action in pending_update_actions {
            if let Err(e) = action.handle() {
                return Err(UpdateError::ActionError(Box::new(e)));
            }
        }

        self.launch_new_version_agent()?;
        // monitor new version agent
        // self.terminate_old_version_agent()?;

        Ok(())
    }

    pub fn parse_bundles(&self, bundles: &[PathBuf]) -> Result<Vec<UpdateAction>, UpdateError> {
        bundles
            .iter()
            .map(|bundle| {
                let yaml_content =
                    fs::read_to_string(bundle).map_err(UpdateError::YamlReadError)?;
                let update_action: UpdateAction =
                    serde_yaml::from_str(&yaml_content).map_err(UpdateError::YamlParseError)?;
                Ok(update_action)
            })
            .collect()
    }

    pub fn extract_pending_update_actions<'b>(
        &'b self,
        update_actions: &'b [UpdateAction],
    ) -> Result<Vec<&'b UpdateAction>, UpdateError> {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|_| UpdateError::InvalidVersionFormat)?;

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

    pub fn launch_new_version_agent(&self) -> Result<(), UpdateError> {
        let agent_manager = self.agent_process_manager.lock().unwrap();
        let process_info = agent_manager.launch_agent()?;
        self.runtime_manager.add_process_info(process_info)?;

        Ok(())
    }

    pub fn terminate_old_version_agent(&self, process_id: u32) -> Result<(), UpdateError> {
        let agent_process_manager = self.agent_process_manager.lock().unwrap();
        agent_process_manager.terminate_agent(process_id)?;
        self.runtime_manager.remove_process_info(process_id)?;

        Ok(())
    }
}
