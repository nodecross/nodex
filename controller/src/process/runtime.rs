use chrono::{DateTime, FixedOffset, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::process::agent::AgentEventListener;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    #[serde(skip)]
    path: PathBuf,
    pub state: State,
    pub process_infos: Vec<ProcessInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum State {
    Default,
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
    #[error("not setting runtime path")]
    NotRuntimePath,
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
}


impl RuntimeInfo {
    pub fn new(path: PathBuf) -> Self {
        RuntimeInfo {
            path,
            state: State::Default,
            process_infos: vec![],
        }
    }

    pub fn read(&self) -> Result<Self, RuntimeError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .map_err(RuntimeError::FileOpen)?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(RuntimeError::FileRead)?;
        let mut runtime_info =
            serde_json::from_str(&content).map_err(RuntimeError::JsonSerialize)?;
        Ok(runtime_info)
    }

    pub fn reload(&mut self) -> Result<(), RuntimeError> {
        let updated_runtime_info = self.read()?;
        self.state = updated_runtime_info.state;
        self.process_infos = updated_runtime_info.process_infos;
        Ok(())
    }

    pub fn update_state(&mut self, state: State) -> Result<(), RuntimeError> {
        self.apply_with_reload_and_write(|runtime_info| {
            runtime_info.state = state.clone();
        })
    }
    
    pub fn add_process_info(&mut self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
        println!("Adding agent info: {}", process_info.process_id);
        self.apply_with_reload_and_write(|runtime_info| {
            runtime_info.process_infos.push(process_info.clone());
        })
    }

    pub fn remove_process_info(&mut self, process_id: u32) -> Result<(), RuntimeError> {
        self.apply_with_reload_and_write(|runtime_info| {
            runtime_info.process_infos.retain(|info| info.process_id != process_id);
        })
    }

    pub fn apply_with_reload_and_write<F>(&mut self, mut operation: F) -> Result<(), RuntimeError>
    where
        F: FnMut(&mut Self) -> (),
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path)
            .map_err(RuntimeError::FileOpen)?;
    
        file.lock_exclusive().map_err(RuntimeError::FileLock)?;
    
        self.reload()?;
    
        operation(self);
    
        let json_data = serde_json::to_string(self).map_err(RuntimeError::JsonSerialize)?;
    
        file.set_len(0).map_err(RuntimeError::FileWrite)?;
        file.write_all(json_data.as_bytes()).map_err(RuntimeError::FileWrite)?;
    
        file.unlock().map_err(RuntimeError::FileUnlock)?;
    
        Ok(())
    }

    // pub fn write(&self) -> Result<(), RuntimeError> {
    //     let mut file = OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .truncate(true)
    //         .open(&self.path)
    //         .map_err(RuntimeError::FileOpen)?;

    //     file.lock_exclusive().map_err(RuntimeError::FileLock)?;

    //     let json_data = serde_json::to_string(self).map_err(RuntimeError::JsonSerialize)?;

    //     file.write_all(json_data.as_bytes())
    //         .map_err(RuntimeError::FileWrite)?;

    //     file.unlock().map_err(RuntimeError::FileUnlock)
    // }

    // pub fn terminate_all_agents(&mut self) {
    //     for process_info in &self.process_infos {
    //         process_info.terminate();
    //     }
    //     self.process_infos.clear();
    // }
}

impl AgentEventListener for RuntimeInfo {
    fn on_agent_started(&mut self, process_info: ProcessInfo) {
        println!("Agent started with PID: {}", process_info.process_id);
        self.add_process_info(process_info);
    }

    fn on_agent_terminated(&mut self, process_id: u32) {
        println!("Agent terminated with PID: {}", process_id);
        self.remove_process_info(process_id);
    }
}

impl ProcessInfo {
    pub fn new(p_id: u32, feat_type: FeatType) -> Self {
        let now = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
        ProcessInfo {
            process_id: p_id,
            executed_at: now,
            version: env!("CARGO_PKG_VERSION").to_string(),
            feat_type,
        }
    }
}
