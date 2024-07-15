use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::nodex::utils::did_accessor::DidAccessorImpl;
use crate::usecase::verifiable_message_usecase::CreateVerifiableMessageUseCaseError as UE;
use crate::{
    services::studio::Studio, usecase::verifiable_message_usecase::VerifiableMessageUseCase,
};

use super::utils;

// NOTE: POST /create-verifiable-message
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
        VerifiableMessageUseCase::new(Studio::new(), utils::did_repository(), DidAccessorImpl {});

    match usecase
        .generate(json.destination_did, json.message, json.operation_tag, now)
        .await
    {
        Ok(v) => Ok(HttpResponse::Ok().body(v)),
        Err(e) => match e {
            UE::DestinationNotFound => Ok(HttpResponse::NotFound().finish()),
            UE::MessageActivityHttpError(e) => Ok(utils::handle_status(e)),
            UE::DidVcServiceGenerateError(e) => {
                log::error!("{:?}", e);
                Ok(HttpResponse::InternalServerError().finish())
            }
            UE::JsonError(_) => todo!(),
        },
    }
}
