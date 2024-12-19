use super::runtime::{RuntimeError, RuntimeInfoStorage, RuntimeManager};
use serde::Deserialize;
use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(unix)]
mod unix_imports {
    pub use nix::{
        sys::signal::{self, Signal},
        unistd::{execvp, fork, setsid, ForkResult, Pid},
    };
    pub use std::ffi::CString;
    pub use std::os::unix::io::{AsRawFd, RawFd};
}

#[cfg(unix)]
use unix_imports::*;

use crate::managers::runtime::{FeatType, ProcessInfo};
use crate::validator::process::{is_manage_by_systemd, is_manage_socket_activation};

#[derive(Debug, Deserialize)]
struct VersionResponse {
    pub version: String,
}

#[cfg(unix)]
static DEFAULT_FD: RawFd = 3;

#[derive(Debug, thiserror::Error)]
pub enum AgentManagerError {
    #[error("Failed to use runtime: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("Failed to fork agent: {0}")]
    ForkAgentError(#[source] std::io::Error),
    #[error("LISTEN_FDS not set or invalid")]
    ListenFdsError,
    #[error("LISTEN_PID not set or invalid")]
    ListenPidError,
    #[error("LISTEN_PID ({listen_pid}) does not match current process ID ({current_pid})")]
    ListenPidMismatch { listen_pid: i32, current_pid: i32 },
    #[error("No file descriptors passed by systemd.")]
    NoFileDescriptors,
    #[cfg(unix)]
    #[error("Failed to bind UDS: {0}")]
    BindUdsError(#[source] std::io::Error),
    #[cfg(unix)]
    #[error("Failed to watch UDS: {0}")]
    WatchUdsError(#[source] notify::Error),
    #[cfg(unix)]
    #[error("Failed to terminate process: {0}")]
    TerminateProcessError(#[source] nix::Error),
    #[cfg(unix)]
    #[error("Request failed: {0}")]
    Request(#[from] crate::unix_utils::GetRequestError),
}

#[trait_variant::make(Send)]
pub trait AgentManagerTrait {
    fn launch_agent(&mut self, is_first: bool) -> Result<ProcessInfo, AgentManagerError>;

    fn terminate_agent(&mut self, process_id: u32) -> Result<(), AgentManagerError>;

    async fn get_version(&self) -> Result<String, AgentManagerError>;

    fn cleanup(&self) -> Result<(), std::io::Error>;
}

#[cfg(unix)]
pub struct UnixAgentManager<H: RuntimeInfoStorage> {
    uds_path: PathBuf,
    meta_uds_path: PathBuf,
    runtime_manager: RuntimeManager<H>,
}

#[cfg(unix)]
impl<H: RuntimeInfoStorage + Send + Sync> AgentManagerTrait for UnixAgentManager<H> {
    fn launch_agent(&mut self, is_first: bool) -> Result<ProcessInfo, AgentManagerError> {
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
        let current_exe = &self.runtime_manager.get_exec_path()?;
        let cmd = CString::new(current_exe.to_string_lossy().as_ref()).map_err(|e| {
            AgentManagerError::ForkAgentError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e,
            ))
        })?;
        let args = vec![cmd.clone()];

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                if is_first {
                    let listener = if is_manage_by_systemd() && is_manage_socket_activation() {
                        Some(Self::get_fd_from_systemd()?)
                    } else {
                        None
                    };
                    let () = crate::unix_utils::wait_until_file_created(&self.meta_uds_path)
                        .map_err(AgentManagerError::WatchUdsError)?;
                    let stream = loop {
                        match std::os::unix::net::UnixStream::connect(&self.meta_uds_path) {
                            Ok(stream) => break stream,
                            Err(err) if err.kind() == std::io::ErrorKind::ConnectionRefused => {
                                // Wait for bind
                                std::thread::sleep(std::time::Duration::from_millis(5));
                                continue;
                            }
                            Err(err) => return Err(AgentManagerError::BindUdsError(err)),
                        }
                    };
                    crate::unix_utils::send_fd(stream.as_raw_fd(), listener)
                        .map_err(|e| AgentManagerError::BindUdsError(e.into()))?;
                }
                let process_info = ProcessInfo::new(
                    child.as_raw().try_into().map_err(|_| {
                        AgentManagerError::ForkAgentError(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to convert child process ID to u32",
                        ))
                    })?,
                    FeatType::Agent,
                );
                Ok(process_info)
            }
            Ok(ForkResult::Child) => {
                setsid().map_err(|e| {
                    AgentManagerError::ForkAgentError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e,
                    ))
                })?;
                execvp(&cmd, &args).map_err(|e| {
                    AgentManagerError::ForkAgentError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e,
                    ))
                })?;
                unreachable!();
            }
            Err(e) => Err(AgentManagerError::ForkAgentError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e,
            ))),
        }
    }

    async fn get_version(&self) -> Result<String, AgentManagerError> {
        let version_response: VersionResponse =
            crate::unix_utils::get_request(&self.uds_path, "/internal/version/get").await?;
        Ok(version_response.version)
    }

    fn terminate_agent(&mut self, process_id: u32) -> Result<(), AgentManagerError> {
        log::info!("Terminating agent with PID: {}", process_id);
        signal::kill(Pid::from_raw(process_id as i32), Signal::SIGUSR1)
            .map_err(AgentManagerError::TerminateProcessError)?;
        self.runtime_manager.remove_process_info(process_id)?;
        Ok(())
    }

    fn cleanup(&self) -> Result<(), std::io::Error> {
        if self.uds_path.exists() {
            let _ = std::fs::remove_file(&self.uds_path);
        }
        if self.meta_uds_path.exists() {
            let _ = std::fs::remove_file(&self.meta_uds_path);
        }
        Ok(())
    }
}

