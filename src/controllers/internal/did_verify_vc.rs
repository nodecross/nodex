use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, web };
use serde_json::{json, Value};

// NOTE: POST /internal/verifiable-credentials/verify
#[derive(Deserialize, Serialize)]
struct InternalVerifyVcRequest {}

#[derive(Deserialize, Serialize)]
struct InternalVerifyVcResponse {}

pub async fn handler(
    req: HttpRequest,
    web::Json(verifiable_credential): web::Json<Value>,
) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.did_verify_vc(&verifiable_credential).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) =>{
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}