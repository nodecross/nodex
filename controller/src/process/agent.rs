use chrono::{DateTime, FixedOffset, Utc};
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

use crate::process::systemd::{check_manage_by_systemd, check_manage_socket_action};
use crate::runtime::AgentInfo;

static DEFAULT_FD: RawFd = 3;

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
        let (listener_fd, listener) = Self::get_fd(uds_path).map_err(|e| {
            log::error!("Error getting file descriptor: {}", e);
            "Failed to get file descriptor"
        })?;

        Ok(AgentProcessManager {
            listener_fd,
            listener,
            event_listener,
        })
    }

    pub fn launch_agent(&self) -> Result<(), &'static str> {
        let current_exe =
            env::current_exe().map_err(|_| "Failed to get current executable path")?;
        let child = Command::new(current_exe)
            .env("LISTENER_FD", self.listener_fd.to_string())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|_| "Failed to fork agent")?;

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

    fn get_fd(uds_path: &PathBuf) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), String> {
        if check_manage_by_systemd() && check_manage_socket_action() {
            let listen_fds = env::var("LISTEN_FDS")
                .map_err(|_| "LISTEN_FDS not set".to_string())?
                .parse::<i32>()
                .map_err(|_| "Invalid LISTEN_FDS".to_string())?;

            let listen_pid = env::var("LISTEN_PID")
                .map_err(|_| "LISTEN_PID not set".to_string())?
                .parse::<i32>()
                .map_err(|_| "Invalid LISTEN_PID".to_string())?;

            let current_pid = std::process::id() as i32;
            if listen_pid != current_pid {
                return Err(format!(
                    "LISTEN_PID ({}) does not match current process ID ({})",
                    listen_pid, current_pid
                ));
            } else if listen_fds <= 0 {
                return Err("No file descriptors passed by systemd.".to_string());
            }

            Ok((DEFAULT_FD, None))
        } else {
            if Path::new(uds_path).exists() {
                fs::remove_file(uds_path)
                    .map_err(|e| format!("Failed to remove existing UDS file: {}", e))?;
            }

            let listener =
                UnixListener::bind(uds_path).map_err(|e| format!("Failed to bind UDS: {}", e))?;
            let listener_fd = dup(listener.as_raw_fd())
                .map_err(|_| "Failed to duplicate file descriptor".to_string())?;

            Ok((listener_fd, Some(Arc::new(Mutex::new(listener)))))
        }
    }

    pub fn terminate_agent(&self, process_id: u32) -> Result<(), String> {
        log::info!("Terminating agent with PID: {}", process_id);

        signal::kill(Pid::from_raw(process_id as i32), Signal::SIGTERM)
            .map_err(|e| format!("Failed to terminate process: {}", e))?;

        self.event_listener
            .lock()
            .unwrap()
            .on_agent_terminated(process_id);
        Ok(())
    }
}
