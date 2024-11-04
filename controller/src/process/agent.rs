use chrono::{FixedOffset, Utc};
use log;
use nix::sys::signal::{self, Signal};
use nix::unistd::dup;
use nix::unistd::Pid;
use std::env;
use std::fs;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::process::systemd::{check_manage_by_systemd, check_manage_socket_activation};
use crate::runtime::AgentInfo;

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
    #[error("Failed to remove existing UDS file: {0}")]
    RemoveUdsFileError(#[source] std::io::Error),
    #[error("Failed to bind UDS: {0}")]
    BindUdsError(#[source] std::io::Error),
    #[error("Failed to duplicate file descriptor")]
    DuplicateFdError(#[source] nix::Error),
    #[error("Failed to terminate process: {0}")]
    TerminateProcessError(#[source] nix::Error),
}

pub trait AgentEventListener {
    fn on_agent_started(&mut self, agent_info: AgentInfo);
    fn on_agent_terminated(&mut self, process_id: u32);
}

pub struct AgentProcessManager {
    listener_fd: RawFd,
    listener: Option<Arc<Mutex<UnixListener>>>,
    event_listener: Arc<Mutex<dyn AgentEventListener + Send>>,
}

impl AgentProcessManager {
    pub fn new(
        uds_path: &PathBuf,
        event_listener: Arc<Mutex<dyn AgentEventListener + Send>>,
    ) -> Result<Self, &'static str> {
        let (listener_fd, listener) = Self::initialize_listener_fd(uds_path).map_err(|e| {
            log::error!("Error getting file descriptor: {}", e);
            "Failed to get file descriptor"
        })?;

        Ok(AgentProcessManager {
            listener_fd,
            listener,
            event_listener,
        })
    }

    pub fn launch_agent(&self) -> Result<(), AgentProcessManagerError> {
        let current_exe =
            env::current_exe().map_err(AgentProcessManagerError::CurrentExecutablePathError)?;

        let child = Command::new(current_exe)
            .env("LISTENER_FD", self.listener_fd.to_string())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(AgentProcessManagerError::ForkAgentError)?;

        let version = env!("CARGO_PKG_VERSION").to_string();
        let executed_at = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());

        let agent_info = AgentInfo {
            process_id: child.id(),
            executed_at,
            version,
        };
        self.event_listener
            .lock()
            .unwrap()
            .on_agent_started(agent_info);

        Ok(())
    }

    fn initialize_listener_fd(
        uds_path: &PathBuf,
    ) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentProcessManagerError> {
        if check_manage_by_systemd() && check_manage_socket_activation() {
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
        } else {
            if uds_path.exists() {
                let listener = UnixListener::bind(uds_path).map_err(AgentProcessManagerError::BindUdsError)?;
                let listener_fd = dup(listener.as_raw_fd()).map_err(AgentProcessManagerError::DuplicateFdError)?;
        
                Ok((listener_fd, Some(Arc::new(Mutex::new(listener)))))
            } else {
                let listener = UnixListener::bind(uds_path).map_err(AgentProcessManagerError::BindUdsError)?;
                let listener_fd = dup(listener.as_raw_fd()).map_err(AgentProcessManagerError::DuplicateFdError)?;
        
                Ok((listener_fd, Some(Arc::new(Mutex::new(listener)))))
            }
        }
    }

    pub fn terminate_agent(&self, process_id: u32) -> Result<(), AgentProcessManagerError> {
        log::info!("Terminating agent with PID: {}", process_id);

        signal::kill(Pid::from_raw(process_id as i32), Signal::SIGTERM)
            .map_err(AgentProcessManagerError::TerminateProcessError)?;

        self.event_listener
            .lock()
            .unwrap()
            .on_agent_terminated(process_id);
        Ok(())
    }
}
