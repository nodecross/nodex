use crate::validator::process::is_running;
use chrono::{DateTime, FixedOffset, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub state: State,
    pub process_infos: Vec<ProcessInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum State {
    Default,
    Update,
    Updating,
    Rollback,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProcessInfo {
    pub process_id: u32,
    pub executed_at: DateTime<FixedOffset>,
    pub version: String,
    pub feat_type: FeatType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum FeatType {
    Agent,
    Controller,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Failed to open file: {0}")]
    FileOpen(#[source] std::io::Error),
    #[error("Failed to read file: {0}")]
    FileRead(#[source] std::io::Error),
    #[error("Failed to write data to file: {0}")]
    FileWrite(#[source] std::io::Error),
    #[error("Failed to acquire exclusive file lock: {0}")]
    FileLock(#[source] std::io::Error),
    #[error("Failed to unlock file: {0}")]
    FileUnlock(#[source] std::io::Error),
    #[error("Failed to serialize runtime info to JSON: {0}")]
    JsonSerialize(#[source] serde_json::Error),
    #[error("Failed to deserialize runtime info from JSON: {0}")]
    JsonDeserialize(#[source] serde_json::Error),
    #[error("Mutex poisoned")]
    MutexPoisoned,
}

pub struct FileHandler {
    path: PathBuf,
}

impl FileHandler {
    pub fn new(path: PathBuf) -> Self {
        FileHandler { path }
    }

    pub fn read(&self) -> Result<RuntimeInfo, RuntimeError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .map_err(RuntimeError::FileOpen)?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(RuntimeError::FileRead)?;

        serde_json::from_str(&content).map_err(RuntimeError::JsonDeserialize)
    }

    pub fn apply_with_lock<F>(&self, operation: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>,
    {
        let mut file = self.lock_file()?;
        let mut runtime_info = self.read_locked(&mut file)?;

        operation(&mut runtime_info)?;

        self.write_locked(&mut file, &runtime_info)?;
        self.unlock_file(&mut file)?;

        Ok(())
    }

    pub fn lock_file(&self) -> Result<std::fs::File, RuntimeError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path)
            .map_err(RuntimeError::FileOpen)?;
        file.lock_exclusive().map_err(RuntimeError::FileLock)?;
        Ok(file)
    }

    pub fn read_locked(&self, file: &mut std::fs::File) -> Result<RuntimeInfo, RuntimeError> {
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(RuntimeError::FileRead)?;

        if content.trim().is_empty() {
            Ok(RuntimeInfo {
                state: State::Default,
                process_infos: vec![],
            })
        } else {
            serde_json::from_str(&content).map_err(RuntimeError::JsonDeserialize)
        }
    }

    pub fn write_locked(
        &self,
        file: &mut std::fs::File,
        runtime_info: &RuntimeInfo,
    ) -> Result<(), RuntimeError> {
        let json_data =
            serde_json::to_string_pretty(runtime_info).map_err(RuntimeError::JsonSerialize)?;

        file.set_len(0).map_err(RuntimeError::FileWrite)?;

        file.seek(std::io::SeekFrom::Start(0))
            .map_err(RuntimeError::FileWrite)?;

        file.write_all(json_data.as_bytes())
            .map_err(RuntimeError::FileWrite)?;

        log::info!("File written successfully");
        Ok(())
    }

    pub fn unlock_file(&self, file: &mut std::fs::File) -> Result<(), RuntimeError> {
        file.unlock().map_err(RuntimeError::FileUnlock)
    }
}

pub struct RuntimeManager {
    file_handler: FileHandler,
}

impl RuntimeManager {
    pub fn new(file_handler: FileHandler) -> Self {
        RuntimeManager { file_handler }
    }

    pub fn read_runtime_info(&self) -> Result<RuntimeInfo, RuntimeError> {
        let runtime_info = if self.file_handler.path.exists() {
            self.file_handler.read()?
        } else {
            RuntimeInfo {
                state: State::Default,
                process_infos: vec![],
            }
        };

        Ok(runtime_info)
    }

    pub fn get_state(&self) -> Result<State, RuntimeError> {
        let runtime_info = self.read_runtime_info()?;

        Ok(runtime_info.state)
    }

    pub fn get_process_infos(&self) -> Result<Vec<ProcessInfo>, RuntimeError> {
        let runtime_info = self.read_runtime_info()?;

        Ok(runtime_info.process_infos)
    }

    pub fn filter_process_infos(
        &self,
        feat_type: FeatType,
    ) -> Result<Vec<ProcessInfo>, RuntimeError> {
        let process_infos = self.get_process_infos()?;
        Ok(process_infos
            .into_iter()
            .filter(|process_info| process_info.feat_type == feat_type)
            .collect::<Vec<ProcessInfo>>())
    }

    pub fn is_running_or_remove_if_stopped(&self, process_info: &ProcessInfo) -> bool {
        if !is_running(process_info.process_id) {
            self.remove_process_info(process_info.process_id)
                .map_err(|e| {
                    log::error!(
                        "Failed to remove process for process ID {}: {}",
                        process_info.process_id,
                        e
                    )
                })
                .ok();
            false
        } else {
            true
        }
    }

    pub fn add_process_info(&self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.process_infos.push(process_info);
            Ok(())
        })
    }

    pub fn remove_process_info(&self, process_id: u32) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info
                .process_infos
                .retain(|info| info.process_id != process_id);
            Ok(())
        })
    }

    pub fn update_state(&self, state: State) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.state = state;
            Ok(())
        })
    }
}

