use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse };
use serde_json::json;

// NOTE: POST /internal/verifiable-credentials/verify
#[derive(Deserialize, Serialize)]
struct InternalVerifyVcRequest {}

#[derive(Deserialize, Serialize)]
struct InternalVerifyVcResponse {}

pub async fn handler(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.did_verify_vc(&json!("{}")) {
        Ok(v) => {
            Ok(HttpResponse::Ok().body(v))
        },
        Err(_) =>{
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}