use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub state: State,
    agent_infos: Vec<AgentInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    Default,
    Updating,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub process_id: u32,
    pub executed_at: DateTime<FixedOffset>,
    pub version: String,
}

impl RuntimeInfo {
    pub fn default() -> Self {
        RuntimeInfo {
            state: State::Default,
            agent_infos: vec![],
        }
    }

    pub fn load_or_default(path: &PathBuf) -> Self {
        Self::read(path).unwrap_or_else(|_| Self::default())
    }

    pub fn read(path: &PathBuf) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read file content: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    pub fn add_agent_info(&mut self, agent_info: AgentInfo) {
        self.agent_infos.push(agent_info);
    }

    pub fn write(&self, path: &PathBuf) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let json_data =
            serde_json::to_string(self).map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        file.write_all(json_data.as_bytes())
            .map_err(|e| format!("Failed to write to file: {}", e))
    }
}
