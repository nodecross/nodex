use crate::validator::process::is_running;
use chrono::{DateTime, FixedOffset, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use tokio::sync::watch;
// use memmap2::MmapMut;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub state: State,
    pub process_infos: Vec<ProcessInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum State {
    Default,
    Update,
    Updating,
    Rollback,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    // #[error("Failed to crate memory mapped io: {0}")]
    // MmapCreate(#[source] std::io::Error),
    // #[error("Failed to flush memory mapped io: {0}")]
    // MmapFlush(#[source] std::io::Error),
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

pub trait RuntimeInfoStorage: std::fmt::Debug {
    fn read(&mut self) -> Result<RuntimeInfo, RuntimeError>;
    fn apply_with_lock<F>(&mut self, operation: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>;
}

// pub struct MmapHandler {
//     mmap: MmapMut,
// }

// impl MmapHandler {
//     pub fn new(length: usize) -> Result<Self, RuntimeError>  {
//         let mmap = MmapMut::map_anon(length).map_err(RuntimeError::MmapCreate)?;
//         Ok(MmapHandler { mmap })
//     }

//     // pub fn lock_file(&self) -> Result<std::fs::File, RuntimeError> {
//     //     let file = OpenOptions::new()
//     //         .read(true)
//     //         .write(true)
//     //         .create(true)
//     //         .truncate(false)
//     //         .open(&self.path)
//     //         .map_err(RuntimeError::FileOpen)?;
//     //     file.lock_exclusive().map_err(RuntimeError::FileLock)?;
//     //     Ok(file)
//     // }

//     // pub fn read_locked(&self, file: &mut std::fs::File) -> Result<RuntimeInfo, RuntimeError> {
//     //     let mut content = String::new();
//     //     file.read_to_string(&mut content)
//     //         .map_err(RuntimeError::FileRead)?;

//     //     if content.trim().is_empty() {
//     //         Ok(RuntimeInfo {
//     //             state: State::Default,
//     //             process_infos: vec![],
//     //         })
//     //     } else {
//     //         serde_json::from_str(&content).map_err(RuntimeError::JsonDeserialize)
//     //     }
//     // }

//     // pub fn write_locked(
//     //     &self,
//     //     file: &mut std::fs::File,
//     //     runtime_info: &RuntimeInfo,
//     // ) -> Result<(), RuntimeError> {
//     //     let json_data =
//     //         serde_json::to_string_pretty(runtime_info).map_err(RuntimeError::JsonSerialize)?;

//     //     file.set_len(0).map_err(RuntimeError::FileWrite)?;

//     //     file.seek(std::io::SeekFrom::Start(0))
//     //         .map_err(RuntimeError::FileWrite)?;

//     //     file.write_all(json_data.as_bytes())
//     //         .map_err(RuntimeError::FileWrite)?;

//     //     log::info!("File written successfully");
//     //     Ok(())
//     // }

//     // pub fn unlock_file(&self, file: &mut std::fs::File) -> Result<(), RuntimeError> {
//     //     file.unlock().map_err(RuntimeError::FileUnlock)
//     // }
// }

// impl RuntimeInfoStorage for MmapHandler {
//     fn read(&self) -> Result<RuntimeInfo, RuntimeError> {
//         self.mmap.lock();
//         (&mut self.mmap[..]).read_to_end()
//         self.mmap.flush()?;
//         self.mmap.unlock();
//     }

//     fn apply_with_lock<F>(&self, operation: F) -> Result<(), RuntimeError>
//     where
//         F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>,
//     {
//         self.mmap.lock();

//         self.mmap.unlock();
//         Ok(())
//     }
// }

#[derive(Debug)]
pub struct FileHandler {
    file: File,
}

impl RuntimeInfoStorage for FileHandler {
    fn read(&mut self) -> Result<RuntimeInfo, RuntimeError> {
        let mut content = String::new();
        self.file
            .read_to_string(&mut content)
            .map_err(RuntimeError::FileRead)?;
        if content.is_empty() {
            return Ok(RuntimeInfo {
                state: State::Default,
                process_infos: vec![],
            });
        }
        serde_json::from_str(&content).map_err(RuntimeError::JsonDeserialize)
    }

    fn apply_with_lock<F>(&mut self, operation: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>,
    {
        self.file.lock_exclusive().map_err(RuntimeError::FileLock)?;
        let mut runtime_info = self.read_locked()?;

        operation(&mut runtime_info)?;

        self.write_locked(&runtime_info)?;
        self.file.unlock().map_err(RuntimeError::FileUnlock)?;

        Ok(())
    }
}

impl FileHandler {
    pub fn new(path: PathBuf) -> Result<Self, RuntimeError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(RuntimeError::FileOpen)?;
        Ok(FileHandler { file })
    }

    pub fn read_locked(&mut self) -> Result<RuntimeInfo, RuntimeError> {
        let mut content = String::new();
        self.file
            .read_to_string(&mut content)
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

    pub fn write_locked(&mut self, runtime_info: &RuntimeInfo) -> Result<(), RuntimeError> {
        let json_data =
            serde_json::to_string_pretty(runtime_info).map_err(RuntimeError::JsonSerialize)?;

        self.file.set_len(0).map_err(RuntimeError::FileWrite)?;

        self.file
            .seek(std::io::SeekFrom::Start(0))
            .map_err(RuntimeError::FileWrite)?;

        self.file
            .write_all(json_data.as_bytes())
            .map_err(RuntimeError::FileWrite)?;

        log::info!("File written successfully");
        Ok(())
    }
}

#[derive(Debug)]
pub struct RuntimeManager<H: RuntimeInfoStorage> {
    file_handler: H,
    state_sender: watch::Sender<State>,
    state_receiver: watch::Receiver<State>,
}

impl<H: RuntimeInfoStorage> RuntimeManager<H> {
    pub fn new(file_handler: H) -> Self {
        let (state_sender, state_receiver) = watch::channel(State::Default);
        RuntimeManager {
            file_handler,
            state_sender,
            state_receiver,
        }
    }

    pub fn read_runtime_info(&mut self) -> Result<RuntimeInfo, RuntimeError> {
        let runtime_info = self.file_handler.read()?;

        Ok(runtime_info)
    }

    pub fn get_state(&mut self) -> Result<State, RuntimeError> {
        let runtime_info = self.read_runtime_info()?;

        Ok(runtime_info.state)
    }

    pub fn get_process_infos(&mut self) -> Result<Vec<ProcessInfo>, RuntimeError> {
        let runtime_info = self.read_runtime_info()?;

        Ok(runtime_info.process_infos)
    }

    pub fn filter_process_infos(
        &mut self,
        feat_type: FeatType,
    ) -> Result<Vec<ProcessInfo>, RuntimeError> {
        let process_infos = self.get_process_infos()?;
        Ok(process_infos
            .into_iter()
            .filter(|process_info| process_info.feat_type == feat_type)
            .collect::<Vec<ProcessInfo>>())
    }

    pub fn is_running_or_remove_if_stopped(&mut self, process_info: &ProcessInfo) -> bool {
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

    pub fn add_process_info(&mut self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.process_infos.push(process_info);
            Ok(())
        })
    }

    pub fn remove_process_info(&mut self, process_id: u32) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info
                .process_infos
                .retain(|info| info.process_id != process_id);
            Ok(())
        })
    }

    pub fn update_state(&mut self, state: State) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.state = state;
            Ok(())
        })?;
        let _ = self.state_sender.send(state);

        Ok(())
    }

    pub fn get_state_receiver(&self) -> watch::Receiver<State> {
        self.state_receiver.clone()
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
