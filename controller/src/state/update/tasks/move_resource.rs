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

pub fn run(src: &String, dest: &String) -> Result<(), MoveResourceError> {
    let src_path = Path::new(src).to_path_buf();
    if !src_path.exists() {
        return Err(MoveResourceError::SourceNotFoundError(src_path));
    } else if src_path.is_dir() {
        return Err(MoveResourceError::InvalidSourceFileName(src_path));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_run_success() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let src_file_path = temp_dir.path().join("source.txt");
        let dest_dir_path = temp_dir.path().join("destination");

        File::create(&src_file_path).expect("Failed to create source file");

        let result = run(
            &src_file_path.to_string_lossy().to_string(),
            &dest_dir_path.to_string_lossy().to_string(),
        );

        assert!(
            result.is_ok(),
            "Expected run to succeed, but got error: {:?}",
            result
        );

        let dest_file_path = dest_dir_path.join("source.txt");
        assert!(
            dest_file_path.exists(),
            "Expected file to be moved to {:?}, but it does not exist",
            dest_file_path
        );

        assert!(
            !src_file_path.exists(),
            "Expected source file to be removed, but it still exists"
        );
    }

    #[test]
    fn test_source_not_found_error() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let src_file_path = temp_dir.path().join("non_existent.txt");
        let dest_dir_path = temp_dir.path().join("destination");

        let result = run(
            &src_file_path.to_string_lossy().to_string(),
            &dest_dir_path.to_string_lossy().to_string(),
        );

        assert!(
            matches!(result, Err(MoveResourceError::SourceNotFoundError(_))),
            "Expected SourceNotFoundError, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_destination_creation_error() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let src_file_path = temp_dir.path().join("source.txt");

        File::create(&src_file_path).expect("Failed to create source file");

        let dest_dir_path = "/invalid/destination/directory";

        let result = run(
            &src_file_path.to_string_lossy().to_string(),
            &dest_dir_path.to_string(),
        );

        assert!(
            matches!(
                result,
                Err(MoveResourceError::DestinationCreationError(_, _))
            ),
            "Expected DestinationCreationError, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_destination_not_directory_error() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let src_file_path = temp_dir.path().join("source.txt");
        let dest_file_path = temp_dir.path().join("not_a_directory.txt");

        File::create(&src_file_path).expect("Failed to create source file");
        File::create(&dest_file_path).expect("Failed to create destination file");

        let result = run(
            &src_file_path.to_string_lossy().to_string(),
            &dest_file_path.to_string_lossy().to_string(),
        );

        assert!(
            matches!(
                result,
                Err(MoveResourceError::DestinationNotDirectoryError(_))
            ),
            "Expected DestinationNotDirectoryError, but got: {:?}",
            result
        );
    }

    #[test]
    fn test_invalid_source_file_name_error() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let dest_dir_path = temp_dir.path().join("destination");

        let result = run(
            &temp_dir.path().to_string_lossy().to_string(),
            &dest_dir_path.to_string_lossy().to_string(),
        );

        assert!(
            matches!(result, Err(MoveResourceError::InvalidSourceFileName(_))),
            "Expected InvalidSourceFileName, but got: {:?}",
            result
        );
    }
}
