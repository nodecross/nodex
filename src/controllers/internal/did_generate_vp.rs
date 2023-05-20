use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::services::internal::did_vp::DIDVPService;

// NOTE: POST /internal/verifiable-presentations
#[derive(Deserialize, Serialize)]
struct MessageContainer {}

pub async fn handler(_req: HttpRequest) -> actix_web::Result<HttpResponse> {
    match DIDVPService::generate() {
        Ok(v) => Ok(HttpResponse::Ok().body(v)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}
