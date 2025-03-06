use crate::did_webvh::domain::did_log_entry::DidLogEntry;
use http::StatusCode;

#[derive(Clone, Debug)]
pub struct DidLogEntryResponse {
    pub(crate) status_code: StatusCode,
    pub(crate) body: String,
}

impl DidLogEntryResponse {
    pub fn new(status_code: StatusCode, body: String) -> Self {
        Self { status_code, body }
    }
}

#[trait_variant::make(Send)]
pub trait DidWebvhDataStore {
    type Error: std::error::Error;
    async fn create(
        &self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<DidLogEntryResponse, Self::Error>;
    async fn get(&self, did_path: &str) -> Result<DidLogEntryResponse, Self::Error>;
    async fn update(
        &self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<Vec<DidLogEntryResponse>, Self::Error>;
    async fn deactivate(&self, did_path: &str) -> Result<Vec<DidLogEntryResponse>, Self::Error>;
}
