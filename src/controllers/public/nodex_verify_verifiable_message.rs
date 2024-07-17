use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use nodex_didcomm::verifiable_credentials::did_vc::DidVcServiceVerifyError as SE;
use nodex_didcomm::verifiable_credentials::types::VerifiableCredentials;

use crate::nodex::utils::did_accessor::DidAccessorImpl;
use crate::usecase::verifiable_message_usecase::VerifyVerifiableMessageUseCaseError as UE;
use crate::{
    services::studio::Studio, usecase::verifiable_message_usecase::VerifiableMessageUseCase,
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

    let repo = utils::did_repository();
    let usecase =
        VerifiableMessageUseCase::new(Studio::new(), repo.clone(), DidAccessorImpl {}, repo);

    match serde_json::from_str::<VerifiableCredentials>(&json.message) {
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
        Ok(vc) => match usecase.verify(vc, now).await {
            Ok(v) => Ok(HttpResponse::Ok().json(v)),
            Err(e) => match e {
                UE::MessageActivity(e) => Ok(utils::handle_status(e)),
                UE::DidVcServiceVerify(SE::VerifyFailed(e)) => {
                    log::warn!("verify error: {}", e);
                    Ok(HttpResponse::Unauthorized().finish())
                }
                UE::DidVcServiceVerify(SE::FindIdentifier(e)) => {
                    log::warn!("find identifier error: {}", e);
                    Ok(HttpResponse::NotFound().finish())
                }
                UE::DidVcServiceVerify(SE::DidDocNotFound(target)) => {
                    log::warn!("Target DID not found. did = {}", target);
                    Ok(HttpResponse::NotFound().finish())
                }
                UE::NotAddressedToMe => Ok(HttpResponse::Forbidden().finish()),
                UE::Json(e) => {
                    log::warn!("json error: {}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
                UE::DidVcServiceVerify(SE::PublicKeyNotFound(e)) => {
                    log::warn!("cannot public key: {}", e);
                    Ok(HttpResponse::BadRequest().body(e.to_string()))
                }
            },
        },
    }
}
