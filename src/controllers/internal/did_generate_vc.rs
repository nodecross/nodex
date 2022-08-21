use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, HttpMessage, web };
use serde_json::Value;

// NOTE: POST /internal/verifiable-credentials
#[derive(Deserialize, Serialize)]
struct InternalGenerateVcRequest {}

#[derive(Deserialize, Serialize)]
struct InternalGenerateVcResponse {}

pub async fn handler(
    req: HttpRequest,
    web::Json(payload): web::Json<Value>,
) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.did_generate_vc(&payload) {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}