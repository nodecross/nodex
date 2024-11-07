use crate::config::get_config;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use glob::glob;
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    time::SystemTime,
};
use tar::{Archive, Builder};

pub struct ResourceManager {
    tmp_path: PathBuf,
}

impl ResourceManager {
    pub fn new() -> Self {
        let tmp_path = if PathBuf::from("/home/nodex/tmp").exists()
            || fs::create_dir_all("/home/nodex/tmp").is_ok()
        {
            PathBuf::from("/home/nodex/tmp")
        } else if PathBuf::from("/tmp/nodex").exists() || fs::create_dir_all("/tmp/nodex").is_ok() {
            PathBuf::from("/tmp/nodex")
        } else {
            PathBuf::from("/tmp")
        };

        Self { tmp_path }
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

    pub fn backup(&self) -> Result<(), std::io::Error> {
        let paths_to_backup = if PathBuf::from("/home/nodex").exists() {
            vec![PathBuf::from("/home/nodex")]
        } else {
            let config = get_config().lock().unwrap();
            vec![
                env::current_exe()?,
                config.config_dir.clone(),
                config.runtime_dir.clone(),
            ]
        };

        self.create_tar_gz(paths_to_backup)?;
        log::info!("Backup created successfully");
        Ok(())
    }

    fn create_tar_gz(&self, src_paths: Vec<PathBuf>) -> Result<(), io::Error> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let dest_path = self
            .tmp_path
            .join(format!("nodex_backup_{}.tar.gz", timestamp));

        let tar_gz = File::create(dest_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        for path in src_paths {
            if path.is_dir() {
                tar.append_dir_all(path.file_name().unwrap(), &path)?;
            } else if path.is_file() {
                tar.append_path_with_name(&path, path.file_name().unwrap())?;
            }
        }

        tar.finish()?;
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

    pub fn rollback(&self, backup_file: &PathBuf) -> Result<(), io::Error> {
        let file = File::open(backup_file)?;
        let decompressed = GzDecoder::new(file);
        let mut archive = Archive::new(decompressed);

        archive.unpack("/")?;
        println!("Rollback completed from {:?}", backup_file);
        Ok(())
    }
}
