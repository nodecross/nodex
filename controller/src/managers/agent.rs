use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, Response};
use hyper_util::client::legacy::{Client, Error as LegacyClientError};
use serde::de::DeserializeOwned;
use std::{
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[cfg(unix)]
mod unix_imports {
    pub use hyperlocal::{UnixClientExt, UnixConnector, Uri};
    pub use nix::{
        sys::signal::{self, Signal},
        unistd::{dup, execvp, fork, setsid, ForkResult, Pid},
    };
    pub use std::ffi::CString;
    pub use std::os::unix::{
        io::{AsRawFd, FromRawFd, RawFd},
        net::UnixListener,
    };
}

#[cfg(unix)]
use unix_imports::*;

use crate::managers::runtime::{FeatType, ProcessInfo};
use crate::validator::process::{is_manage_by_systemd, is_manage_socket_activation};

#[cfg(unix)]
static DEFAULT_FD: RawFd = 3;

#[derive(Debug, thiserror::Error)]
pub enum AgentManagerError {
    #[error["Failed to initialize listener"]]
    FailedInitialize,
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
    #[cfg(unix)]
    #[error("Failed to duplicate file descriptor")]
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
}

#[async_trait]
pub trait AgentManagerTrait: Send {
    fn launch_agent(&self) -> Result<ProcessInfo, AgentManagerError>;

    fn terminate_agent(&self, process_id: u32) -> Result<(), AgentManagerError>;

    async fn get_request<T>(&self, endpoint: &str) -> Result<T, AgentManagerError>
    where
        T: serde::de::DeserializeOwned + Send;

    async fn parse_response_body<T>(
        &self,
        response: Response<Incoming>,
    ) -> Result<T, AgentManagerError>
    where
        T: DeserializeOwned;

    fn cleanup(&self) -> Result<(), std::io::Error>;
}

#[cfg(unix)]
pub struct UnixAgentManager {
    uds_path: PathBuf,
    listener_fd: RawFd,
    #[allow(dead_code)]
    listener: Option<Arc<Mutex<UnixListener>>>,
}

#[cfg(unix)]
#[async_trait]
impl AgentManagerTrait for UnixAgentManager {
    fn launch_agent(&self) -> Result<ProcessInfo, AgentManagerError> {
        let current_exe =
            env::current_exe().map_err(AgentManagerError::CurrentExecutablePathError)?;

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
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

                let cmd = CString::new(current_exe.to_string_lossy().as_ref()).map_err(|e| {
                    AgentManagerError::ForkAgentError(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        e,
                    ))
                })?;
                let args = vec![cmd.clone()];

                std::env::set_var("LISTENER_FD", self.listener_fd.to_string());

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

    fn terminate_agent(&self, process_id: u32) -> Result<(), AgentManagerError> {
        log::info!("Terminating agent with PID: {}", process_id);

        signal::kill(Pid::from_raw(process_id as i32), Signal::SIGTERM)
            .map_err(AgentManagerError::TerminateProcessError)?;

        Ok(())
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

    fn cleanup(&self) -> Result<(), std::io::Error> {
        if self.uds_path.exists() {
            std::fs::remove_file(&self.uds_path)?;
        }
        Ok(())
    }
}

#[cfg(unix)]
impl UnixAgentManager {
    pub fn new(uds_path: PathBuf) -> Result<Self, AgentManagerError> {
        // Setup UDS listener logic
        let (listener_fd, listener) = Self::setup_listener(&uds_path).map_err(|e| {
            log::error!("Error initializing listener: {}", e);
            AgentManagerError::FailedInitialize
        })?;

        Ok(UnixAgentManager {
            uds_path,
            listener_fd,
            listener,
        })
    }

    fn setup_listener(
        uds_path: &PathBuf,
    ) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentManagerError> {
        if is_manage_by_systemd() && is_manage_socket_activation() {
            Self::get_fd_from_systemd()
        } else if let Ok(listener_fd_str) = env::var("LISTENER_FD") {
            Self::duplicate_fd(listener_fd_str)
        } else {
            Self::bind_new_uds(uds_path)
        }
    }

    fn get_fd_from_systemd() -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentManagerError>
    {
        let listen_fds = env::var("LISTEN_FDS")
            .map_err(|_| AgentManagerError::ListenFdsError)?
            .parse::<i32>()
            .map_err(|_| AgentManagerError::ListenFdsError)?;

        let listen_pid = env::var("LISTEN_PID")
            .map_err(|_| AgentManagerError::ListenPidError)?
            .parse::<i32>()
            .map_err(|_| AgentManagerError::ListenPidError)?;

        let current_pid = std::process::id() as i32;
        if listen_pid != current_pid {
            return Err(AgentManagerError::ListenPidMismatch {
                listen_pid,
                current_pid,
            });
        } else if listen_fds <= 0 {
            return Err(AgentManagerError::NoFileDescriptors);
        }

        Ok((DEFAULT_FD, None))
    }

    fn duplicate_fd(
        listener_fd_str: String,
    ) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentManagerError> {
        let listener_fd: RawFd = listener_fd_str
            .parse::<i32>()
            .map_err(|_| AgentManagerError::ListenerFdParseError)?;

        let duplicated_fd = dup(listener_fd).map_err(AgentManagerError::DuplicateFdError)?;
        let listener: UnixListener = unsafe { UnixListener::from_raw_fd(duplicated_fd) };

        Ok((duplicated_fd, Some(Arc::new(Mutex::new(listener)))))
    }

    fn bind_new_uds(
        uds_path: &PathBuf,
    ) -> Result<(RawFd, Option<Arc<Mutex<UnixListener>>>), AgentManagerError> {
        if uds_path.exists() {
            log::warn!("UDS file already exists, removing: {:?}", uds_path);
            std::fs::remove_file(uds_path).map_err(AgentManagerError::BindUdsError)?;
        }

        let listener = UnixListener::bind(uds_path).map_err(AgentManagerError::BindUdsError)?;
        let listener_fd = dup(listener.as_raw_fd()).map_err(AgentManagerError::DuplicateFdError)?;

        Ok((listener_fd, Some(Arc::new(Mutex::new(listener)))))
    }
}

unsafe impl Sync for UnixAgentManager {}

#[cfg(windows)]
pub struct WindowsAgentManager;

#[cfg(windows)]
#[async_trait]
impl AgentManagerTrait for WindowsAgentManager {
    pub fn new(uds_path: PathBuf) -> Result<Self, AgentManagerError> {
        unimplemented!()
    }

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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::tempdir;
//     use std::sync::Arc;
//     use std::sync::Mutex;

//     #[test]
//     fn test_new_agent_manager() {
//         let temp_dir = tempdir().expect("Failed to create temporary directory");
//         let uds_path = temp_dir.path().join("test_agent_manager.sock");

//         let agent_manager = AgentManager::new(uds_path.clone());
//         assert!(agent_manager.is_ok(), "Failed to create AgentManager");
//         let agent_manager = agent_manager.unwrap();

//         #[cfg(unix)]
//         assert_eq!(agent_manager.uds_path, uds_path, "UDS path mismatch");
//     }

//     #[cfg(unix)]
//     #[test]
//     fn test_bind_new_uds() {
//         let temp_dir = tempdir().expect("Failed to create temporary directory");
//         let uds_path = temp_dir.path().join("test_bind_new_uds.sock");

//         let result = AgentManager::bind_new_uds(&uds_path);
//         assert!(result.is_ok(), "Failed to bind new UDS");
//         let (listener_fd, listener) = result.unwrap();

//         assert!(uds_path.exists(), "UDS file was not created");
//         assert!(listener.is_some(), "Listener should be initialized");
//         assert!(listener_fd >= 0, "Listener FD should be valid");

//         // Cleanup
//         std::fs::remove_file(&uds_path).unwrap();
//     }

//     #[cfg(unix)]
//     #[test]
//     fn test_cleanup_uds_file() {
//         let temp_dir = tempdir().expect("Failed to create temporary directory");
//         let uds_path = temp_dir.path().join("test_cleanup_uds_file.sock");

//         // Create a dummy UDS file
//         let _listener = UnixListener::bind(&uds_path).expect("Failed to bind UDS");

//         let agent_manager = AgentManager::new(uds_path.clone()).unwrap();
//         assert!(uds_path.exists(), "UDS file should exist before cleanup");

//         agent_manager.cleanup_uds_file().expect("Failed to cleanup UDS file");
//         assert!(
//             !uds_path.exists(),
//             "UDS file should not exist after cleanup"
//         );
//     }

//     #[cfg(unix)]
//     #[test]
//     fn test_launch_agent() {
//         let temp_dir = tempdir().expect("Failed to create temporary directory");
//         let uds_path = temp_dir.path().join("test_launch_agent.sock");

//         let agent_manager = AgentManager::new(uds_path).unwrap();

//         // Fork and execute a dummy agent process
//         let process_info = agent_manager.launch_agent();
//         assert!(process_info.is_ok(), "Failed to launch agent");
//         let process_info = process_info.unwrap();

//         assert!(process_info.process_id > 0, "Process ID should be valid");
//         assert_eq!(process_info.feat_type, FeatType::Agent, "Feature type mismatch");

//         // Terminate the process
//         agent_manager
//             .terminate_agent(process_info.process_id)
//             .expect("Failed to terminate agent");
//     }

//     #[cfg(unix)]
//     #[tokio::test]
//     async fn test_get_request() {
//         let temp_dir = tempdir().expect("Failed to create temporary directory");
//         let uds_path = temp_dir.path().join("test_get_request.sock");

//         // Start a mock UDS server
//         let listener = UnixListener::bind(&uds_path).expect("Failed to bind UDS");
//         let listener = Arc::new(Mutex::new(listener));

//         tokio::spawn({
//             let listener = Arc::clone(&listener);
//             async move {
//                 let listener = listener.lock().unwrap();
//                 let (stream, _) = listener.accept().unwrap();

//                 let response = Response::new(Full::from(Bytes::from_static(b"{\"status\":\"ok\"}")));
//                 hyper::server::conn::http1::Builder::new()
//                     .serve_connection(stream, hyper::service::service_fn(|_| async {
//                         Ok::<_, hyper::Error>(response)
//                     }))
//                     .await
//                     .unwrap();
//             }
//         });

//         // Create AgentManager and make a request
//         let agent_manager = AgentManager::new(uds_path.clone()).unwrap();
//         let response: serde_json::Value = agent_manager
//             .get_request("/status")
//             .await
//             .expect("Failed to send request");

//         assert_eq!(
//             response["status"], "ok",
//             "Unexpected response: {:?}",
//             response
//         );

//         // Cleanup
//         agent_manager.cleanup_uds_file().unwrap();
//     }
// }
