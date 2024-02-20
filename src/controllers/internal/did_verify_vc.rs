use crate::{
    nodex::schema::general::GeneralVcDataModel,
    services::{internal::did_vc::DIDVCService, nodex::NodeX},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

// NOTE: POST /internal/verifiable-credentials/verify
#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    message: GeneralVcDataModel,
}

pub async fn handler(
    _req: HttpRequest,
    web::Json(json): web::Json<MessageContainer>,
) -> actix_web::Result<HttpResponse> {
    let service = DIDVCService::new(NodeX::new());

    match service.verify(json.message).await {
        Ok(v) => Ok(HttpResponse::Ok().json(&v)),
        Err(e) => {
            log::error!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
