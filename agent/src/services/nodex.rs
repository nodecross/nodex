use crate::nodex::extension::secure_keystore::FileBaseKeyStore;
use crate::nodex::keyring;
use crate::nodex::utils::sidetree_client::SideTreeClient;
use crate::{app_config, server_config};
use anyhow;

use protocol::did::did_repository::{DidRepository, DidRepositoryImpl};
use protocol::did::sidetree::payload::DidResolutionResponse;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(unix)]
mod unix_imports {
    pub use controller::managers::{
        resource::ResourceManager,
        runtime::{FeatType, FileHandler, RuntimeManager, State},
    };
    pub use controller::validator::{
        network::can_connect_to_download_server,
        process::{is_manage_by_systemd, is_manage_socket_activation},
        storage::check_storage,
    };
    pub use nix::{
        sys::signal::{self, Signal},
        unistd::{execvp, fork, setsid, ForkResult, Pid},
    };
    pub use std::ffi::CString;
}

#[cfg(unix)]
use unix_imports::*;

pub struct NodeX {
    did_repository: DidRepositoryImpl<SideTreeClient>,
}

impl NodeX {
    pub fn new() -> Self {
        let server_config = server_config();
        let sidetree_client = SideTreeClient::new(&server_config.did_http_endpoint()).unwrap();
        let did_repository = DidRepositoryImpl::new(sidetree_client);

        NodeX { did_repository }
    }

    pub fn did_repository(&self) -> &DidRepositoryImpl<SideTreeClient> {
        &self.did_repository
    }

    pub async fn create_identifier(&self) -> anyhow::Result<DidResolutionResponse> {
        // NOTE: find did
        let config = app_config();
        let keystore = FileBaseKeyStore::new(config.clone());
        if let Some(did) =
            keyring::keypair::KeyPairingWithConfig::load_keyring(config.clone(), keystore.clone())
                .ok()
                .and_then(|v| v.get_identifier().ok())
        {
            if let Some(json) = self.find_identifier(&did).await? {
                return Ok(json);
            }
        }

        let mut keyring_with_config =
            keyring::keypair::KeyPairingWithConfig::create_keyring(config, keystore);
        let res = self
            .did_repository
            .create_identifier(keyring_with_config.get_keyring())
            .await?;
        keyring_with_config.save(&res.did_document.id);

        Ok(res)
    }

    pub async fn find_identifier(
        &self,
        did: &str,
    ) -> anyhow::Result<Option<DidResolutionResponse>> {
        let res = self.did_repository.find_identifier(did).await?;

        Ok(res)
    }

    pub async fn update_version(
        &self,
        binary_url: &str,
        output_path: PathBuf,
    ) -> anyhow::Result<()> {
        if !check_storage(&output_path) {
            log::error!("Not enough storage space");
            return Err(anyhow::anyhow!("Not enough storage space"));
        } else if !can_connect_to_download_server("https://github.com").await {
            log::error!("Not connected to the Internet");
            return Err(anyhow::anyhow!("Not connected to the Internet"));
        } else if !binary_url.starts_with("https://github.com/nodecross/nodex/releases/download/") {
            log::error!("Invalid url");
            return Err(anyhow::anyhow!("Invalid url"));
        }

        #[cfg(unix)]
        let agent_filename = { "nodex-agent" };
        #[cfg(windows)]
        let agent_filename = { "nodex-agent.exe" };
        let agent_path = output_path.join(agent_filename);
        if PathBuf::from(&agent_path).exists() {
            fs::remove_file(&agent_path)?;
        }

        let resource_manager = ResourceManager::new();
        resource_manager
            .download_update_resources(binary_url, Some(&output_path))
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        #[cfg(unix)]
        {
            let home_dir = dirs::home_dir().unwrap();
            let resource_manager = ResourceManager::new();
            resource_manager.backup().map_err(|e| {
                log::error!("Failed to backup: {}", e);
                anyhow::anyhow!(e)
            })?;

            let runtime_info_path = home_dir
                .join(".nodex")
                .join("run")
                .join("runtime_info.json");
            let file_handler = FileHandler::new(runtime_info_path);
            let runtime_manager = RuntimeManager::new(file_handler);

            self.run_controller(&agent_path, &runtime_manager)?;
            runtime_manager.update_state(State::Update)?;
        }

        #[cfg(windows)]
        self.run_agent(&agent_path)?;

        Ok(())
    }

    #[cfg(unix)]
    fn kill_current_controller(&self, runtime_manager: &RuntimeManager) -> anyhow::Result<()> {
        let controller_processes = runtime_manager
            .filter_process_infos(FeatType::Controller)
            .map_err(|e| anyhow::anyhow!("Failed to get process infos: {}", e))?;
        for controller_process in controller_processes {
            signal::kill(
                Pid::from_raw(controller_process.process_id as i32),
                Signal::SIGTERM,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to kill process {}: {}",
                    controller_process.process_id,
                    e
                )
            })?;
        }
        Ok(())
    }

    #[cfg(unix)]
    fn run_controller(
        &self,
        agent_path: &Path,
        runtime_manager: &RuntimeManager,
    ) -> anyhow::Result<()> {
        self.kill_current_controller(runtime_manager)?;
        if is_manage_by_systemd() && is_manage_socket_activation() {
            return Ok(());
        }

        Command::new("chmod").arg("+x").arg(agent_path).status()?;

        let agent_path_str = agent_path.to_str().ok_or_else(|| {
            anyhow::anyhow!("Invalid path: failed to convert agent_path to string")
        })?;
        let cmd = CString::new(agent_path_str)
            .map_err(|e| anyhow::anyhow!("Failed to create command CString: {}", e))?;
        let args = vec![
            cmd.clone(),
            CString::new("controller")
                .map_err(|e| anyhow::anyhow!("Failed to create argument CString: {}", e))?,
        ];

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                log::info!("Parent process launched child with PID: {}", child);
                Ok(())
            }
            Ok(ForkResult::Child) => {
                setsid().map_err(|e| {
                    anyhow::anyhow!("Failed to create new session using setsid: {}", e)
                })?;

                execvp(&cmd, &args).map_err(|e| {
                    anyhow::anyhow!("Failed to execute command using execvp: {}", e)
                })?;
                unreachable!();
            }
            Err(e) => Err(anyhow::anyhow!("Failed to fork process: {}", e)),
        }
    }

    #[cfg(windows)]
    fn run_agent(&self, agent_path: &Path) -> anyhow::Result<()> {
        let agent_path_str = agent_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert agent_path to string"))?;

        let status = Command::new("cmd")
            .args(&["/C", "start", agent_path_str])
            .status()?;

        if !status.success() {
            eprintln!("Command execution failed with status: {}", status);
        } else {
            println!("Started child process");
        }

        Ok(())
    }
}
