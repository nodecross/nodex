use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use nodex_didcomm::verifiable_credentials::did_vc::DidVcServiceVerifyError as SE;

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

    let usecase =
        VerifiableMessageUseCase::new(Studio::new(), utils::did_repository(), DidAccessorImpl {});

    match usecase.verify(&json.message, now).await {
        Ok(v) => Ok(HttpResponse::Ok().json(v)),
        Err(e) => match e {
            UE::DidVcServiceVerifyError(SE::VerifyFailed(e)) => {
                log::warn!("verify error: {}", e);
                Ok(HttpResponse::Unauthorized().finish())
            }
            UE::NotAddressedToMe => Ok(HttpResponse::Forbidden().finish()),
            UE::MessageActivityHttpError(e) => Ok(utils::handle_status(e)),
            UE::JsonError(_) => todo!(),
            UE::DidVcServiceVerifyError(SE::PublicKeyNotFound(_))
            | UE::DidVcServiceVerifyError(SE::DidDocNotFound(_))
            | UE::DidVcServiceVerifyError(SE::FindIdentifierError(_)) => todo!(),
        },
    }
}
