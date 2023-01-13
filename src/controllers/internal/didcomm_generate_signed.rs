use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, web };
use serde_json::Value;

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
    let service = crate::services::internal::Internal::new();

    if json.destinations.len() != 1 {
        return Ok(HttpResponse::InternalServerError().finish())
    }

    let to_did = match json.destinations.first() {
        Some(v) => v,
        _ => return Ok(HttpResponse::InternalServerError().finish())
    };

    match service.didcomm_generate_signed_message(&to_did, &json.message) {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}