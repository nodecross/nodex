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
pub struct RuntimeManager<H, P>
where
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    pub self_pid: u32,
    file_handler: H,
    process_manager: P,
    uds_path: PathBuf,
    meta_uds_path: PathBuf,
    state_sender: watch::Sender<State>,
}

impl<H, P> RuntimeManager<H, P>
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
        let meta_uds_path = crate::unix_utils::convention_of_meta_uds_path(&uds_path)
            .map_err(|_| RuntimeError::PathConvention)?;
        let self_pid = std::process::id();
        let mut runtime_manager = RuntimeManager {
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
        // dummy
        let (state_sender, _) = watch::channel(State::Init);
        let self_pid = std::process::id();
        RuntimeManager {
            self_pid,
            file_handler,
            state_sender,
            process_manager,
            uds_path: "".into(),
            meta_uds_path: "".into(),
        }
    }

    pub fn launch_agent(&mut self, is_first: bool) -> Result<ProcessInfo, RuntimeError> {
        if cfg!(unix) && is_first {
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

        if cfg!(unix) && is_first {
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

    pub fn is_agent_running(&mut self) -> Result<bool, RuntimeError> {
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

    pub fn get_exec_path(&mut self) -> Result<PathBuf, RuntimeError> {
        let runtime_info = self.file_handler.read()?;
        Ok(runtime_info.exec_path)
    }

    fn add_process_info(&mut self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
        self.file_handler
            .apply_with_lock(|runtime_info| runtime_info.add_process_info(process_info))
    }

    fn remove_process_info(&mut self, process_id: u32) -> Result<(), RuntimeError> {
        self.file_handler
            .apply_with_lock(|runtime_info| runtime_info.remove_process_info(process_id))
    }

    pub fn kill_process(&mut self, process_info: &ProcessInfo) -> Result<(), RuntimeError> {
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

    pub fn update_state_without_send(&mut self, state: State) -> Result<(), RuntimeError> {
        self.file_handler.apply_with_lock(|runtime_info| {
            runtime_info.state = state;
            Ok(())
        })
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

    pub fn kill_other_agents(&mut self, target: u32) -> Result<(), RuntimeError> {
        self.kill_others(target, Some(FeatType::Agent))
    }

    fn kill_others(&mut self, target: u32, feat_type: Option<FeatType>) -> Result<(), RuntimeError> {
        let (_oks, errs): (Vec<_>, Vec<_>) = self
            .file_handler
            .read()?
            .process_infos
            .into_iter()
            .flatten()
            .filter(|process_info| process_info.process_id != target)
            .filter(|p| feat_type.as_ref().map(|f| p.feat_type == *f).unwrap_or(true))
            .map(|process_info| self.kill_process(&process_info))
            .partition(Result::is_ok);
        if errs.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::Kills(
                errs.into_iter().map(Result::unwrap_err).collect(),
            ))
        }
    }

    pub fn launch_controller(
        &mut self,
        new_controller_path: impl AsRef<Path>,
    ) -> Result<(), RuntimeError> {
        self.kill_others(self.self_pid, None)?;
        if is_manage_by_systemd() && is_manage_socket_activation() {
            return Ok(());
        }
        // TODO: care about windows
        if cfg!(unix) {
            crate::unix_utils::change_to_executable(new_controller_path.as_ref())
                .map_err(RuntimeError::Command)?;
        }
        let child = self
            .process_manager
            .spawn_process(new_controller_path, &["controller"])
            .map_err(RuntimeError::Fork)?;
        log::info!("Parent process launched child with PID: {}", child);
        Ok(())
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
                "Failed to add process_info",
            )))
        }
    }
}
