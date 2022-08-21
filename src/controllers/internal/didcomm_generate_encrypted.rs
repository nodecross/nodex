use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse };

// NOTE: POST /internal/didcomm/encrypted-messages
#[derive(Deserialize, Serialize)]
struct InternalDidcommGenerateEncryptedMessageRequest {}

#[derive(Deserialize, Serialize)]
struct InternalDidcommGenerateEncryptedMessageResponse {}

pub async fn handler(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let service = crate::services::internal::Internal::new();

    match service.didcomm_generate_encrypted_message() {
        Ok(v) => {
            Ok(HttpResponse::Ok().body(v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}