use crate::server_config;
use protocol::did_webvh::domain::did_document::DidDocument;
use protocol::did_webvh::domain::did_log_entry::DidLogEntry;
use protocol::did_webvh::infra::did_webvh_data_store::DidWebvhDataStore;
use url::{ParseError, Url};

#[derive(Clone)]
pub struct DidWebvhDataStoreImpl {
    base_url: Url,
    client: reqwest::Client,
}

impl DidWebvhDataStoreImpl {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub fn new_from_server_config() -> Result<Self, ParseError> {
        let server_config = server_config();
        let base_url = &server_config.did_http_endpoint();
        let base_url = Url::parse(base_url)?;
        Ok(Self::new(base_url))
    }
}

impl DidWebvhDataStore for DidWebvhDataStoreImpl {
    type Error = ParseError;

    async fn create(
        &self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<DidDocument, Self::Error> {
        unimplemented!();
    }
    async fn get(&self, did_path: &str) -> Result<Vec<DidLogEntry>, Self::Error> {
        unimplemented!();
    }

    async fn update(
        &self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<DidDocument, Self::Error> {
        unimplemented!();
    }
    async fn deactivate(&self, did_path: &str) -> Result<DidDocument, Self::Error> {
        unimplemented!();
    }
}
