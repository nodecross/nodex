use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse };

// NOTE: POST /internal/didcomm/signed-messages
#[derive(Deserialize, Serialize)]
struct InternalDidcommGenerateSignedMessageRequest {}

#[derive(Deserialize, Serialize)]
struct InternalDidcommGenerateSignedMessageResponse {}

pub async fn handler(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.didcomm_generate_signed_message() {
        Ok(v) => {
            Ok(HttpResponse::Ok().body(v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}