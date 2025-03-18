use crate::nodex::extension::secure_keystore::FileBaseKeyStore;
use crate::nodex::keyring;
use crate::nodex::utils::webvh_client::DidWebvhDataStoreImpl;
use crate::{app_config, server_config};
use anyhow;
use controller::managers::{
    resource::ResourceManagerTrait,
    runtime::{RuntimeManagerImpl, RuntimeManagerWithoutAsync, State},
};
use controller::validator::storage::check_storage;
use protocol::did_webvh::domain::did::Did;
use protocol::did_webvh::domain::did_document::DidDocument;
use protocol::did_webvh::service::controller::controller_service::DidWebvhControllerService;
use protocol::did_webvh::service::resolver::resolver_service::DidWebvhResolverService;
use protocol::did_webvh::service::service_impl::DidWebvhServiceImpl;
use std::str::FromStr;

#[cfg(windows)]
mod windows_imports {
    pub use controller::managers::resource::WindowsResourceManager;
}

#[cfg(windows)]
use windows_imports::*;

pub struct NodeX {
    webvh: DidWebvhServiceImpl<DidWebvhDataStoreImpl>,
    baseurl: url::Url,
}

impl NodeX {
    pub fn new() -> Self {
        let server_config = server_config();
        let baseurl =
            url::Url::parse(&server_config.did_http_endpoint()).expect("failed to parse url");
        let datastore = DidWebvhDataStoreImpl::new(baseurl.clone());
        let webvh = DidWebvhServiceImpl::new(datastore);

        NodeX { webvh, baseurl }
    }

    pub async fn create_identifier(&mut self) -> anyhow::Result<DidDocument> {
        let config = app_config();
        let keystore = FileBaseKeyStore::new(config.clone());
        if let Some(did) =
            keyring::keypair::KeyPairingWithConfig::load_keyring(config.clone(), keystore.clone())
                .ok()
                .and_then(|v| v.get_identifier().ok())
        {
            let did = Did::from_str(&did)?;
            if let Some(json) = self.webvh.resolve_identifier(&did).await? {
                return Ok(json);
            }
        }

        let mut keyring_with_config =
            keyring::keypair::KeyPairingWithConfig::create_keyring(config, keystore);
        let id = uuid::Uuid::new_v4();

        let host = self
            .baseurl
            .host_str()
            .ok_or(anyhow::anyhow!("Failed to get host"))?;
        let port = self.baseurl.port();
        let base = match port {
            Some(port) => &format!("{}:{}", host, port),
            None => host,
        };
        let path = format!("{}/webvh/v1/{}", base, id);

        let res = self
            .webvh
            .create_identifier(&path, true, keyring_with_config.get_keyring())
            .await?;
        keyring_with_config.save(&res.id);

        Ok(res)
    }

    pub fn update_identifier(&mut self) -> anyhow::Result<()> {
        let config = app_config();
        let keystore = FileBaseKeyStore::new(config.clone());
        if keyring::keypair::KeyPairingWithConfig::load_keyring(config.clone(), keystore.clone())?
            .get_identifier()
            .is_err()
        {}

        // if let Some(did) =
        //     keyring::keypair::KeyPairingWithConfig::load_keyring(config.clone(), keystore.clone())
        //         .ok()
        //         .and_then(|v| v.get_identifier().ok())
        // {
        //     let did = Did::from_str(&did)?;
        //     if let Some(json) = self.webvh.resolve_identifier(&did).await? {
        //         return Ok(json);
        //     }
        // }

        // let mut keyring_with_config =
        //     keyring::keypair::KeyPairingWithConfig::create_keyring(config, keystore);
        // let id = uuid::Uuid::new_v4();

        // let host = self
        //     .baseurl
        //     .host_str()
        //     .ok_or(anyhow::anyhow!("Failed to get host"))?;
        // let port = self.baseurl.port();
        // let base = match port {
        //     Some(port) => &format!("{}:{}", host, port),
        //     None => host,
        // };
        // let path = format!("{}/webvh/v1/{}", base, id);

        // let res = self
        //     .webvh
        //     .create_identifier(&path, true, keyring_with_config.get_keyring())
        //     .await?;
        // keyring_with_config.save(&res.id);

        Ok(res)
    }
}

pub async fn update_version(binary_url: &str) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        unimplemented!();
    }

    #[cfg(unix)]
    {
        let handler = controller::managers::mmap_storage::MmapHandler::new("nodex_runtime_info")?;
        let mut runtime_manager = RuntimeManagerImpl::new_by_agent(
            handler,
            controller::managers::unix_process_manager::UnixProcessManager,
        );
        let agent_path = &runtime_manager.get_runtime_info()?.exec_path;
        let output_path = agent_path
            .parent()
            .ok_or(anyhow::anyhow!("Failed to get path of parent directory"))?;
        if !check_storage(output_path) {
            log::error!("Not enough storage space: {:?}", output_path);
            anyhow::bail!("Not enough storage space");
        }
        let resource_manager = controller::managers::resource::UnixResourceManager::new(agent_path);

        resource_manager.backup().map_err(|e| {
            log::error!("Failed to backup: {}", e);
            anyhow::anyhow!(e)
        })?;

        resource_manager
            .download_update_resources(binary_url, Some(output_path))
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        runtime_manager.launch_controller(agent_path)?;
        runtime_manager.update_state(State::Update)?;
    }

    Ok(())
}
