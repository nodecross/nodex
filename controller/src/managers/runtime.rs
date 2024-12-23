use crate::validator::process::{is_manage_by_systemd, is_manage_socket_activation};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::watch;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub state: State,
    pub process_infos: [Option<ProcessInfo>; 4],
    pub exec_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum State {
    Init,
    Idle,
    Update,
    Rollback,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

pub enum NodexSignal {
    Terminate,
    SendFd,
}

pub trait ProcessManager: Clone {
    fn is_running(&self, process_id: u32) -> bool;
    fn spawn_process(&self, cmd: impl AsRef<Path>, args: &[&str]) -> Result<u32, std::io::Error>;
    fn kill_process(&self, process_id: u32, signal: NodexSignal) -> Result<(), std::io::Error>;
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
    #[error("Failed to kill process")]
    Kill(std::io::Error),
    #[error("Failed to kill processes")]
    Kills(Vec<RuntimeError>),
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
    #[cfg(unix)]
    #[error("Failed to get fd from systemd: {0}")]
    GetFd(#[from] crate::unix_utils::GetFdError),
    #[cfg(unix)]
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

pub trait RuntimeManagerWithoutAsync {
    fn launch_agent(&mut self, is_first: bool) -> Result<ProcessInfo, RuntimeError>;

    fn is_agent_running(&mut self) -> Result<bool, RuntimeError>;

    fn get_exec_path(&mut self) -> Result<PathBuf, RuntimeError>;

    fn kill_process(&mut self, process_info: &ProcessInfo) -> Result<(), RuntimeError>;

    fn update_state_without_send(&mut self, state: State) -> Result<(), RuntimeError>;

    fn update_state(&mut self, state: State) -> Result<(), RuntimeError>;

    fn kill_other_agents(&mut self, target: u32) -> Result<(), RuntimeError>;

    fn launch_controller(
        &mut self,
        new_controller_path: impl AsRef<Path>,
    ) -> Result<(), RuntimeError>;
}

#[trait_variant::make(Send)]
pub trait RuntimeManager: RuntimeManagerWithoutAsync {
    async fn get_version(&self) -> Result<String, RuntimeError>;
}

#[derive(Debug, Clone)]
pub struct RuntimeManagerImpl<H, P>
where
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    self_pid: u32,
    file_handler: H,
    process_manager: P,
    uds_path: PathBuf,
    meta_uds_path: PathBuf,
    state_sender: watch::Sender<State>,
}

impl<H, P> RuntimeManager for RuntimeManagerImpl<H, P>
where
    H: RuntimeInfoStorage + Sync + Send,
    P: ProcessManager + Sync + Send,
{
    async fn get_version(&self) -> Result<String, RuntimeError> {
        #[cfg(unix)]
        let version_response: VersionResponse =
            crate::unix_utils::get_request(&self.uds_path, "/internal/version/get").await?;
        #[cfg(windows)]
        let version_response = VersionResponse {
            version: "dummy".to_string(),
        };
        Ok(version_response.version)
    }
}

impl<H, P> RuntimeManagerWithoutAsync for RuntimeManagerImpl<H, P>
where
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    fn launch_agent(&mut self, is_first: bool) -> Result<ProcessInfo, RuntimeError> {
        #[cfg(unix)]
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
        let child = self
            .process_manager
            .spawn_process(current_exe, &[])
            .map_err(RuntimeError::Fork)?;

        #[cfg(unix)]
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
            let stream = std::os::unix::io::AsRawFd::as_raw_fd(&stream);
            crate::unix_utils::send_fd(stream, listener)
                .map_err(|e| RuntimeError::BindUdsError(e.into()))?;
        }
        let process_info = ProcessInfo::new(child, FeatType::Agent);
        self.add_process_info(process_info.clone())?;
        Ok(process_info)
    }

    fn is_agent_running(&mut self) -> Result<bool, RuntimeError> {
        let runtime_info = self.file_handler.read()?;
        let is_not_empty = runtime_info
            .process_infos
            .into_iter()
            .flatten()
            .filter(|process_info| process_info.feat_type == FeatType::Agent)
            .peekable()
            .peek()
            .is_some();
        Ok(is_not_empty)
    }

    fn get_exec_path(&mut self) -> Result<PathBuf, RuntimeError> {
        let runtime_info = self.file_handler.read()?;
        Ok(runtime_info.exec_path)
    }

    fn kill_process(&mut self, process_info: &ProcessInfo) -> Result<(), RuntimeError> {
        let signal = if process_info.feat_type == FeatType::Agent {
            NodexSignal::SendFd
        } else {
            NodexSignal::Terminate
        };
        self.process_manager
            .kill_process(process_info.process_id, signal)
            .map_err(RuntimeError::Kill)?;
        self.remove_process_info(process_info.process_id)?;
        Ok(())
    }

    fn update_state_without_send(&mut self, state: State) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.state = state;
            Ok(())
        })
    }

    fn update_state(&mut self, state: State) -> Result<(), RuntimeError> {
        self.update_state_without_send(state)?;
        let _ = self.state_sender.send(state);
        Ok(())
    }

    fn kill_other_agents(&mut self, target: u32) -> Result<(), RuntimeError> {
        self.kill_others(target, Some(FeatType::Agent))
    }

    fn launch_controller(
        &mut self,
        new_controller_path: impl AsRef<Path>,
    ) -> Result<(), RuntimeError> {
        self.kill_others(self.self_pid, None)?;
        if is_manage_by_systemd() && is_manage_socket_activation() {
            return Ok(());
        }
        // TODO: care about windows
        #[cfg(unix)]
        crate::unix_utils::change_to_executable(new_controller_path.as_ref())
            .map_err(RuntimeError::Command)?;
        let child = self
            .process_manager
            .spawn_process(new_controller_path, &["controller"])
            .map_err(RuntimeError::Fork)?;
        log::info!("Parent process launched child with PID: {}", child);
        Ok(())
    }
}