#[cfg(unix)]
impl<H: RuntimeInfoStorage + Send> UnixAgentManager<H> {
    pub fn new(
        uds_path: impl AsRef<Path>,
        meta_uds_path: impl AsRef<Path>,
        runtime_manager: RuntimeManager<H>,
    ) -> Self {
        UnixAgentManager {
            uds_path: uds_path.as_ref().into(),
            meta_uds_path: meta_uds_path.as_ref().into(),
            runtime_manager,
        }
    }

    fn get_fd_from_systemd() -> Result<RawFd, AgentManagerError> {
        let listen_fds = env::var("LISTEN_FDS")
            .ok()
            .and_then(|x| x.parse::<i32>().ok())
            .ok_or(AgentManagerError::ListenFdsError)?;

        let listen_pid = env::var("LISTEN_PID")
            .ok()
            .and_then(|x| x.parse::<i32>().ok())
            .ok_or(AgentManagerError::ListenPidError)?;

        let current_pid = std::process::id() as i32;
        if listen_pid != current_pid {
            return Err(AgentManagerError::ListenPidMismatch {
                listen_pid,
                current_pid,
            });
        } else if listen_fds <= 0 {
            return Err(AgentManagerError::NoFileDescriptors);
        }

        Ok(DEFAULT_FD)
    }
}

#[cfg(unix)]
unsafe impl<H: RuntimeInfoStorage + Sync> Sync for UnixAgentManager<H> {}
unsafe impl<H: RuntimeInfoStorage + Send> Send for UnixAgentManager<H> {}

#[cfg(windows)]
pub struct WindowsAgentManager;

#[cfg(windows)]
impl AgentManagerTrait for WindowsAgentManager {
    fn launch_agent(&self) -> Result<ProcessInfo, AgentManagerError> {
        unimplemented!()
    }

    fn terminate_agent(&self, process_id: u32) -> Result<(), AgentManagerError> {
        unimplemented!()
    }

    async fn get_request<T>(&self, endpoint: &str) -> Result<T, AgentManagerError>
    where
        T: DeserializeOwned,
    {
        unimplemented!()
    }

    async fn parse_response_body<T>(
        &self,
        response: Response<Incoming>,
    ) -> Result<T, AgentManagerError>
    where
        T: DeserializeOwned,
    {
        unimplemented!()
    }

    fn cleanup(&self) -> Result<(), std::io::Error> {
        unimplemented!()
    }
}

#[cfg(windows)]
impl WindowsAgentManager {
    pub fn new() -> Result<Self, AgentManagerError> {
        Ok(WindowsAgentManager {})
    }
}
