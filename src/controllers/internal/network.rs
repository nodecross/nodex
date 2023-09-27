use crate::network::Network;
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
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let recipient_dids = match json.message["recipient_dids"].as_array() {
        Some(url) => url,
        None => return Ok(HttpResponse::BadRequest().json("recipient_dids is required")),
    };
    let recipient_dids: Vec<String> = recipient_dids.iter().map(|v| v.to_string()).collect();

    let hub_endpoint = match json.message["hub_endpoint"].as_str() {
        Some(p) => p,
        None => return Ok(HttpResponse::BadRequest().json("hub_endpoint is required")),
    };
    let heartbeat: u64 = match json.message["heartbeat"].as_u64() {
        Some(h) => h,
        None => return Ok(HttpResponse::BadRequest().json("heartbeat is required")),
    };
    let trm = match json.message["trm"].as_str() {
        Some(t) => t,
        None => return Ok(HttpResponse::BadRequest().json("trm is required")),
    };
    let mut network_config = Network::new();
    network_config.root.recipient_dids = Some(recipient_dids);
    network_config.root.hub_endpoint = Some(hub_endpoint.to_string());
    network_config.root.heartbeat = Some(heartbeat);
    network_config.root.trm = Some(trm.to_string());
    match network_config.save() {
        Ok(_) => {}
        Err(e) => {
            log::error!("{:?}", e);
            return Ok(HttpResponse::InternalServerError().json("Internal Server Error"));
        }
    };
    Ok(HttpResponse::Ok().json("ok"))
}
