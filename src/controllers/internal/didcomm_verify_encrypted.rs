use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::internal::didcomm_encrypted::DIDCommEncryptedService;

// NOTE: POST /internal/didcomm/encrypted-messages/verify
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: Value,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    match DIDCommEncryptedService::verify(&json.message).await {
        Ok(v) => Ok(HttpResponse::Ok().json(&v)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}
