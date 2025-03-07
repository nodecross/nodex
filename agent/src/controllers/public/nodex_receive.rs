use crate::nodex::utils::did_accessor::{DidAccessor, DidAccessorImpl};
use crate::nodex::utils::webvh_client::DidWebvhDataStoreImpl;
use crate::services::nodex::update_version;
use crate::services::studio::{MessageResponse, Studio};
use anyhow::anyhow;
use controller::validator::network::can_connect_to_download_server;
use protocol::did_webvh::service::resolver::resolver_service::DidWebvhResolverService;
use protocol::did_webvh::service::service_impl::DidWebvhServiceImpl;
use protocol::didcomm::sign_encrypt::decrypt_message;
use protocol::didcomm::types::DidCommMessage;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
enum OperationType {
    UpdateAgent,
    UpdateNetworkJson,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct AckMessage {
    pub message_id: String,
}

struct MessageReceiveUsecase {
    studio: Studio,
    webvh: DidWebvhServiceImpl<DidWebvhDataStoreImpl>,
    project_did: String,
}

impl MessageReceiveUsecase {
    pub fn new() -> Self {
        let project_did = {
            let network = crate::network_config();
            let network = network.lock();
            if let Some(v) = network.get_project_did() {
                v
            } else {
                panic!("Failed to read project_did")
            }
        };
        let datastore =
            DidWebvhDataStoreImpl::new_from_server_config().expect("failed to parse url");
        let webvh = DidWebvhServiceImpl::new(datastore);
        Self {
            studio: Studio::new(),
            webvh,
            project_did,
        }
    }

    async fn handle_invalid_json(
        &self,
        m: &MessageResponse,
        e: serde_json::Error,
    ) -> Result<(), anyhow::Error> {
        self.studio
            .ack_message(&self.project_did, m.id.clone(), false)
            .await?;
        Err(anyhow::anyhow!("Invalid Json: {:?}", e))
    }

    pub async fn receive_message(&mut self) -> anyhow::Result<()> {
        for m in self.studio.get_message(&self.project_did).await? {
            let json_message: DidCommMessage = match serde_json::from_str(&m.raw_message) {
                Ok(msg) => msg,
                Err(e) => return self.handle_invalid_json(&m, e).await,
            };
            log::info!("Receive message. message_id = {:?}", m.id);

            let to_keyring = &DidAccessorImpl {}.get_my_keyring();
            let from_did = json_message.find_sender()?;
            let from_doc = self
                .webvh
                .resolve_identifier(&from_did)
                .await?
                .ok_or(anyhow!("Not found did document"))?;
            match decrypt_message(&json_message, &from_doc, to_keyring) {
                Ok(verified) => {
                    log::info!("Verify success. message_id = {}, from = {}", m.id, from_did);
                    self.studio
                        .ack_message(&self.project_did, m.id, true)
                        .await?;
                    if &*from_did == self.project_did {
                        let container = serde_json::to_value(&verified)?;
                        let operation_type = container["operation"].clone();
                        match serde_json::from_value::<OperationType>(operation_type) {
                            Ok(OperationType::UpdateAgent) => {
                                let binary_url = container["binary_url"]
                                    .as_str()
                                    .ok_or(anyhow!("the container doesn't have binary_url"))?;
                                if !can_connect_to_download_server("https://github.com").await {
                                    log::error!("Not connected to the Internet");
                                    anyhow::bail!("Not connected to the Internet");
                                } else if !binary_url.starts_with(
                                    "https://github.com/nodecross/nodex/releases/download/",
                                ) {
                                    log::error!("Invalid url");
                                    anyhow::bail!("Invalid url");
                                }
                                update_version(binary_url).await?;
                            }
                            Ok(OperationType::UpdateNetworkJson) => {
                                self.studio.network().await?;
                            }
                            Err(e) => {
                                log::error!("Json Parse Error: {:?}", e);
                            }
                        }
                        continue;
                    } else {
                        log::error!("Not supported");
                    }
                }
                Err(_) => {
                    log::error!("Verify failed : message_id = {}", m.id);
                    self.studio
                        .ack_message(&self.project_did, m.id, false)
                        .await?;
                    continue;
                }
            }
        }

        Ok(())
    }
}

pub async fn polling_task(shutdown_token: CancellationToken) {
    log::info!("Polling task is started");

    let mut usecase = MessageReceiveUsecase::new();

    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match usecase.receive_message().await {
                    Ok(_) => {},
                    Err(e) => log::error!("Error: {:?}", e),
                }
            }
            _ = shutdown_token.cancelled() => {
                break;
            },
        }
    }

    log::info!("Polling task is stopped");
}
