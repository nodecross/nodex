use super::runtime::{RuntimeError, RuntimeInfoStorage, RuntimeManager};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, Response};
use hyper_util::client::legacy::{Client, Error as LegacyClientError};
use notify::event::{AccessKind, AccessMode, CreateKind};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(unix)]
mod unix_imports {
    pub use hyperlocal::{UnixClientExt, UnixConnector, Uri};
    pub use nix::{
        sys::signal::{self, Signal},
        sys::socket::{recvmsg, sendmsg, ControlMessage, ControlMessageOwned, MsgFlags},
        unistd::{execvp, fork, setsid, ForkResult, Pid},
    };
    pub use std::ffi::CString;
    pub use std::io::{IoSlice, IoSliceMut};
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
    #[error["Failed to initialize listener"]]
    FailedInitialize,
    #[error("Failed to get current executable path: {0}")]
    CurrentExecutablePathError(#[source] std::io::Error),
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
    #[error("Failed to bind UDS: {0}")]
    BindUdsError(#[source] std::io::Error),
    #[error("Failed to watch UDS: {0}")]
    WatchUdsError(#[source] notify::Error),
    #[cfg(unix)]
    #[error("Failed to duplicate file descriptor: {0}")]
    DuplicateFdError(#[source] nix::Error),
    #[cfg(unix)]
    #[error("Failed to terminate process: {0}")]
    TerminateProcessError(#[source] nix::Error),
    #[error("Failed to parse LISTENER_FD")]
    ListenerFdParseError,
    #[error("Request failed: {0}")]
    RequestFailed(#[from] LegacyClientError),
    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(#[source] serde_json::Error),
    #[error("Failed to collect body: {0}")]
    CollectBodyError(String),
    #[error("Failed to convert body to string: {0}")]
    Utf8Error(#[source] std::str::Utf8Error),
    #[error("Failed to use runtime: {0}")]
    Runtime(#[from] RuntimeError),
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
pub fn send_fd(tx: RawFd, fd: Option<RawFd>) -> nix::Result<()> {
    match fd {
        Some(fd) => {
            let iov = [IoSlice::new(&[0u8; 1])];
            let fds = [fd];
            let cmsg = ControlMessage::ScmRights(&fds);
            sendmsg::<()>(tx, &iov, &[cmsg], MsgFlags::empty(), None)?;
        }
        None => {
            let iov = [IoSlice::new(&[1u8; 1])];
            let fds = [];
            let cmsg = ControlMessage::ScmRights(&fds);
            sendmsg::<()>(tx, &iov, &[cmsg], MsgFlags::empty(), None)?;
        }
    };
    Ok(())
}

#[cfg(unix)]
pub fn recv_fd(socket: RawFd) -> nix::Result<Option<RawFd>> {
    let mut buf = [0u8; 1];
    let mut iov = [IoSliceMut::new(&mut buf)];
    let mut space = nix::cmsg_space!([RawFd; 1]);
    let msg = recvmsg::<()>(socket, &mut iov, Some(&mut space), MsgFlags::empty())?;
    let buf = msg.iovs().next().ok_or(nix::errno::Errno::ENOENT)?;
    let is_some = !buf.is_empty() && buf[0] == 1;
    if is_some {
        return Ok(None);
    } else {
        let cmsg = msg.cmsgs()?.next().ok_or(nix::errno::Errno::ENOENT)?;
        if let ControlMessageOwned::ScmRights(fds) = cmsg {
            if !fds.is_empty() {
                return Ok(Some(fds[0]));
            }
        }
    }
    Err(nix::Error::ENOENT)
}

pub fn wait_until_file_created(path: impl AsRef<Path>) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    let dir = path
        .as_ref()
        .parent()
        .ok_or(notify::Error::io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to get parent of watching path",
        )))?;
    watcher.watch(dir.as_ref(), RecursiveMode::NonRecursive)?;
    let path = path.as_ref().to_path_buf();
    if !path.exists() {
        for res in rx {
            match res? {
                Event {
                    kind: EventKind::Access(AccessKind::Close(AccessMode::Write)),
                    paths,
                    ..
                }
                | Event {
                    kind: EventKind::Create(CreateKind::File),
                    paths,
                    ..
                } if paths.contains(&path) => return Ok(()),
                _ => continue,
            }
        }
    }
    Ok(())
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
        let runtime_info = self.runtime_manager.read_runtime_info()?;
        let current_exe = &runtime_info.exec_path;
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
                    let () = wait_until_file_created(&self.meta_uds_path)
                        .map_err(AgentManagerError::WatchUdsError)?;
                    let stream = std::os::unix::net::UnixStream::connect(&self.meta_uds_path)
                        .map_err(AgentManagerError::BindUdsError)?;
                    send_fd(stream.as_raw_fd(), listener)
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
        let version_response: VersionResponse = self.get_request("/internal/version/get").await?;
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

    async fn parse_response_body<T>(
        &self,
        response: Response<Incoming>,
    ) -> Result<T, AgentManagerError>
    where
        T: DeserializeOwned,
    {
        let collected_body = response
            .into_body()
            .collect()
            .await
            .map_err(|e| AgentManagerError::CollectBodyError(e.to_string()))?;

        let bytes = collected_body.to_bytes();
        let string_body =
            std::str::from_utf8(bytes.as_ref()).map_err(AgentManagerError::Utf8Error)?;

        serde_json::from_str(string_body).map_err(AgentManagerError::JsonParseError)
    }

    async fn get_request<T>(&self, endpoint: &str) -> Result<T, AgentManagerError>
    where
        T: serde::de::DeserializeOwned + Send,
    {
        let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
        let uri = Uri::new(&self.uds_path, endpoint).into();

        let response: Response<Incoming> = client.get(uri).await?;

        self.parse_response_body(response).await
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
