use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::services::internal::did_vp::DIDVPService;

// NOTE: POST /internal/verifiable-presentations/verify
#[derive(Deserialize, Serialize)]
struct MessageContainer {}

pub async fn handler(_req: HttpRequest) -> actix_web::Result<HttpResponse> {
    match DIDVPService::verify() {
        Ok(v) => Ok(HttpResponse::Ok().body(v)),
        Err(e) => {
            log::error!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
