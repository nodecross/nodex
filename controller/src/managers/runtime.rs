use crate::validator::process::is_running;
use crate::validator::process::{is_manage_by_systemd, is_manage_socket_activation};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::watch;

#[cfg(unix)]
mod unix_imports {
    pub use nix::{
        sys::signal::{self, Signal},
        unistd::{execvp, fork, setsid, ForkResult, Pid},
    };
    pub use std::ffi::CString;
}

#[cfg(unix)]
use unix_imports::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub state: State,
    pub process_infos: Vec<ProcessInfo>,
    pub exec_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum State {
    Init,
    Idle,
    Update,
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
    #[error("Failed to kill processes")]
    Kill(Vec<std::io::Error>),
    #[error("Failed to create command: {0}")]
    Command(#[source] std::io::Error),
    #[error("Failed to fork: {0}")]
    Fork(#[source] std::io::Error),
    #[error("failed to know path of self exe: {0}")]
    FailedCurrentExe(#[source] std::io::Error),
    #[cfg(unix)]
    #[error("Failed to terminate process: {0}")]
    TerminateProcessError(#[source] nix::Error),
}

pub trait RuntimeInfoStorage: std::fmt::Debug {
    fn read(&mut self) -> Result<RuntimeInfo, RuntimeError>;
    fn apply_with_lock<F>(&mut self, operation: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>;
}

#[derive(Debug, Clone)]
pub struct RuntimeManager<H: RuntimeInfoStorage> {
    self_pid: u32,
    file_handler: H,
    state_sender: watch::Sender<State>,
    state_receiver: watch::Receiver<State>,
}

impl<H: RuntimeInfoStorage> RuntimeManager<H> {
    pub fn new(mut file_handler: H) -> Result<Self, RuntimeError> {
        let runtime_info = file_handler.read()?;
        let (state_sender, state_receiver) = watch::channel(runtime_info.state);
        Ok(RuntimeManager {
            self_pid: std::process::id(),
            file_handler,
            state_sender,
            state_receiver,
        })
    }

    pub fn read_runtime_info(&mut self) -> Result<RuntimeInfo, RuntimeError> {
        let runtime_info = self.file_handler.read()?;
        Ok(runtime_info)
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
            .collect())
    }

    pub fn is_running_or_remove_if_stopped(&mut self, process_info: &ProcessInfo) -> bool {
        if !is_running(process_info.process_id) {
            let _ = self
                .remove_process_info(process_info.process_id)
                .map_err(|e| {
                    log::error!(
                        "Failed to remove process for process ID {}: {}",
                        process_info.process_id,
                        e
                    )
                });
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

    pub fn kill_process(&mut self, process_info: &ProcessInfo) -> Result<(), RuntimeError> {
        let signal = if process_info.feat_type == FeatType::Agent {
            Signal::SIGUSR1
        } else {
            Signal::SIGTERM
        };
        signal::kill(Pid::from_raw(process_info.process_id as i32), signal)
            .map_err(RuntimeError::TerminateProcessError)?;
        self.remove_process_info(process_info.process_id)?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<(), RuntimeError> {
        self.kill_others()?;
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.process_infos = vec![];
            runtime_info.state = State::Init;
            Ok(())
        })
    }

    pub fn update_state_without_send(&mut self, state: State) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.state = state;
            Ok(())
        })?;
        Ok(())
    }

    pub fn update_state(&mut self, state: State) -> Result<(), RuntimeError> {
        self.update_state_without_send(state)?;
        let _ = self.state_sender.send(state);
        Ok(())
    }

    pub fn get_state_receiver(&self) -> watch::Receiver<State> {
        self.state_receiver.clone()
    }

    #[cfg(unix)]
    fn kill_others(&mut self) -> Result<(), RuntimeError> {
        let mut errs = vec![];
        let self_pid = self.self_pid;
        let others = self
            .get_process_infos()?
            .into_iter()
            .filter(|process_info| process_info.process_id != self_pid);
        for other in others {
            // self.kill_process(&other)
            let signal = if other.feat_type == FeatType::Agent {
                Signal::SIGUSR1
            } else {
                Signal::SIGTERM
            };
            let res = signal::kill(Pid::from_raw(other.process_id as i32), signal);
            if let Err(e) = res {
                errs.push(e.into());
            } else {
                self.remove_process_info(other.process_id)?;
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::Kill(errs))
        }
    }

    #[cfg(unix)]
    pub fn run_controller(&mut self, agent_path: impl AsRef<Path>) -> Result<(), RuntimeError> {
        self.kill_others()?;
        if is_manage_by_systemd() && is_manage_socket_activation() {
            return Ok(());
        }
        crate::unix_utils::change_to_executable(agent_path.as_ref())
            .map_err(RuntimeError::Command)?;
        let agent_path =
            agent_path
                .as_ref()
                .to_str()
                .ok_or(RuntimeError::Command(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid path: failed to convert agent_path to string",
                )))?;
        let cmd = CString::new(agent_path).map_err(|x| RuntimeError::Command(x.into()))?;
        let args = vec![cmd.clone(), CString::new("controller").unwrap()];

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                log::info!("Parent process launched child with PID: {}", child);
                Ok(())
            }
            Ok(ForkResult::Child) => {
                setsid().map_err(|no| {
                    RuntimeError::Fork(std::io::Error::from_raw_os_error(no as core::ffi::c_int))
                })?;
                execvp(&cmd, &args).map_err(|no| {
                    RuntimeError::Fork(std::io::Error::from_raw_os_error(no as core::ffi::c_int))
                })?;
                unreachable!();
            }
            Err(e) => Err(RuntimeError::Fork(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to fork process: {}", e),
            ))),
        }
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
