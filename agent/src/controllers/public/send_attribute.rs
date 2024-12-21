use crate::{
    controllers::errors::AgentErrorCode, repository::attribute_repository::AttributeStoreRequest,
    usecase::attribute_usecase::AttributeUsecase,
};
use axum::extract::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MessageContainer {
    #[serde(default)]
    key_name: String,
    #[serde(default)]
    value: String,
}

pub async fn handler(Json(json): Json<MessageContainer>) -> Result<(), AgentErrorCode> {
    if json.key_name.is_empty() {
        Err(AgentErrorCode::SendAttributeNoKeyName)?
    }
    if json.value.is_empty() {
        Err(AgentErrorCode::SendAttributeNoValue)?
    }

    let usecase = AttributeUsecase::new();
    match usecase
        .save(AttributeStoreRequest {
            key_name: json.key_name,
            value: json.value,
        })
        .await
    {
        Ok(_) => {
            log::info!("save attribute");
            Ok(())
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::SendAttributeInternal)?
        }
    }
}
