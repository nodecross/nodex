use crate::controllers::errors::AgentErrorCode;
use axum::extract::Json;
use protocol::did::sidetree::payload::DidResolutionResponse;

pub async fn handler() -> Result<Json<DidResolutionResponse>, AgentErrorCode> {
    let service = crate::services::nodex::NodeX::new();

    match service.create_identifier().await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::CreateIdentifierInternal)?
        }
    }
}
