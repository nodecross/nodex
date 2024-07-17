use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use nodex_didcomm::didcomm::encrypted::DidCommEncryptedServiceGenerateError as SE;

use crate::nodex::utils::did_accessor::DidAccessorImpl;
use crate::usecase::didcomm_message_usecase::GenerateDidcommMessageUseCaseError as UE;
use crate::{services::studio::Studio, usecase::didcomm_message_usecase::DidcommMessageUseCase};

use super::utils;

// NOTE: POST /create-didcomm-message
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    destination_did: String,
    message: String,
    operation_tag: String,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let now = Utc::now();

    let usecase =
        DidcommMessageUseCase::new(Studio::new(), utils::did_repository(), DidAccessorImpl {});

    match usecase
        .generate(json.destination_did, json.message, json.operation_tag, now)
        .await
    {
        Ok(v) => Ok(HttpResponse::Ok().body(v)),
        Err(e) => match e {
            UE::MessageActivity(e) => Ok(utils::handle_status(e)),
            UE::ServiceGenerate(SE::DidDocNotFound(target)) => {
                log::warn!("Target DID not found. did = {}", target);
                Ok(HttpResponse::NotFound().finish())
            }
            UE::ServiceGenerate(SE::DidPublicKeyNotFound(e)) => {
                log::warn!("cannot public key: {}", e);
                Ok(HttpResponse::BadRequest().body(e.to_string()))
            }
            UE::Json(e) | UE::ServiceGenerate(SE::Json(e)) => {
                log::warn!("json error: {}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
            UE::ServiceGenerate(SE::VcService(e)) => {
                log::warn!("verify error: {}", e);
                Ok(HttpResponse::Unauthorized().finish())
            }
            UE::ServiceGenerate(SE::SidetreeFindRequestFailed(e)) => {
                log::warn!("sidetree error: {}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
            UE::ServiceGenerate(SE::EncryptFailed(e)) => {
                log::warn!("decrypt failed: {}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
        },
    }
}