impl ProcessInfo {
    pub fn new(process_id: u32, feat_type: FeatType) -> Self {
        let now = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
        ProcessInfo {
            process_id,
            executed_at: now,
            version: env!("CARGO_PKG_VERSION").to_string(),
            feat_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    fn setup_temp_file() -> (RuntimeManager, tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_file_path = temp_dir.path().join("runtime_info.json");

        File::create(&temp_file_path).expect("Failed to create temporary runtime_info.json");

        assert!(
            temp_file_path.exists(),
            "Temporary file was not created: {:?}",
            temp_file_path
        );

        let file_handler = FileHandler::new(temp_file_path.clone());
        let runtime_manager = RuntimeManager::new(file_handler);

        (runtime_manager, temp_dir, temp_file_path)
    }

    #[test]
    fn test_read_write_runtime_info() {
        let (runtime_manager, _temp_dir, temp_file_path) = setup_temp_file();

        let initial_runtime_info = RuntimeInfo {
            state: State::Default,
            process_infos: vec![],
        };

        let file_handler = FileHandler::new(temp_file_path.clone());

        file_handler
            .write_locked(
                &mut File::create(&temp_file_path).unwrap(),
                &initial_runtime_info,
            )
            .unwrap();

        let read_runtime_info = runtime_manager.read_runtime_info().unwrap();
        assert_eq!(read_runtime_info.state, State::Default);
        assert_eq!(read_runtime_info.process_infos.len(), 0);
    }

    #[test]
    fn test_add_process_info() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        let process_info = ProcessInfo::new(12345, FeatType::Agent);
        runtime_manager
            .add_process_info(process_info.clone())
            .unwrap();

        let process_infos = runtime_manager.get_process_infos().unwrap();
        assert_eq!(process_infos.len(), 1);
        assert_eq!(process_infos[0].process_id, 12345);
        assert_eq!(process_infos[0].feat_type, FeatType::Agent);
    }

    #[test]
    fn test_remove_process_info() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        let process_info1 = ProcessInfo::new(12345, FeatType::Agent);
        let process_info2 = ProcessInfo::new(67890, FeatType::Controller);

        runtime_manager
            .add_process_info(process_info1.clone())
            .unwrap();
        runtime_manager
            .add_process_info(process_info2.clone())
            .unwrap();

        runtime_manager.remove_process_info(12345).unwrap();

        let process_infos = runtime_manager.get_process_infos().unwrap();
        assert_eq!(process_infos.len(), 1);
        assert_eq!(process_infos[0].process_id, 67890);
    }

    #[test]
    fn test_update_state() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        runtime_manager.update_state(State::Updating).unwrap();

        let state = runtime_manager.get_state().unwrap();
        assert_eq!(state, State::Updating);
    }

    #[test]
    fn test_filter_process_infos() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        let process_info1 = ProcessInfo::new(12345, FeatType::Agent);
        let process_info2 = ProcessInfo::new(67890, FeatType::Controller);

        runtime_manager
            .add_process_info(process_info1.clone())
            .unwrap();
        runtime_manager
            .add_process_info(process_info2.clone())
            .unwrap();

        let agents = runtime_manager
            .filter_process_infos(FeatType::Agent)
            .unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].process_id, 12345);

        let controllers = runtime_manager
            .filter_process_infos(FeatType::Controller)
            .unwrap();
        assert_eq!(controllers.len(), 1);
        assert_eq!(controllers[0].process_id, 67890);
    }

    #[test]
    fn test_is_running_or_remove_if_stopped() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        let process_info = ProcessInfo::new(12345, FeatType::Agent);

        runtime_manager
            .add_process_info(process_info.clone())
            .unwrap();

        let result = runtime_manager.is_running_or_remove_if_stopped(&process_info);
        assert!(!result);

        let process_infos = runtime_manager.get_process_infos().unwrap();
        assert!(process_infos.is_empty());
    }
}
