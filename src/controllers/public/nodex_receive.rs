use crate::services::nodex::NodeX;
use crate::services::{hub::Hub, internal::didcomm_encrypted::DIDCommEncryptedService};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

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
    hub: Hub,
    agent: NodeX,
    project_did: String,
}

impl MessageReceiveUsecase {
    pub fn new() -> Self {
        let network = crate::network_config();
        let network = network.lock();
        let project_did = if let Some(v) = network.get_project_did() {
            v
        } else {
            panic!("Failed to read project_did")
        };
        drop(network);

        Self {
            hub: Hub::new(),
            agent: NodeX::new(),
            project_did,
        }
    }

    pub async fn receive_message(&self) -> anyhow::Result<()> {
        for m in self.hub.get_message(&self.project_did).await? {
            let json_message = serde_json::from_str(&m.raw_message)
                .map_err(|e| anyhow::anyhow!("Invalid Json: {:?}", e))?;
            log::info!("Receive message. message_id = {:?}", m.id);
            match DIDCommEncryptedService::verify(&json_message).await {
                Ok(verified) => {
                    log::info!(
                        "Verify success. message_id = {}, from = {}",
                        m.id,
                        verified.message.issuer.id
                    );
                    if verified.message.issuer.id == self.project_did {
                        let container = verified.message.credential_subject.container;
                        let operation_type = container["operation"].clone();
                        match serde_json::from_value::<OperationType>(operation_type) {
                            Ok(OperationType::UpdateAgent) => {
                                let binary_url = container["binary_url"]
                                    .as_str()
                                    .ok_or(anyhow!("the container does n't have binary_url"))?;
                                self.agent
                                    .update_version(binary_url, "/tmp/nodex-agent")
                                    .await?;
                                self.hub.ack_message(&self.project_did, m.id, true).await?;
                            }
                            Ok(OperationType::UpdateNetworkJson) => {
                                self.hub.network().await?;
                                self.hub.ack_message(&self.project_did, m.id, true).await?;
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
                    self.hub.ack_message(&self.project_did, m.id, false).await?;
                    continue;
                }
            }
        }

        Ok(())
    }
}

pub async fn polling_task(shutdown_marker: Arc<AtomicBool>) {
    log::info!("Polling task is started");

    let usecase = MessageReceiveUsecase::new();

    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    while !shutdown_marker.load(std::sync::atomic::Ordering::SeqCst) {
        interval.tick().await;
        match usecase.receive_message().await {
            Ok(_) => {}
            Err(e) => log::error!("Error: {:?}", e),
        }
    }

    log::info!("Polling task is stopped");
}
