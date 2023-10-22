use crate::network::Network;
use crate::services::hub::Hub;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// NOTE: POST /internal/version
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: Value,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(_): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let network = Network::new();
    let project_did = network.root.project_did.expect("project_did is not set");

    let hub = Hub::new();
    match hub.network(&project_did).await {
        Ok(res) => {
            let mut network_config = Network::new();
            network_config.root.secret_key = Some(res.secret_key);
            network_config.root.project_did = Some(res.project_did);
            network_config.root.recipient_dids = Some(res.recipient_dids);
            network_config.root.hub_endpoint = Some(res.hub_endpoint);
            network_config.root.heartbeat = Some(res.heartbeat);
            match network_config.save() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{:?}", e);
                    return Ok(HttpResponse::InternalServerError().json("Internal Server Error"));
                }
            };
        }
        Err(e) => {
            log::error!("{:?}", e);
            return Ok(HttpResponse::InternalServerError().json("Internal Server Error"));
        }
    };
    Ok(HttpResponse::Ok().json("ok"))
}
