use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse };

// NOTE: POST /internal/didcomm/signed-messages/verify
#[derive(Deserialize, Serialize)]
struct InternalDidcommVerifySignedMessageRequest {}

#[derive(Deserialize, Serialize)]
struct InternalDidcommVerifySignedMessageResponse {}

pub async fn handler(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.didcomm_verify_signed_message() {
        Ok(v) => {
            Ok(HttpResponse::Ok().body(v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}