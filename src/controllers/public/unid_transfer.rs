use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, web };
use serde_json::Value;

// NOTE: POST /transfer
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    destinations: Vec<String>,
    messages: Vec<Value>,
    metadata: Value,
}

pub async fn handler(
    req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let service = crate::services::unid::UNiD::new();

    // NOTE: We will provide an update soon to allow multiple destinations.
    if json.destinations.len() != 1 {
        return Ok(HttpResponse::InternalServerError().finish())
    }

    let to_did = match json.destinations.first() {
        Some(v) => v,
        _ => return Ok(HttpResponse::InternalServerError().finish())
    };

    match service.transfer(&to_did, &json.messages, &json.metadata).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}