use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    services::studio::Studio, usecase::verifiable_message_usecase::VerifiableMessageUseCase,
};
use crate::{
    services::{
        internal::did_vc::DIDVCService, nodex::NodeX,
        project_verifier::ProjectVerifierImplOnNetworkConfig,
    },
    usecase::verifiable_message_usecase::VerifyVerifiableMessageUseCaseError,
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

    let usecase = VerifiableMessageUseCase::new(
        Box::new(ProjectVerifierImplOnNetworkConfig::new()),
        Box::new(NodeX::new()),
        Box::new(Studio::new()),
        DIDVCService::new(NodeX::new()),
    );

    match usecase.verify(&json.message, now).await {
        Ok(v) => Ok(HttpResponse::Ok().json(v)),
        Err(e) => match e {
            VerifyVerifiableMessageUseCaseError::VerificationFailed => {
                Ok(HttpResponse::Unauthorized().finish())
            }
            VerifyVerifiableMessageUseCaseError::NotAddressedToMe => {
                Ok(HttpResponse::Forbidden().finish())
            }
            VerifyVerifiableMessageUseCaseError::VCServiceFailed(e) => {
                log::error!("{:?}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
            VerifyVerifiableMessageUseCaseError::BadRequest(message) => {
                log::warn!("Bad Request: {}", message);
                Ok(HttpResponse::BadRequest().body(message))
            }
            VerifyVerifiableMessageUseCaseError::Unauthorized(message) => {
                log::warn!("Unauthorized: {}", message);
                Ok(HttpResponse::Unauthorized().body(message))
            }
            VerifyVerifiableMessageUseCaseError::Forbidden(message) => {
                log::warn!("Forbidden: {}", message);
                Ok(HttpResponse::Forbidden().body(message))
            }
            VerifyVerifiableMessageUseCaseError::NotFound(message) => {
                log::warn!("NotFound: {}", message);
                Ok(HttpResponse::NotFound().body(message))
            }
            VerifyVerifiableMessageUseCaseError::Conflict(message) => {
                log::warn!("Conflict: {}", message);
                Ok(HttpResponse::Conflict().body(message))
            }
            VerifyVerifiableMessageUseCaseError::Other(e) => {
                log::error!("{:?}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
        },
    }
}
