use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::usecase::verifiable_message_usecase::VerifiableMessageUseCase;
use crate::{
    services::project_verifier::ProjectVerifierImplOnNetworkConfig,
    usecase::verifiable_message_usecase::DidRepositoryImpl,
};

// NOTE: POST /create-verifiable-message
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    destination_did: String,
    message: String,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let now = Utc::now();

    let usecase = VerifiableMessageUseCase::new(
        Box::new(ProjectVerifierImplOnNetworkConfig::new()),
        Box::new(DidRepositoryImpl::new()),
    );

    match usecase
        .generate(json.destination_did, json.message, now)
        .await
    {
        Ok(v) => Ok(HttpResponse::Ok().json(&v)),
        Err(e) => {
            log::error!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
