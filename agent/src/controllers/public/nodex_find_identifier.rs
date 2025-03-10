use crate::controllers::errors::AgentErrorCode;
use crate::nodex::utils::webvh_client::DidWebvhDataStoreImpl;
use crate::server_config;
use axum::extract::{Json, Path};
use protocol::did_webvh::domain::did::Did;
use protocol::did_webvh::domain::did_document::DidDocument;
use protocol::did_webvh::service::resolver::resolver_service::DidWebvhResolverService;
use protocol::did_webvh::service::service_impl::DidWebvhServiceImpl;
use std::str::FromStr;

pub async fn handler(Path(did): Path<String>) -> Result<Json<Option<DidDocument>>, AgentErrorCode> {
    let server_config = server_config();
    let baseurl = url::Url::parse(&server_config.did_http_endpoint()).expect("failed to parse url");
    let datastore = DidWebvhDataStoreImpl::new(baseurl.clone());
    let mut service = DidWebvhServiceImpl::new(datastore, baseurl.scheme() == "https");
    let did = Did::from_str(&did).map_err(|_| AgentErrorCode::FindIdentifierInternal)?;
    match service.resolve_identifier(&did).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            log::error!("{:?}", e);
            Err(AgentErrorCode::FindIdentifierInternal)?
        }
    }
}
