use crate::config::get_config;
use bytes::Bytes;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use glob::glob;
use std::{
    env,
    fs::{self, File},
    io::{self, Cursor},
    path::{Path, PathBuf},
    time::SystemTime,
};
use tar::{Archive, Builder};
use zip::{result::ZipError, ZipArchive};

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Failed to download the file from {0}")]
    DownloadFailed(String),
    #[error("Failed to write to output path: {0}")]
    IoError(#[from] io::Error),
    #[error("Failed to extract zip file")]
    ZipError(#[from] ZipError),
    #[error("Failed to create tarball: {0}")]
    TarError(String),
    #[error("Failed to delete files in {0}")]
    RemoveFailed(String),
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}

pub struct ResourceManager {
    tmp_path: PathBuf,
}

impl ResourceManager {
    pub fn new() -> Self {
        let tmp_path = if PathBuf::from("/home/nodex/tmp").exists() {
            PathBuf::from("/home/nodex/tmp")
        } else if PathBuf::from("/tmp/nodex").exists() || fs::create_dir_all("/tmp/nodex").is_ok() {
            PathBuf::from("/tmp/nodex")
        } else {
            PathBuf::from("/tmp")
        };

        Self { tmp_path }
    }

    pub async fn download_update_resources(
        &self,
        binary_url: &str,
        output_path: Option<&PathBuf>,
    ) -> Result<(), ResourceError> {
        let download_path = output_path.unwrap_or(&self.tmp_path);

        let response = reqwest::get(binary_url)
            .await
            .map_err(|_| ResourceError::DownloadFailed(binary_url.to_string()))?;
        let content = response
            .bytes()
            .await
            .map_err(|_| ResourceError::DownloadFailed(binary_url.to_string()))?;

        self.extract_zip(content, download_path)?;
        Ok(())
    }

    fn extract_zip(&self, archive_data: Bytes, output_path: &Path) -> Result<(), ResourceError> {
        let cursor = Cursor::new(archive_data);
        let mut archive = ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = output_path.join(file.mangled_name());

            if file.is_file() {
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut output_file = std::fs::File::create(&file_path)?;
                std::io::copy(&mut file, &mut output_file)?;
            } else if file.is_dir() {
                std::fs::create_dir_all(&file_path)?;
            }
        }

        Ok(())
    }

    pub fn collect_downloaded_bundles(&self) -> Vec<PathBuf> {
        let pattern = self
            .tmp_path
            .join("bundles")
            .join("*.yml")
            .to_string_lossy()
            .into_owned();

        match glob(&pattern) {
            Ok(paths) => paths.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn backup(&self) -> Result<(), ResourceError> {
        let paths_to_backup = if PathBuf::from("/home/nodex").exists() {
            vec![PathBuf::from("/home/nodex")]
        } else {
            let config = get_config().lock().unwrap();
            vec![env::current_exe()?, config.config_dir.clone()]
        };

        self.create_tar_gz(paths_to_backup)?;
        log::info!("Backup created successfully");
        Ok(())
    }

    fn create_tar_gz(&self, src_paths: Vec<PathBuf>) -> Result<(), ResourceError> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| {
                ResourceError::TarError(format!("Failed to get current timestamp: {}", e))
            })?
            .as_secs();

        let dest_path = self
            .tmp_path
            .join(format!("nodex_backup_{}.tar.gz", timestamp));

        let tar_gz_file = File::create(&dest_path)
            .map_err(|e| ResourceError::IoError(io::Error::new(io::ErrorKind::Other, e)))?;
        let encoder = GzEncoder::new(tar_gz_file, Compression::default());
        let mut tar_builder = Builder::new(encoder);

        for path in &src_paths {
            if path == &env::current_exe()? {
                let original_path = path.strip_prefix("/").unwrap_or(path);
                tar_builder
                    .append_path_with_name(path, original_path)
                    .map_err(|e| {
                        ResourceError::TarError(format!("Failed to add nodex-agent to tar: {}", e))
                    })?;
            } else {
                let base_dir = path.parent().unwrap_or_else(|| Path::new(""));
                self.add_path_to_tar(&mut tar_builder, base_dir, path)?;
            }
        }

        tar_builder
            .finish()
            .map_err(|e| ResourceError::TarError(format!("Failed to finish tarball: {}", e)))?;

        log::info!("Tarball created at {:?}", dest_path);
        Ok(())
    }

    fn add_path_to_tar<W: std::io::Write>(
        &self,
        tar_builder: &mut Builder<W>,
        base_dir: &Path,
        path: &Path,
    ) -> Result<(), ResourceError> {
        let relative_path = path.strip_prefix(base_dir).map_err(|_| {
            ResourceError::TarError(format!("Failed to calculate relative path for {:?}", path))
        })?;

        if path.is_dir() {
            tar_builder
                .append_dir_all(relative_path, path)
                .map_err(|e| {
                    ResourceError::TarError(format!("Failed to append directory: {}", e))
                })?;
        } else if path.is_file() {
            tar_builder
                .append_path_with_name(path, relative_path)
                .map_err(|e| ResourceError::TarError(format!("Failed to append file: {}", e)))?;
        } else {
            return Err(ResourceError::TarError(format!(
                "Invalid path type: {:?}",
                path
            )));
        }
        Ok(())
    }

    pub fn get_latest_backup(&self) -> Option<PathBuf> {
        fs::read_dir(&self.tmp_path)
            .ok()?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("gz")
            })
            .max_by_key(|path| {
                path.metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            })
    }

    pub fn remove(&self) -> Result<(), ResourceError> {
        for entry in fs::read_dir(&self.tmp_path)
            .map_err(|e| ResourceError::RemoveFailed(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                ResourceError::RemoveFailed(format!("Failed to access entry: {}", e))
            })?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                fs::remove_dir_all(&entry_path).map_err(|e| {
                    ResourceError::RemoveFailed(format!("Failed to remove directory: {}", e))
                })?;
            } else if entry_path.is_file() {
                fs::remove_file(&entry_path).map_err(|e| {
                    ResourceError::RemoveFailed(format!("Failed to remove file: {}", e))
                })?;
            }
        }
        Ok(())
    }

    pub fn rollback(&self, backup_file: &PathBuf) -> Result<(), ResourceError> {
        let file = File::open(backup_file).map_err(|e| {
            ResourceError::RollbackFailed(format!(
                "Failed to open backup file {:?}: {}",
                backup_file, e
            ))
        })?;

        let decompressed = GzDecoder::new(file);
        let mut archive = Archive::new(decompressed);

        archive.unpack("/").map_err(|e| {
            ResourceError::RollbackFailed(format!(
                "Failed to unpack backup archive from {:?}: {}",
                backup_file, e
            ))
        })?;

        Ok(())
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
