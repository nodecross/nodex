use crate::controllers::errors::AgentErrorCode;
use axum::extract::Json;
use protocol::did_webvh::domain::did_document::DidDocument;

pub async fn handler() -> Result<Json<DidDocument>, AgentErrorCode> {
    let mut service = crate::services::nodex::NodeX::new();

    match service.create_identifier().await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::CreateIdentifierInternal)?
        }
    }
}
