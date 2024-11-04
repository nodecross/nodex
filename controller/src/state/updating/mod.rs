mod actions;

use std::fs;
use std::error::Error;
use std::path::{Path, PathBuf};
use crate::state::updating::actions::UpdateYaml;
use crate::state::updating::actions::run_actions;

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
}

pub struct UpdatingState;

impl UpdatingState {
    pub fn handle(&self) -> Result<(), UpdatingError> {
        let download_path = self.find_download_path()?;

        let bundles = self.collect_downloaded_bundles(&download_path);
        let filtered_bundles = self.filter_bundles(&bundles)?;
        let yaml_content = fs::read_to_string("example.yaml").map_err(|e| UpdatingError::YamlReadError(e))?;
        let update_yaml: UpdateYaml = serde_yaml::from_str(&yaml_content).map_err(|e| UpdatingError::YamlParseError(e))?;
        run_actions(update_yaml.steps).map_err(|e| UpdatingError::ActionError(e))?;

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

    fn downloaded_binary_path(&self, download_path: &PathBuf) -> Result<(), UpdatingError> {
        // Implement the logic to download the binary here
        Ok(())
    }

    fn collect_downloaded_bundles(&self, download_path: &PathBuf) -> Vec<PathBuf> {
        fs::read_dir(download_path.join("bundles"))
            .unwrap_or_else(|_| fs::read_dir(Path::new("")).unwrap())
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file()) 
            .collect::<Vec<PathBuf>>()
    }

    fn filter_bundles(&self, bundles: &Vec<PathBuf>) -> Result<(), UpdatingError> {
        // Implement the logic to filter the bundles here
        Ok(())
    }
}
