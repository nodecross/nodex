use crate::nodex::extension::secure_keystore::FileBaseKeyStore;
use crate::nodex::keyring;
use crate::nodex::utils::sidetree_client::SideTreeClient;
use crate::{app_config, server_config};
use anyhow;

use controller::managers::{
    mmap_storage::MmapHandler,
    resource::ResourceManagerTrait,
    runtime::{FeatType, RuntimeInfoStorage, RuntimeManager, State},
};
use protocol::did::did_repository::{DidRepository, DidRepositoryImpl};
use protocol::did::sidetree::payload::DidResolutionResponse;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use controller::validator::storage::check_storage;

#[cfg(unix)]
mod unix_imports {
    pub use controller::managers::resource::UnixResourceManager;
    pub use nix::{
        sys::signal::{self, Signal},
        unistd::{execvp, fork, setsid, ForkResult, Pid},
    };
    pub use std::ffi::CString;
}

#[cfg(unix)]
use unix_imports::*;

#[cfg(windows)]
mod windows_imports {
    pub use controller::managers::resource::WindowsResourceManager;
}

#[cfg(windows)]
use windows_imports::*;

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

    pub async fn update_version(&self, binary_url: &str) -> anyhow::Result<()> {
        #[cfg(windows)]
        {
            let resource_manager = WindowsResourceManager::new();
            self.run_agent(&agent_path)?;
        }

        #[cfg(unix)]
        {
            let len = std::env::var("MMAP_SIZE")
                .ok()
                .and_then(|x| x.parse::<usize>().ok())
                .ok_or(anyhow::anyhow!("Incompatible size"))?;
            let len = core::num::NonZero::new(len).ok_or(anyhow::anyhow!("Incompatible size"))?;
            let handler = MmapHandler::new("nodex_runtime_info", len)?;
            let mut runtime_manager = RuntimeManager::new(handler)?;
            let agent_path = &runtime_manager.read_runtime_info()?.exec_path;
            let output_path = agent_path
                .parent()
                .ok_or(anyhow::anyhow!("Failed to get path of parent directory"))?;
            if !check_storage(output_path) {
                log::error!("Not enough storage space: {:?}", output_path);
                anyhow::bail!("Not enough storage space");
            }
            let resource_manager = UnixResourceManager::new(agent_path);

            resource_manager.backup().map_err(|e| {
                log::error!("Failed to backup: {}", e);
                anyhow::anyhow!(e)
            })?;

            resource_manager
                .download_update_resources(binary_url, Some(output_path))
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

            runtime_manager.run_controller(&agent_path)?;
            runtime_manager.update_state(State::Update)?;
        }

        Ok(())
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
