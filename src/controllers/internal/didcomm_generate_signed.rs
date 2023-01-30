use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, web };
use serde_json::Value;

use crate::services::internal::didcomm_signed::DIDCommSignedService;

// NOTE: POST /internal/didcomm/signed-messages
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    destinations: Vec<String>,
    message: Value,
}

pub async fn handler(
    req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    // NOTE: We will provide an update soon to allow multiple destinations.
    if json.destinations.len() != 1 {
        return Ok(HttpResponse::InternalServerError().finish())
    }

    let to_did = match json.destinations.first() {
        Some(v) => v,
        _ => return Ok(HttpResponse::InternalServerError().finish())
    };

    match DIDCommSignedService::generate(&to_did, &json.message, None) {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}