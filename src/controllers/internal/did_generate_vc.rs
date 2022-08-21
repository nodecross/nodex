use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse };
use serde_json::json;

// NOTE: POST /internal/verifiable-credentials
#[derive(Deserialize, Serialize)]
struct InternalGenerateVcRequest {}

#[derive(Deserialize, Serialize)]
struct InternalGenerateVcResponse {}

pub async fn handler(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.did_generate_vc(&json!("{}")) {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}