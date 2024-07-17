use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use nodex_didcomm::didcomm::encrypted::DidCommEncryptedServiceVerifyError as SE;
use nodex_didcomm::didcomm::types::DidCommMessage;

use crate::nodex::utils::did_accessor::DidAccessorImpl;
use crate::{
    services::studio::Studio,
    usecase::didcomm_message_usecase::{
        DidcommMessageUseCase, VerifyDidcommMessageUseCaseError as UE,
    },
};

use super::utils;

// NOTE: POST /verify-verifiable-message
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: String,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let now = Utc::now();

    let usecase =
        DidcommMessageUseCase::new(Studio::new(), utils::did_repository(), DidAccessorImpl {});

    match serde_json::from_str::<DidCommMessage>(&json.message) {
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
        Ok(message) => match usecase.verify(message, now).await {
            Ok(v) => Ok(HttpResponse::Ok().json(v)),
            Err(e) => match e {
                UE::MessageActivity(e) => Ok(utils::handle_status(e)),
                UE::NotAddressedToMe => {
                    log::warn!("its not to me: {}", e);
                    Ok(HttpResponse::BadRequest().body(e.to_string()))
                }
                UE::ServiceVerify(SE::FindSender(e)) => {
                    log::warn!("cannot find sender: {}", e);
                    Ok(HttpResponse::BadRequest().body(e.to_string()))
                }
                UE::ServiceVerify(SE::DidPublicKeyNotFound(e)) => {
                    log::warn!("cannot public key: {}", e);
                    Ok(HttpResponse::BadRequest().body(e.to_string()))
                }
                UE::ServiceVerify(SE::MetadataBodyNotFound(e)) => {
                    let e = e.map(|e| e.to_string()).unwrap_or("".to_string());
                    log::warn!("cannot find sender: {}", e);
                    Ok(HttpResponse::BadRequest().body(e))
                }
                UE::ServiceVerify(SE::VcService(e)) => {
                    log::warn!("verify error: {}", e);
                    Ok(HttpResponse::Unauthorized().finish())
                }
                UE::ServiceVerify(SE::DidDocNotFound(target)) => {
                    log::warn!("Target DID not found. did = {}", target);
                    Ok(HttpResponse::NotFound().finish())
                }
                UE::Json(e) | UE::ServiceVerify(SE::Json(e)) => {
                    log::warn!("json error: {}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
                UE::ServiceVerify(SE::DecryptFailed(e)) => {
                    log::warn!("decrypt failed: {}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
                UE::ServiceVerify(SE::SidetreeFindRequestFailed(e)) => {
                    log::warn!("sidetree error: {}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            },
        },
    }
}
