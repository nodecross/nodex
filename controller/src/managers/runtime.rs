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
    pub use std::os::unix::io::AsRawFd;
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
    #[error("Failed to kill processes")]
    Kill(Vec<std::io::Error>),
    #[error("Failed to create command: {0}")]
    Command(#[source] std::io::Error),
    #[error("Failed to fork: {0}")]
    Fork(#[source] std::io::Error),
    #[error("failed to know path of self exe: {0}")]
    FailedCurrentExe(#[source] std::io::Error),
    #[error("Failed to bind UDS: {0}")]
    BindUdsError(#[source] std::io::Error),
    #[error("Failed to watch UDS: {0}")]
    WatchUdsError(#[source] notify::Error),
    #[error("Failed to get fd from systemd: {0}")]
    GetFd(#[from] crate::unix_utils::GetFdError),
    #[error("Request failed: {0}")]
    Request(#[from] crate::unix_utils::GetRequestError),
    #[error("Controller already running")]
    AlreadyExistController,
    #[error("Failed to get meta uds path")]
    PathConvention,
}

pub trait RuntimeInfoStorage: std::fmt::Debug {
    fn read(&mut self) -> Result<RuntimeInfo, RuntimeError>;
    fn apply_with_lock<F>(&mut self, operation: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>;
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeManager<H: RuntimeInfoStorage> {
    self_pid: u32,
    file_handler: H,
    uds_path: PathBuf,
    meta_uds_path: PathBuf,
    state_sender: watch::Sender<State>,
}

impl<H: RuntimeInfoStorage> RuntimeManager<H> {
    pub fn new_by_controller(
        mut file_handler: H,
        uds_path: impl AsRef<Path>,
    ) -> Result<(Self, watch::Receiver<State>), RuntimeError> {
        let runtime_info = file_handler.read()?;
        let (state_sender, state_receiver) = watch::channel(runtime_info.state);
        let meta_uds_path = crate::unix_utils::convention_of_meta_uds_path(&uds_path)
            .map_err(|_| RuntimeError::PathConvention)?;
        let self_pid = std::process::id();
        let mut runtime_manager = RuntimeManager {
            self_pid,
            file_handler,
            state_sender,
            uds_path: uds_path.as_ref().into(),
            meta_uds_path,
        };
        // We assume that caller is controller.
        let controller_processes = runtime_info
            .process_infos
            .iter()
            .filter(|process_info| {
                runtime_manager.is_running_or_remove_if_stopped(process_info)
                    && process_info.feat_type == FeatType::Controller
                    && process_info.process_id != self_pid
            })
            .collect::<Vec<&ProcessInfo>>();
        if !controller_processes.is_empty() {
            return Err(RuntimeError::AlreadyExistController);
        }
        let self_info = ProcessInfo::new(self_pid, FeatType::Controller);
        runtime_manager.add_process_info(self_info)?;
        Ok((runtime_manager, state_receiver))
    }

    pub fn new_by_agent(file_handler: H) -> Self {
        // dummy
        let (state_sender, _) = watch::channel(State::Init);
        let self_pid = std::process::id();
        let runtime_manager = RuntimeManager {
            self_pid,
            file_handler,
            state_sender,
            uds_path: "".into(),
            meta_uds_path: "".into(),
        };
        runtime_manager
    }

    pub fn launch_agent(&mut self, is_first: bool) -> Result<ProcessInfo, RuntimeError> {
        if is_first {
            if self.uds_path.exists() {
                log::warn!("UDS file already exists, removing: {:?}", self.uds_path);
                let _ = std::fs::remove_file(&self.uds_path);
            }
            if self.meta_uds_path.exists() {
                log::warn!(
                    "UDS file already exists, removing: {:?}",
                    self.meta_uds_path
                );
                let _ = std::fs::remove_file(&self.meta_uds_path);
            }
        }
        let current_exe = &self.get_exec_path()?;
        let cmd = CString::new(current_exe.to_string_lossy().as_ref()).map_err(|e| {
            RuntimeError::Fork(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
        })?;
        let args = vec![cmd.clone()];

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                if is_first {
                    let listener = if is_manage_by_systemd() && is_manage_socket_activation() {
                        Some(crate::unix_utils::get_fd_from_systemd()?)
                    } else {
                        None
                    };
                    let () = crate::unix_utils::wait_until_file_created(&self.meta_uds_path)
                        .map_err(RuntimeError::WatchUdsError)?;
                    let stream = loop {
                        match std::os::unix::net::UnixStream::connect(&self.meta_uds_path) {
                            Ok(stream) => break stream,
                            Err(err) if err.kind() == std::io::ErrorKind::ConnectionRefused => {
                                // Wait for bind
                                std::thread::sleep(std::time::Duration::from_millis(5));
                                continue;
                            }
                            Err(err) => return Err(RuntimeError::BindUdsError(err)),
                        }
                    };
                    crate::unix_utils::send_fd(stream.as_raw_fd(), listener)
                        .map_err(|e| RuntimeError::BindUdsError(e.into()))?;
                }
                let process_info = ProcessInfo::new(
                    child.as_raw().try_into().map_err(|_| {
                        RuntimeError::Fork(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to convert child process ID to u32",
                        ))
                    })?,
                    FeatType::Agent,
                );
                self.add_process_info(process_info.clone())?;
                Ok(process_info)
            }
            Ok(ForkResult::Child) => {
                setsid().map_err(|e| {
                    RuntimeError::Fork(std::io::Error::new(std::io::ErrorKind::Other, e))
                })?;
                execvp(&cmd, &args).map_err(|e| {
                    RuntimeError::Fork(std::io::Error::new(std::io::ErrorKind::Other, e))
                })?;
                unreachable!();
            }
            Err(e) => Err(RuntimeError::Fork(std::io::Error::new(
                std::io::ErrorKind::Other,
                e,
            ))),
        }
    }

    fn get_process_infos(&mut self) -> Result<Vec<ProcessInfo>, RuntimeError> {
        let runtime_info = self.file_handler.read()?;
        Ok(runtime_info.process_infos)
    }

    pub fn is_agent_running(&mut self) -> Result<bool, RuntimeError> {
        Ok(!self.filter_process_infos(FeatType::Agent)?.is_empty())
    }

    fn filter_process_infos(
        &mut self,
        feat_type: FeatType,
    ) -> Result<Vec<ProcessInfo>, RuntimeError> {
        let process_infos = self.get_process_infos()?;
        Ok(process_infos
            .into_iter()
            .filter(|process_info| process_info.feat_type == feat_type)
            .collect())
    }

    fn is_running_or_remove_if_stopped(&mut self, process_info: &ProcessInfo) -> bool {
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

    pub fn get_exec_path(&mut self) -> Result<PathBuf, RuntimeError> {
        let runtime_info = self.file_handler.read()?;
        Ok(runtime_info.exec_path)
    }

    fn add_process_info(&mut self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
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
            .map_err(|e| RuntimeError::Kill(vec![e.into()]))?;
        self.remove_process_info(process_info.process_id)?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<(), RuntimeError> {
        crate::unix_utils::remove_file_if_exists(&self.uds_path);
        crate::unix_utils::remove_file_if_exists(&self.meta_uds_path);
        self.kill_otherwise()?;
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.process_infos = vec![];
            runtime_info.state = State::Init;
            Ok(())
        })
        // let self_pid = self.self_info.process_id;
        // self.file_handler.apply_with_lock(move |runtime_info| {
        //     let mut errs = vec![];
        //     for process in runtime_info.process_infos.iter() {
        //         if process.process_id == self_pid {
        //             continue;
        //         }
        //         let signal = if process.feat_type == FeatType::Agent {
        //             Signal::SIGUSR1
        //         } else {
        //             Signal::SIGTERM
        //         };
        //         if let Err(e) = signal::kill(Pid::from_raw(process.process_id as i32), signal) {
        //             errs.push(e);
        //         }
        //     }
        //     runtime_info.process_infos = vec![];
        //     runtime_info.state = State::Init;
        //     if errs.is_empty() {
        //         Ok(())
        //     } else {
        //         Err(RuntimeError::Kill(errs.into_iter().map(|e| e.into()).collect()))
        //     }
        // })
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

    pub async fn get_version(&self) -> Result<String, RuntimeError> {
        let version_response: VersionResponse =
            crate::unix_utils::get_request(&self.uds_path, "/internal/version/get").await?;
        Ok(version_response.version)
    }

    pub fn kill_otherwise_agents(&mut self, target: u32) -> Result<(), RuntimeError> {
        let mut errs = vec![];
        let others = self
            .get_process_infos()?
            .into_iter()
            .filter(|_process_info| _process_info.process_id != target)
            .filter(|_process_info| _process_info.feat_type == FeatType::Agent);
        for other in others {
            let res = signal::kill(Pid::from_raw(other.process_id as i32), Signal::SIGUSR1);
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

    fn kill_otherwise(&mut self) -> Result<(), RuntimeError> {
        let mut errs = vec![];
        let self_pid = self.self_pid;
        let others = self
            .get_process_infos()?
            .into_iter()
            .filter(|process_info| process_info.process_id != self_pid);
        for other in others {
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

    pub fn launch_controller(
        &mut self,
        new_controller_path: impl AsRef<Path>,
    ) -> Result<(), RuntimeError> {
        dbg!(self.file_handler.read());
        self.kill_otherwise()?;
        dbg!(self.file_handler.read());
        if is_manage_by_systemd() && is_manage_socket_activation() {
            return Ok(());
        }
        crate::unix_utils::change_to_executable(new_controller_path.as_ref())
            .map_err(RuntimeError::Command)?;
        let new_controller_path =
            new_controller_path
                .as_ref()
                .to_str()
                .ok_or(RuntimeError::Command(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid path: failed to convert new_controller_path to string",
                )))?;
        let cmd = CString::new(new_controller_path).map_err(|x| RuntimeError::Command(x.into()))?;
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
