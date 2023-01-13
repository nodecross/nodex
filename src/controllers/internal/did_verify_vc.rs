use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, web };
use serde_json::Value;

// NOTE: POST /internal/verifiable-credentials/verify
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: Value,
}

pub async fn handler(
    req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.did_verify_vc(&json.message).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) =>{
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}