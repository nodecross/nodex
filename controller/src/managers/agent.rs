use nix::sys::signal::{self, Signal};
use nix::unistd::{dup, Pid};
use std::env;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::managers::runtime::{FeatType, ProcessInfo};
use crate::validator::process::{is_manage_by_systemd, is_manage_socket_activation};

static DEFAULT_FD: RawFd = 3;

#[derive(Debug, thiserror::Error)]
pub enum AgentProcessManagerError {
    #[error("Failed to get current executable path")]
    CurrentExecutablePathError(#[source] std::io::Error),
    #[error("Failed to fork agent")]
    ForkAgentError(#[source] std::io::Error),
    #[error("LISTEN_FDS not set or invalid")]
    ListenFdsError,
    #[error("LISTEN_PID not set or invalid")]
    ListenPidError,
    #[error("LISTEN_PID ({listen_pid}) does not match current process ID ({current_pid})")]
    ListenPidMismatch { listen_pid: i32, current_pid: i32 },
    #[error("No file descriptors passed by systemd.")]
    NoFileDescriptors,
    #[error("Failed to bind UDS: {0}")]
    BindUdsError(#[source] std::io::Error),
    #[error("Failed to duplicate file descriptor")]
    DuplicateFdError(#[source] nix::Error),
    #[error("Failed to terminate process: {0}")]
    TerminateProcessError(#[source] nix::Error),
    #[error("Failed to parse LISTENER_FD")]
    ListenerFdParseError,
}

pub struct AgentProcessManager {
    uds_path: PathBuf,
    listener_fd: RawFd,
    #[allow(dead_code)]
    listener: Option<Arc<Mutex<UnixListener>>>,
}

impl AgentProcessManager {
    pub fn new(uds_path: PathBuf) -> Result<Self, &'static str> {
        let (listener_fd, listener) = Self::setup_listener(&uds_path).map_err(|e| {
            log::error!("Error initializing listener: {}", e);
            "Failed to initialize listener"
        })?;

        Ok(AgentProcessManager {
            uds_path,
            listener_fd,
            listener,
        })
    }

    pub fn launch_agent(&self) -> Result<ProcessInfo, AgentProcessManagerError> {
        let current_exe =
            env::current_exe().map_err(AgentProcessManagerError::CurrentExecutablePathError)?;

        let child = Command::new(current_exe)
            .env("LISTENER_FD", self.listener_fd.to_string())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(AgentProcessManagerError::ForkAgentError)?;

        let process_info = ProcessInfo::new(child.id(), FeatType::Agent);

        Ok(process_info)
    }

    pub fn terminate_agent(&self, process_id: u32) -> Result<(), AgentProcessManagerError> {
        log::info!("Terminating agent with PID: {}", process_id);

        signal::kill(Pid::from_raw(process_id as i32), Signal::SIGTERM)
            .map_err(AgentProcessManagerError::TerminateProcessError)?;

        Ok(())
    }

    fn setup_listener(
        uds_path: &PathBuf,
    ) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentProcessManagerError> {
        if is_manage_by_systemd() && is_manage_socket_activation() {
            Self::get_fd_from_systemd()
        } else if let Ok(listener_fd_str) = env::var("LISTENER_FD") {
            Self::duplicate_fd(listener_fd_str)
        } else {
            Self::bind_new_uds(uds_path)
        }
    }

    fn get_fd_from_systemd(
    ) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentProcessManagerError> {
        let listen_fds = env::var("LISTEN_FDS")
            .map_err(|_| AgentProcessManagerError::ListenFdsError)?
            .parse::<i32>()
            .map_err(|_| AgentProcessManagerError::ListenFdsError)?;

        let listen_pid = env::var("LISTEN_PID")
            .map_err(|_| AgentProcessManagerError::ListenPidError)?
            .parse::<i32>()
            .map_err(|_| AgentProcessManagerError::ListenPidError)?;

        let current_pid = std::process::id() as i32;
        if listen_pid != current_pid {
            return Err(AgentProcessManagerError::ListenPidMismatch {
                listen_pid,
                current_pid,
            });
        } else if listen_fds <= 0 {
            return Err(AgentProcessManagerError::NoFileDescriptors);
        }

        Ok((DEFAULT_FD, None))
    }

    fn duplicate_fd(
        listener_fd_str: String,
    ) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentProcessManagerError> {
        let listener_fd: RawFd = listener_fd_str
            .parse::<i32>()
            .map_err(|_| AgentProcessManagerError::ListenerFdParseError)?;

        let duplicated_fd = dup(listener_fd).map_err(AgentProcessManagerError::DuplicateFdError)?;
        let listener: UnixListener = unsafe { UnixListener::from_raw_fd(duplicated_fd) };

        Ok((duplicated_fd, Some(Arc::new(Mutex::new(listener)))))
    }

    fn bind_new_uds(
        uds_path: &PathBuf,
    ) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentProcessManagerError> {
        if uds_path.exists() {
            log::warn!("UDS file already exists, removing: {:?}", uds_path);
            std::fs::remove_file(uds_path).map_err(AgentProcessManagerError::BindUdsError)?;
        }

        let listener =
            UnixListener::bind(uds_path).map_err(AgentProcessManagerError::BindUdsError)?;
        let listener_fd =
            dup(listener.as_raw_fd()).map_err(AgentProcessManagerError::DuplicateFdError)?;

        Ok((listener_fd, Some(Arc::new(Mutex::new(listener)))))
    }

    pub fn cleanup_uds_file(&self) -> Result<(), std::io::Error> {
        if self.uds_path.exists() {
            log::warn!("Removing UDS file: {:?}", self.uds_path);
            std::fs::remove_file(&self.uds_path)?;
            log::info!("UDS file removed successfully.");
        } else {
            log::info!("No UDS file to remove at {:?}", self.uds_path);
        }
        Ok(())
    }
}
