use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse };

// NOTE: POST /identifiers
#[derive(Deserialize, Serialize)]
struct MessageContainer {}

pub async fn handler(
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let service = crate::services::unid::UNiD::new();

    match service.create_identifier().await {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}