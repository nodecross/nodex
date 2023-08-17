use crate::nodex::{
    errors::NodeXError,
    utils::hub_client::{HubClient, HubClientConfig},
};
use crate::server_config;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EmptyResponse {}

pub struct Hub {
    http_client: HubClient,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterDeviceRequest {
    device_did: String,
    project_id: String,
}

impl Hub {
    pub fn new() -> Self {
        let server_config = server_config();
        let client_config: HubClientConfig = HubClientConfig {
            base_url: server_config.hub_http_endpoint(),
        };

        let client = match HubClient::new(&client_config) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                panic!()
            }
        };

        Hub {
            http_client: client,
        }
    }

    pub async fn register_device(&self, did: String, project_id: String) -> Result<(), NodeXError> {
        let request = RegisterDeviceRequest {
            device_did: did,
            project_id,
        };
        let payload = serde_json::to_string(&request).expect("failed to serialize");
        let res = match self.http_client.post("/device", &payload).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(NodeXError {});
            }
        };
        match res.json::<EmptyResponse>().await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("{:?}", e);
                Err(NodeXError {})
            }
        }
    }
}
