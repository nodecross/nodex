use chrono::{DateTime, FixedOffset, Utc};
use nix::unistd::dup;
use std::env;
use std::fs;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::process::systemd::{check_manage_by_systemd, check_manage_socket_action};

static DEFAULT_FD: RawFd = 3;

pub struct AgentProcessManager {
    pub pid: u32,
    pub version: String,
    pub executed_at: DateTime<FixedOffset>,
    listener: Option<Arc<Mutex<UnixListener>>>,
}

impl AgentProcessManager {
    pub fn new(uds_path: &PathBuf) -> Result<Self, &'static str> {
        let (pid, listener) = Self::launch_agent(uds_path)?;
        let version = env!("CARGO_PKG_VERSION").to_string();
        let executed_at = Utc::now().with_timezone(&FixedOffset::east(9 * 3600));
        Ok(AgentProcessManager { pid, version, executed_at, listener })
    }

    fn launch_agent(
        uds_path: &PathBuf,
    ) -> Result<(u32, Option<Arc<Mutex<UnixListener>>>), &'static str> {
        let (listener_fd, listener) = Self::get_fd(uds_path).map_err(|e| {
            eprintln!("Error getting file descriptor: {}", e);
            "Failed to get file descriptor"
        })?;

        let current_exe =
            env::current_exe().map_err(|_| "Failed to get current executable path")?;
        let child = Command::new(current_exe)
            .env("LISTENER_FD", listener_fd.to_string())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|_| "Failed to fork agent")?;

        Ok((child.id(), listener))
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
            if Path::new(&uds_path).exists() {
                fs::remove_file(&uds_path)
                    .map_err(|e| format!("Failed to remove existing UDS file: {}", e))?;
            }

            let listener =
                UnixListener::bind(&uds_path).map_err(|e| format!("Failed to bind UDS: {}", e))?;

            let listener_fd = dup(listener.as_raw_fd())
                .map_err(|_| "Failed to duplicate file descriptor".to_string())?;

            Ok((listener_fd, Some(Arc::new(Mutex::new(listener)))))
        }
    }
}
