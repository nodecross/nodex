use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, Response};
use hyper_util::client::legacy::Client;
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
    RequestFailed(String),
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

        let response: Response<Incoming> = client.get(uri).await.map_err(|e| {
            log::error!("Request failed: {}", e);
            AgentManagerError::RequestFailed(e.to_string())
        })?;

        self.parse_response_body(response).await
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

#[cfg(unix)]
unsafe impl Sync for UnixAgentManager {}

#[cfg(windows)]
pub struct WindowsAgentManager;

#[cfg(windows)]
#[async_trait]
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

    fn cleanup(&self) -> Result<(), std::io::Error> {
        unimplemented!()
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use libc;
    use std::env;
    use std::path::Path;

    #[test]
    fn test_unix_agent_manager_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let uds_path = temp_dir.path().join("test_socket");

        let manager = UnixAgentManager::new(uds_path.clone());
        assert!(
            manager.is_ok(),
            "UnixAgentManager should be initialized successfully"
        );
        let manager = manager.unwrap();

        assert_eq!(manager.uds_path, uds_path);
    }

    #[test]
    fn test_bind_new_uds() {
        let temp_dir = tempfile::tempdir().unwrap();
        let uds_path = temp_dir.path().join("test_socket");

        let result = UnixAgentManager::bind_new_uds(&uds_path);
        assert!(result.is_ok(), "UDS binding should succeed");
        let (listener_fd, listener) = result.unwrap();

        assert!(uds_path.exists(), "UDS file should be created");
        assert!(listener.is_some(), "Listener should be created");
        unsafe {
            libc::close(listener_fd);
        }
    }

    #[test]
    fn test_setup_listener_with_systemd_activation() {
        env::set_var("LISTEN_FDS", "1");
        env::set_var("LISTEN_PID", std::process::id().to_string());

        let result = UnixAgentManager::get_fd_from_systemd();
        assert!(result.is_ok(), "Systemd socket activation should succeed");
        let (listener_fd, listener) = result.unwrap();

        assert_eq!(
            listener_fd, DEFAULT_FD,
            "Listener FD should match DEFAULT_FD"
        );
        assert!(
            listener.is_none(),
            "Listener should not be created in this mode"
        );
    }

    #[test]
    fn test_duplicate_fd() {
        let temp_dir = tempfile::tempdir().unwrap();
        let uds_path = temp_dir.path().join("test_socket");
        let listener = UnixListener::bind(&uds_path).unwrap();

        let listener_fd = listener.as_raw_fd();
        let listener_fd_str = listener_fd.to_string();

        let result = UnixAgentManager::duplicate_fd(listener_fd_str);
        assert!(result.is_ok(), "Duplicating FD should succeed");
        let (duplicated_fd, listener) = result.unwrap();

        assert!(listener.is_some(), "Listener should be created");
    }

    #[tokio::test]
    async fn test_launch_and_terminate_agent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let uds_path = temp_dir.path().join("test_socket");

        let manager = UnixAgentManager::new(uds_path).unwrap();
        let process_info = manager.launch_agent();
        assert!(process_info.is_ok(), "Agent launch should succeed");

        let process_info = process_info.unwrap();
        assert!(
            manager.terminate_agent(process_info.process_id).is_ok(),
            "Agent termination should succeed"
        );
    }

    // #[tokio::test]
    // async fn test_get_request() {
    //     let temp_dir = tempfile::tempdir().unwrap();
    //     let uds_path = temp_dir.path().join("test_socket");
    //     let manager = UnixAgentManager::new(uds_path).unwrap();

    //     let response: Result<String, AgentManagerError> = manager.get_request("/mock_endpoint").await;
    //     assert!(response.is_err(), "Request should fail because no server is running");
    // }

    // #[tokio::test]
    // async fn test_get_request() {
    //     use serde::{Deserialize, Serialize};
    //     #[derive(Debug, Deserialize, Serialize)]
    //     pub struct Response {
    //         pub message: String,
    //     }

    //     let temp_dir = tempfile::tempdir().unwrap();
    //     let uds_path = temp_dir.path().join("test_socket");

    //     let listener = tokio::net::UnixListener::bind(&uds_path).unwrap();

    //     let server_task = tokio::spawn(async move {
    //         let (mut socket, _) = listener.accept().await.unwrap();

    //         use tokio::io::{AsyncReadExt, AsyncWriteExt};

    //         let mut request_buf = Vec::new();
    //         socket.read_to_end(&mut request_buf).await.unwrap();
    //         println!("Received request: {}", String::from_utf8_lossy(&request_buf));

    //         let json_body = r#"{"message":"Hello, JSON"}"#;
    //         let response = format!(
    //             "HTTP/1.1 200 OK\r\n\
    //             Content-Type: application/json\r\n\
    //             Content-Length: {}\r\n\
    //             Connection: close\r\n\
    //             \r\n\
    //             {}",
    //             json_body.len(),
    //             json_body
    //         );

    //         socket.write_all(response.as_bytes()).await.unwrap();

    //         socket.shutdown().await.unwrap();
    //     });

    //     let manager = UnixAgentManager::new(uds_path.clone()).unwrap();
    //     let response: Response = manager.get_request("/mock_endpoint").await.unwrap();

    //     server_task.await.unwrap();
    //     assert_eq!(response.message, "Hello, JSON");
    // }
}
