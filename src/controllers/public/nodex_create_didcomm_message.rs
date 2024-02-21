use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{services::hub::Hub, usecase::didcomm_message_usecase::DidcommMessageUseCase};
use crate::{
    services::{
        internal::{did_vc::DIDVCService, didcomm_encrypted::DIDCommEncryptedService},
        nodex::NodeX,
        project_verifier::ProjectVerifierImplOnNetworkConfig,
    },
    usecase::didcomm_message_usecase::GenerateDidcommMessageUseCaseError,
};

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

    let usecase = DidcommMessageUseCase::new(
        ProjectVerifierImplOnNetworkConfig::new(),
        Hub::new(),
        DIDCommEncryptedService::new(NodeX::new(), DIDVCService::new(NodeX::new())),
    );

    match usecase
        .generate(json.destination_did, json.message, json.operation_tag, now)
        .await
    {
        Ok(v) => Ok(HttpResponse::Ok().body(v)),
        Err(e) => match e {
            GenerateDidcommMessageUseCaseError::TargetDidNotFound(target) => {
                log::warn!("Target DID not found. did = {}", target);
                Ok(HttpResponse::NotFound().finish())
            }
            GenerateDidcommMessageUseCaseError::Other(e) => {
                log::error!("{:?}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
        },
    }
}
