use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::{
    internal::{did_vc::DIDVCService, didcomm_encrypted::DIDCommEncryptedService},
    nodex::NodeX,
};

// NOTE: POST /internal/didcomm/encrypted-messages/verify
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: Value,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    // TODO: refactoring more simple
    let service = DIDCommEncryptedService::new(NodeX::new(), DIDVCService::new(NodeX::new()));

    match service.verify(&json.message).await {
        Ok(v) => Ok(HttpResponse::Ok().json(&v)),
        Err(e) => {
            log::error!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
