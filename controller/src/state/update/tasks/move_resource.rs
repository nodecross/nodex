use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum MoveResourceError {
    #[error("Source file '{0}' not found or is not a file")]
    SourceNotFoundError(PathBuf),
    #[error("Destination directory '{0}' does not exist and could not be created: {1}")]
    DestinationCreationError(PathBuf, #[source] std::io::Error),
    #[error("Destination path '{0}' is not a directory")]
    DestinationNotDirectoryError(PathBuf),
    #[error("Invalid source file name for '{0}'")]
    InvalidSourceFileName(PathBuf),
    #[error("Failed to move file from '{0}' to '{1}': {2}")]
    FileMoveError(PathBuf, PathBuf, #[source] std::io::Error),
}

pub fn execute(src: &String, dest: &String) -> Result<(), MoveResourceError> {
    let src_path = Path::new(src).to_path_buf();
    if !src_path.exists() || !src_path.is_file() {
        return Err(MoveResourceError::SourceNotFoundError(src_path));
    }

    let dest_path = Path::new(dest).to_path_buf();
    if !dest_path.exists() {
        log::info!(
            "Destination directory does not exist. Creating directory: {}",
            dest
        );
        fs::create_dir_all(&dest_path)
            .map_err(|e| MoveResourceError::DestinationCreationError(dest_path.clone(), e))?;
    } else if !dest_path.is_dir() {
        return Err(MoveResourceError::DestinationNotDirectoryError(dest_path));
    }

    let file_name = src_path
        .file_name()
        .ok_or_else(|| MoveResourceError::InvalidSourceFileName(src_path.clone()))?;
    let dest_file_path = dest_path.join(file_name);

    log::info!("Moving file from {} to {}", src, dest_file_path.display());
    fs::rename(&src_path, &dest_file_path)
        .map_err(|e| MoveResourceError::FileMoveError(src_path, dest_file_path, e))?;

    Ok(())
}
