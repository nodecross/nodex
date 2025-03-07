use crate::did_webvh::domain::did_document::DidDocument;
use crate::did_webvh::domain::did_log_entry::DidLogEntry;

#[trait_variant::make(Send)]
pub trait DidWebvhDataStore {
    type Error: std::error::Error;
    async fn create(
        &mut self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<DidDocument, Self::Error>;
    async fn get(&mut self, did_path: &str) -> Result<Vec<DidLogEntry>, Self::Error>;
    async fn update(
        &mut self,
        did_path: &str,
        did_log_entries: &[DidLogEntry],
    ) -> Result<DidDocument, Self::Error>;
    async fn deactivate(&mut self, did_path: &str) -> Result<DidDocument, Self::Error>;
}
