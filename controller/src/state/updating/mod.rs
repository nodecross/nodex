mod action;

use crate::state::updating::action::UpdateAction;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum UpdatingError {
    #[error("Failed to read YAML file: {0}")]
    YamlReadError(#[source] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[source] serde_yaml::Error),
    #[error("Failed to run actions: {0}")]
    ActionError(#[source] Box<dyn Error>),
    #[error("Failed to find download path")]
    FindDownloadPathError,
    #[error("Failed to find bundle")]
    BundleNotFoundError,
    #[error("Invalid version format")]
    InvalidVersionFormat,
}

pub struct UpdatingState;

impl UpdatingState {
    pub fn handle(&self) -> Result<(), UpdatingError> {
        let download_path = self.find_download_path()?;

        let bundles = self.collect_downloaded_bundles(&download_path);
        if bundles.is_empty() {
            return Err(UpdatingError::BundleNotFoundError);
        };

        let update_actions = self.parse_bundles(&bundles)?;
        if update_actions.is_empty() {
            return Err(UpdatingError::YamlParseError);
        };

        let pending_update_actions = self.extract_pending_update_actions(&update_actions)?;
        if pending_update_actions.is_empty() {
            Ok(())
        }

        for action in pending_update_actions {
            action.run();
        }

        println!("downloading binary");

        Ok(())
    }

    fn find_download_path(&self) -> Result<PathBuf, UpdatingError> {
        let download_path = if PathBuf::from("/home/nodex/tmp").exists() {
            PathBuf::from("/home/nodex/tmp")
        } else if PathBuf::from("/tmp/nodex/").exists() {
            PathBuf::from("/tmp/nodex/")
        } else {
            return Err(UpdatingError::FindDownloadPathError);
        };

        Ok(download_path)
    }

    fn parse_bundles(&self, bundles: &Vec<PathBuf>) -> Result<Vec<UpdateAction>, UpdatingError> {
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

    fn downloaded_binary_path(&self, download_path: &PathBuf) -> Result<(), UpdatingError> {
        // Implement the logic to download the binary here
        Ok(())
    }

    fn collect_downloaded_bundles(&self, download_path: &PathBuf) -> Vec<PathBuf> {
        let pattern = download_path
            .join("bundles")
            .join("*.yml")
            .to_string_lossy()
            .into_owned();

        glob(&pattern)
            .unwrap_or_else(|_| Vec::new().into_iter())
            .filter_map(Result::ok)
            .collect()
    }

    fn extract_pending_update_actions(
        &self,
        update_actions: &[UpdateAction],
    ) -> Result<Vec<&UpdateAction>, UpdatingError> {
        let current_version =
            Version::parse(build::VERSION).map_err(|_| UpdatingError::InvalidVersionFormat)?;

        let pending_actions: Vec<&UpdateAction> = update_actions
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