impl<H, P> RuntimeManagerImpl<H, P>
where
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    pub fn new_by_controller(
        mut file_handler: H,
        process_manager: P,
        uds_path: impl AsRef<Path>,
    ) -> Result<(Self, watch::Receiver<State>), RuntimeError> {
        let runtime_info = file_handler.read()?;
        let (state_sender, state_receiver) = watch::channel(runtime_info.state);
        #[cfg(unix)]
        let meta_uds_path = crate::unix_utils::convention_of_meta_uds_path(&uds_path)
            .map_err(|_| RuntimeError::PathConvention)?;
        #[cfg(windows)]
        let meta_uds_path = PathBuf::from("");
        let self_pid = std::process::id();
        let mut runtime_manager = RuntimeManagerImpl {
            self_pid,
            file_handler,
            state_sender,
            process_manager,
            uds_path: uds_path.as_ref().into(),
            meta_uds_path,
        };
        // We assume that caller is controller.
        runtime_manager.cleanup_process_info()?;
        let runtime_info = runtime_manager.file_handler.read()?;
        let controller_processes = runtime_info
            .process_infos
            .iter()
            .filter_map(|x| x.as_ref())
            .filter(|process_info| {
                process_info.feat_type == FeatType::Controller
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

    pub fn new_by_agent(file_handler: H, process_manager: P) -> Self {
        // We assume that caller is agent.
        // dummy channel
        let (state_sender, _) = watch::channel(State::Init);
        RuntimeManagerImpl {
            self_pid: std::process::id(),
            file_handler,
            state_sender,
            process_manager,
            uds_path: "".into(),
            meta_uds_path: "".into(),
        }
    }

    fn add_process_info(&mut self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
        self.file_handler
            .apply_with_lock(|runtime_info| runtime_info.add_process_info(process_info))
    }

    fn remove_process_info(&mut self, process_id: u32) -> Result<(), RuntimeError> {
        self.file_handler
            .apply_with_lock(|runtime_info| runtime_info.remove_process_info(process_id))
    }

    // Kill all related processes
    pub fn cleanup_all(&mut self) -> Result<(), RuntimeError> {
        #[cfg(unix)]
        {
            crate::unix_utils::remove_file_if_exists(&self.uds_path);
            crate::unix_utils::remove_file_if_exists(&self.meta_uds_path);
        }
        let process_manager = &self.process_manager;
        self.file_handler.apply_with_lock(move |runtime_info| {
            let mut errs = vec![];
            for info in runtime_info.process_infos.iter_mut() {
                if let Some(info) = info {
                    if let Err(err) =
                        process_manager.kill_process(info.process_id, NodexSignal::Terminate)
                    {
                        errs.push(RuntimeError::Kill(err));
                    }
                }
                *info = None;
            }
            runtime_info.state = State::Init;
            if errs.is_empty() {
                Ok(())
            } else {
                Err(RuntimeError::Kills(errs))
            }
        })
    }

    pub fn cleanup(&mut self) -> Result<(), RuntimeError> {
        self.remove_process_info(self.self_pid)
    }

    fn cleanup_process_info(&mut self) -> Result<(), RuntimeError> {
        let process_manager = &self.process_manager;
        self.file_handler.apply_with_lock(|runtime_info| {
            for process_info in runtime_info.process_infos.iter_mut() {
                if let Some(ref p) = process_info {
                    if !process_manager.is_running(p.process_id) {
                        *process_info = None;
                    }
                }
            }
            Ok(())
        })
    }

    fn kill_others(
        &mut self,
        target: u32,
        feat_type: Option<FeatType>,
    ) -> Result<(), RuntimeError> {
        let (_oks, errs): (Vec<_>, Vec<_>) = self
            .file_handler
            .read()?
            .process_infos
            .into_iter()
            .flatten()
            .filter(|process_info| process_info.process_id != target)
            .filter(|p| {
                feat_type
                    .as_ref()
                    .map(|f| p.feat_type == *f)
                    .unwrap_or(true)
            })
            .map(move |process_info| self.kill_process(&process_info))
            .partition(Result::is_ok);
        if errs.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::Kills(
                errs.into_iter().map(Result::unwrap_err).collect(),
            ))
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

impl RuntimeInfo {
    pub fn add_process_info(&mut self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
        for info in self.process_infos.iter_mut() {
            if info.is_none() {
                *info = Some(process_info);
                return Ok(());
            }
        }
        Err(RuntimeError::FileWrite(std::io::Error::new(
            std::io::ErrorKind::StorageFull,
            "Failed to add process_info",
        )))
    }
    pub fn remove_process_info(&mut self, process_id: u32) -> Result<(), RuntimeError> {
        let pid = process_id;
        let mut i = None;
        for (j, info) in self.process_infos.iter_mut().enumerate() {
            match info.as_ref() {
                Some(ProcessInfo { process_id, .. }) if pid == *process_id => {
                    *info = None;
                    i = Some(j);
                    break;
                }
                _ => continue,
            }
        }
        if let Some(i) = i {
            self.process_infos[i..].rotate_left(1);
            Ok(())
        } else {
            Err(RuntimeError::FileWrite(std::io::Error::new(
                std::io::ErrorKind::StorageFull,
                "Failed to remove process_info",
            )))
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    fn setup_temp_file() -> (RuntimeManager, tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_file_path = temp_dir.path().join("runtime_info.json");

        File::create(&temp_file_path).expect("Failed to create temporary runtime_info.json");

        assert!(
            temp_file_path.exists(),
            "Temporary file was not created: {:?}",
            temp_file_path
        );

        let file_handler = FileHandler::new(temp_file_path.clone());
        let runtime_manager = RuntimeManager::new(file_handler);

        (runtime_manager, temp_dir, temp_file_path)
    }

    #[test]
    fn test_read_write_runtime_info() {
        let (runtime_manager, _temp_dir, temp_file_path) = setup_temp_file();

        let initial_runtime_info = RuntimeInfo {
            state: State::Default,
            process_infos: vec![],
        };

        let file_handler = FileHandler::new(temp_file_path.clone());

        file_handler
            .write_locked(
                &mut File::create(&temp_file_path).unwrap(),
                &initial_runtime_info,
            )
            .unwrap();

        let read_runtime_info = runtime_manager.read_runtime_info().unwrap();
        assert_eq!(read_runtime_info.state, State::Default);
        assert_eq!(read_runtime_info.process_infos.len(), 0);
    }

    #[test]
    fn test_add_process_info() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        let process_info = ProcessInfo::new(12345, FeatType::Agent);
        runtime_manager
            .add_process_info(process_info.clone())
            .unwrap();

        let process_infos = runtime_manager.get_process_infos().unwrap();
        assert_eq!(process_infos.len(), 1);
        assert_eq!(process_infos[0].process_id, 12345);
        assert_eq!(process_infos[0].feat_type, FeatType::Agent);
    }

    #[test]
    fn test_remove_process_info() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        let process_info1 = ProcessInfo::new(12345, FeatType::Agent);
        let process_info2 = ProcessInfo::new(67890, FeatType::Controller);

        runtime_manager
            .add_process_info(process_info1.clone())
            .unwrap();
        runtime_manager
            .add_process_info(process_info2.clone())
            .unwrap();

        runtime_manager.remove_process_info(12345).unwrap();

        let process_infos = runtime_manager.get_process_infos().unwrap();
        assert_eq!(process_infos.len(), 1);
        assert_eq!(process_infos[0].process_id, 67890);
    }

    #[test]
    fn test_update_state() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        runtime_manager.update_state(State::Updating).unwrap();

        let state = runtime_manager.get_state().unwrap();
        assert_eq!(state, State::Updating);
    }

    #[test]
    fn test_filter_process_infos() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        let process_info1 = ProcessInfo::new(12345, FeatType::Agent);
        let process_info2 = ProcessInfo::new(67890, FeatType::Controller);

        runtime_manager
            .add_process_info(process_info1.clone())
            .unwrap();
        runtime_manager
            .add_process_info(process_info2.clone())
            .unwrap();

        let agents = runtime_manager
            .filter_process_infos(FeatType::Agent)
            .unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].process_id, 12345);

        let controllers = runtime_manager
            .filter_process_infos(FeatType::Controller)
            .unwrap();
        assert_eq!(controllers.len(), 1);
        assert_eq!(controllers[0].process_id, 67890);
    }

    #[test]
    fn test_is_running_or_remove_if_stopped() {
        let (runtime_manager, _temp_dir, _) = setup_temp_file();

        let process_info = ProcessInfo::new(12345, FeatType::Agent);

        runtime_manager
            .add_process_info(process_info.clone())
            .unwrap();

        let result = runtime_manager.is_running_or_remove_if_stopped(&process_info);
        assert!(!result);

        let process_infos = runtime_manager.get_process_infos().unwrap();
        assert!(process_infos.is_empty());
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
}
