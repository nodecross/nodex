use crate::nodex::extension::secure_keystore::FileBaseKeyStore;
use crate::nodex::keyring;
use crate::nodex::utils::sidetree_client::SideTreeClient;
use crate::{app_config, server_config};
use anyhow;
use bytes::Bytes;
use protocol::did::did_repository::{DidRepository, DidRepositoryImpl};
use protocol::did::sidetree::payload::DidResolutionResponse;
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};
use zip::ZipArchive;

#[cfg(unix)]
use daemonize::Daemonize;

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
        anyhow::ensure!(
            binary_url.starts_with("https://github.com/nodecross/nodex/releases/download/"),
            "Invalid url"
        );

        #[cfg(unix)]
        let agent_filename = { "nodex-agent" };
        #[cfg(windows)]
        let agent_filename = { "nodex-agent.exe" };

        let agent_path = output_path.join(agent_filename);

        let response = reqwest::get(binary_url).await?;
        let content = response.bytes().await?;
        if PathBuf::from(&agent_path).exists() {
            fs::remove_file(&agent_path)?;
        }
        self.extract_zip(content, &output_path)?;

        self.run_agent(&agent_path)?;

        Ok(())
    }

    fn extract_zip(&self, archive_data: Bytes, output_path: &Path) -> anyhow::Result<()> {
        let cursor = Cursor::new(archive_data);
        let mut archive = ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = output_path.join(file.mangled_name());

            if file.is_file() {
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut output_file = fs::File::create(&file_path)?;
                std::io::copy(&mut file, &mut output_file)?;
            } else if file.is_dir() {
                std::fs::create_dir_all(&file_path)?;
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    fn run_agent(&self, agent_path: &Path) -> anyhow::Result<()> {
        Command::new("chmod").arg("+x").arg(agent_path).status()?;

        let daemonize = Daemonize::new();
        daemonize.start().expect("Failed to update nodex process");
        std::process::Command::new(agent_path)
            .spawn()
            .expect("Failed to execute command")
            .wait()?;
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
