pub mod action;

use crate::state::updating::action::UpdateAction;
use crate::state::resource::ResourceManager;
use semver::Version;
use serde_yaml::Error as SerdeYamlError;
use std::fs;
use std::path::PathBuf;

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
}

pub struct UpdatingState {
    resource_manager: ResourceManager,
}

impl UpdatingState {
    pub fn new(resource_manager: ResourceManager) -> Self {
        Self { resource_manager }
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

        log::info!("downloading binary");

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

    pub fn extract_pending_update_actions<'a>(
        &'a self,
        update_actions: &'a [UpdateAction],
    ) -> Result<Vec<&'a UpdateAction>, UpdatingError> {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|_| UpdatingError::InvalidVersionFormat)?;

        let pending_actions: Vec<&'a UpdateAction> = update_actions
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
}
