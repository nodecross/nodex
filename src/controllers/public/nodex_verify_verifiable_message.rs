use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::services::{
    internal::did_vc::DIDVCService, nodex::NodeX,
    project_verifier::ProjectVerifierImplOnNetworkConfig,
};
use crate::usecase::verifiable_message_usecase::VerifiableMessageUseCase;

// NOTE: POST /verify-verifiable-message
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: String,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let usecase = VerifiableMessageUseCase::new(
        Box::new(ProjectVerifierImplOnNetworkConfig::new()),
        Box::new(NodeX::new()),
        DIDVCService::new(NodeX::new()),
    );

    match usecase.verify(&json.message).await {
        Ok(v) => Ok(HttpResponse::Ok().json(&v)),
        Err(e) => {
            log::error!("{:#?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
