use crate::did_webvh::domain::did_log_entry::DidLogEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataStoreError {
    NotFound,
    InternalServerError,
}
#[trait_variant::make(Send)]
pub trait DidWebvhDataStore {
    async fn post(&self, did_path: &str, did_log_entry: DidLogEntry) -> Result<(), DataStoreError>;
    async fn get(&self, did_path: &str) -> Result<Vec<DidLogEntry>, DataStoreError>;
    async fn put(
        &self,
        did_path: &str,
        did_log_entry: DidLogEntry,
    ) -> Result<Vec<DidLogEntry>, DataStoreError>;
    async fn delete(&self, did_path: &str) -> Result<Vec<DidLogEntry>, DataStoreError>;
}
