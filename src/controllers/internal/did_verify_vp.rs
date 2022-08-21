use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse };

// NOTE: POST /internal/verifiable-presentations/verify
#[derive(Deserialize, Serialize)]
struct InternalVerifyVpRequest {}

#[derive(Deserialize, Serialize)]
struct InternalVerifyVpResponse {}

pub async fn handler(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.did_verify_vp() {
        Ok(v) => {
            Ok(HttpResponse::Ok().body(v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}