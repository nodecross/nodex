use crate::controllers::errors::AgentErrorCode;
use axum::extract::{Json, Path};
use protocol::did::sidetree::payload::DidResolutionResponse;

pub async fn handler(
    did: Path<String>,
) -> Result<Json<Option<DidResolutionResponse>>, AgentErrorCode> {
    let service = crate::services::nodex::NodeX::new();

    match service.find_identifier(&did).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::FindIdentifierInternal)?
        }
    }
}
