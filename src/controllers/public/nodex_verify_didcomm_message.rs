use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use nodex_didcomm::didcomm::encrypted::DidCommEncryptedServiceVerifyError as SE;

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

    match usecase.verify(&json.message, now).await {
        Ok(v) => Ok(HttpResponse::Ok().json(v)),
        Err(e) => match e {
            UE::DidCommEncryptedServiceVerifyError(SE::VCServiceError(e)) => {
                log::warn!("verify error: {}", e);
                Ok(HttpResponse::Unauthorized().finish())
            }
            UE::DidCommEncryptedServiceVerifyError(SE::DidDocNotFound(target)) => {
                log::warn!("Target DID not found. did = {}", target);
                Ok(HttpResponse::NotFound().finish())
            }
            UE::MessageActivityHttpError(e) => Ok(utils::handle_status(e)),
            UE::JsonError(_) => todo!(),
            UE::DidCommEncryptedServiceVerifyError(SE::SidetreeFindRequestFailed(_))
            | UE::DidCommEncryptedServiceVerifyError(SE::DidPublicKeyNotFound(_))
            | UE::DidCommEncryptedServiceVerifyError(SE::DecryptFailed(_)) => todo!(),
            UE::DidCommEncryptedServiceVerifyError(SE::MetadataBodyNotFound(_))
            | UE::DidCommEncryptedServiceVerifyError(SE::JsonError(_))
            | UE::DidCommEncryptedServiceVerifyError(SE::FindSenderError(_)) => todo!(),
        },
    }
}
