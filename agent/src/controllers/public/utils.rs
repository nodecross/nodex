use crate::controllers::errors::AgentErrorCode;
use crate::nodex::utils::sidetree_client::SideTreeClient;
use crate::repository::message_activity_repository::MessageActivityHttpError;
use crate::server_config;
use anyhow::Context as _;
use chrono::{DateTime, Utc};
use protocol::did::did_repository::DidRepositoryImpl;

pub fn did_repository() -> DidRepositoryImpl<SideTreeClient> {
    let server_config = server_config();
    let sidetree_client = SideTreeClient::new(&server_config.did_http_endpoint())
        .context("")
        .unwrap();
    DidRepositoryImpl::new(sidetree_client)
}

pub fn handle_status(e: MessageActivityHttpError) -> AgentErrorCode {
    match e {
        MessageActivityHttpError::BadRequest(message) => {
            log::warn!("Bad Request: {}", message);
            AgentErrorCode::MessageActivityBadRequest
        }
        MessageActivityHttpError::Forbidden(message) => {
            log::warn!("Forbidden: {}", message);
            AgentErrorCode::MessageActivityForbidden
        }
        MessageActivityHttpError::Unauthorized(message) => {
            log::warn!("Unauthorized: {}", message);
            AgentErrorCode::MessageActivityUnauthorized
        }
        MessageActivityHttpError::NotFound(message) => {
            log::warn!("Not Found: {}", message);
            AgentErrorCode::MessageActivityNotFound
        }
        MessageActivityHttpError::Conflict(message) => {
            log::warn!("Conflict: {}", message);
            AgentErrorCode::MessageActivityConflict
        }
        _ => AgentErrorCode::MessageActivityInternal,
    }
}

pub fn milliseconds_to_time(milliseconds: u64) -> Option<DateTime<Utc>> {
    let milliseconds = milliseconds as i64;
    match milliseconds.to_string().len() {
        13 => {
            let secs = milliseconds / 1000;
            let nsecs = (milliseconds % 1000) * 1_000_000;
            DateTime::from_timestamp(secs, nsecs as u32)
        }
        _ => None,
    }
}
