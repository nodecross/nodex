use crate::nodex::extension::secure_keystore::FileBaseKeyStore;
use crate::nodex::keyring;
use crate::nodex::utils::webvh_client::DidWebvhDataStoreImpl;
use crate::{app_config, server_config};
use anyhow;
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
        let server_config = server_config().expect("Failed to get server config");
        let baseurl = server_config.did_http_endpoint();
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
}
