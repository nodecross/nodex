use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse };

// NOTE: GET /identifiers/${ did }
#[derive(Deserialize, Serialize)]
struct FindIdentifierByIdRequest {}

#[derive(Deserialize, Serialize)]
struct FindIdentifierByIdResponse {}

pub async fn handler(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let service = crate::services::unid::UNiD::new();
    let params = req.match_info();

    let did = match params.get("did") {
        Some(v) => v,
        None => {
            return Ok(HttpResponse::BadRequest().finish())
        }
    };

    match service.find_identifier(&did).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}