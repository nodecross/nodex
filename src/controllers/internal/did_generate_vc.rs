use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::{internal::did_vc::DIDVCService, nodex::NodeX};

// NOTE: POST /internal/verifiable-credentials
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: Value,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let now = Utc::now();
    let service = DIDVCService::new(NodeX::new());
    match service.generate(&json.message, now) {
        Ok(v) => Ok(HttpResponse::Ok().json(&v)),
        Err(e) => {
            log::error!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
