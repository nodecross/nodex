use chrono::{DateTime, FixedOffset};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub state: State,
    agent_infos: Vec<AgentInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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

    pub fn load_or_default(path: &PathBuf, lock: Arc<Mutex<()>>) -> Self {
        let _guard = lock.lock().unwrap();
        Self::read(path).unwrap_or_else(|_| Self::default())
    }

    pub fn read(path: &PathBuf) -> Result<Self, String> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

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

    pub fn terminate_all_agents(&mut self) {
        for agent_info in &self.agent_infos {
            agent_info.terminate();
        }
        self.agent_infos.clear();
    }
}

impl AgentInfo {
    fn terminate(&self) {
        println!("Terminating agent with PID: {}", self.process_id);
        let _ = signal::kill(Pid::from_raw(self.process_id as i32), Signal::SIGTERM);
    }
}
