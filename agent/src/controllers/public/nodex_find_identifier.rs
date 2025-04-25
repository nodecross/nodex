use crate::controllers::errors::AgentErrorCode;
use crate::nodex::utils::webvh_client::DidWebvhDataStoreImpl;
use crate::server_config;
use axum::extract::Json;
use axum::http::Uri;
use protocol::did_webvh::domain::did::Did;
use protocol::did_webvh::domain::did_document::DidDocument;
use protocol::did_webvh::service::resolver::resolver_service::DidWebvhResolverService;
use protocol::did_webvh::service::service_impl::DidWebvhServiceImpl;
use std::str::FromStr;

pub async fn handler(uri: Uri) -> Result<Json<Option<DidDocument>>, AgentErrorCode> {
    let raw_path = uri.path();
    let did = if let Some(stripped) = raw_path.strip_prefix("/identifiers/") {
        stripped
    } else {
        return Err(AgentErrorCode::FindIdentifierInternal)?;
    };
    let server_config = server_config().map_err(|_| AgentErrorCode::FindIdentifierInternal)?;
    let baseurl = server_config.did_http_endpoint();
    let datastore = DidWebvhDataStoreImpl::new(baseurl.clone());
    let mut service = DidWebvhServiceImpl::new(datastore);
    let did = Did::from_str(did).map_err(|_| AgentErrorCode::FindIdentifierInternal)?;
    match service.resolve_identifier(&did).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::FindIdentifierInternal)?
        }
    }
}
