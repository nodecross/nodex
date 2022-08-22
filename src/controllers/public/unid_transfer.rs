use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, web };
use serde_json::Value;

// NOTE: POST /transfer
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    to_did: String,
    message: Value,
}

pub async fn handler(
    req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let service = crate::services::unid::UNiD::new();

    match service.transfer(&json.to_did, &json.message).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}