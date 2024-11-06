use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::process::agent::AgentEventListener;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub state: State,
    pub process_infos: Vec<ProcessInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum State {
    Default,
    Updating,
}

#[derive(Debug, Serialize, Deserialize)]
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
    #[error("Failed to open or access file: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed to parse JSON content: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

impl RuntimeInfo {
    pub fn default() -> Self {
        RuntimeInfo {
            state: State::Default,
            process_infos: vec![],
        }
    }

    pub fn load_or_default(path: &PathBuf, lock: Arc<Mutex<()>>) -> Self {
        let _guard = lock.lock().unwrap();
        Self::read(path).unwrap_or_else(|_| Self::default())
    }

    pub fn read(path: &PathBuf) -> Result<Self, RuntimeError> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let runtime_info = serde_json::from_str(&content)?;
        Ok(runtime_info)
    }

    pub fn add_process_info(&mut self, process_info: ProcessInfo) {
        println!("Adding agent info: {}", process_info.process_id);
        self.process_infos.push(process_info);
    }

    pub fn write(&self, path: &PathBuf) -> Result<(), RuntimeError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let json_data = serde_json::to_string(self)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

    pub fn remove_process_info(&mut self, process_id: u32) {
        self.process_infos
            .retain(|process_info| process_info.process_id != process_id);
    }

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
