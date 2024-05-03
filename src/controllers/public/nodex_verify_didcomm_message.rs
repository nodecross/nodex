use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::services::{
    internal::did_vc::DIDVCService, nodex::NodeX,
    project_verifier::ProjectVerifierImplOnNetworkConfig,
};
use crate::{
    services::{internal::didcomm_encrypted::DIDCommEncryptedService, studio::Studio},
    usecase::didcomm_message_usecase::{DidcommMessageUseCase, VerifyDidcommMessageUseCaseError},
};

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

    let usecase = DidcommMessageUseCase::new(
        ProjectVerifierImplOnNetworkConfig::new(),
        Studio::new(),
        DIDCommEncryptedService::new(NodeX::new(), DIDVCService::new(NodeX::new())),
    );

    match usecase.verify(&json.message, now).await {
        Ok(v) => Ok(HttpResponse::Ok().json(v)),
        Err(e) => match e {
            VerifyDidcommMessageUseCaseError::VerificationFailed => {
                Ok(HttpResponse::Unauthorized().finish())
            }
            VerifyDidcommMessageUseCaseError::TargetDidNotFound(target) => {
                log::warn!("Target DID not found. did = {}", target);
                Ok(HttpResponse::NotFound().finish())
            }
            VerifyDidcommMessageUseCaseError::BadRequest(message) => {
                log::warn!("Bad Request: {}", message);
                Ok(HttpResponse::BadRequest().body(message))
            }
            VerifyDidcommMessageUseCaseError::Unauthorized(message) => {
                log::warn!("Unauthorized: {}", message);
                Ok(HttpResponse::Unauthorized().body(message))
            }
            VerifyDidcommMessageUseCaseError::Forbidden(message) => {
                log::warn!("Forbidden: {}", message);
                Ok(HttpResponse::Forbidden().body(message))
            }
            VerifyDidcommMessageUseCaseError::NotFound(message) => {
                log::warn!("Not Found: {}", message);
                Ok(HttpResponse::NotFound().body(message))
            }
            VerifyDidcommMessageUseCaseError::Conflict(message) => {
                log::warn!("Conflict: {}", message);
                Ok(HttpResponse::Conflict().body(message))
            }
            VerifyDidcommMessageUseCaseError::Other(e) => {
                log::error!("{:?}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
        },
    }
}
