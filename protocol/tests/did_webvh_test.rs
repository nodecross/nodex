#[cfg(test)]
mod tests {
    use protocol::did_webvh::domain::did::DidWebvh;
    use protocol::did_webvh::domain::did_document::DidDocument;
    use protocol::did_webvh::domain::did_log_entry::DidLogEntry;
    use protocol::did_webvh::infra::did_webvh_data_store::DidWebvhDataStore;

    use protocol::did_webvh::service::controller::controller_service::DidWebvhControllerService;
    use protocol::did_webvh::service::resolver::resolver_service::DidWebvhResolverService;
    use protocol::did_webvh::service::service_impl::DidWebvhServiceImpl;
    use protocol::keyring::*;
    use rand_core::OsRng;
    use std::fs;
    use uuid::Uuid;

    struct MockDataStore {}
    impl MockDataStore {
        fn new() -> Self {
            MockDataStore {}
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum MockDataStoreError {
        #[error("error: {0}")]
        JsonError(#[from] serde_json::Error),
        #[error("error: {0}")]
        IoError(#[from] std::io::Error),
    }
    impl DidWebvhDataStore for MockDataStore {
        type Error = MockDataStoreError;
        // localhost:8080/v1/uuidv4/did.jsonl
        async fn create(
            &mut self,
            _path: &str,
            did_log_entries: &[DidLogEntry],
        ) -> Result<DidDocument, Self::Error> {
            let log_entry = did_log_entries.last().unwrap();
            let doc = log_entry.state.clone();
            Ok(doc)
        }
        async fn get(&mut self, _path: &str) -> Result<Vec<DidLogEntry>, Self::Error> {
            let log = fs::read_to_string("test_resources/did.jsonl")?;
            let log_entries: Vec<DidLogEntry> = log
                .lines()
                .map(serde_json::from_str)
                .collect::<Result<Vec<DidLogEntry>, serde_json::Error>>()?;
            Ok(log_entries)
        }
        async fn update(
            &mut self,
            _path: &str,
            _body: &[DidLogEntry],
        ) -> Result<DidDocument, Self::Error> {
            unimplemented!()
        }
        async fn deactivate(
            &mut self,
            _path: &str,
            _body: &[DidLogEntry],
        ) -> Result<DidDocument, Self::Error> {
            unimplemented!()
        }
    }

    #[tokio::test]
    pub async fn test_create_did_log_entry() {
        let keyring = keypair::KeyPairing::create_keyring(OsRng);
        let datastore = MockDataStore::new();
        let uuid = Uuid::new_v4();
        let path = format!("localhost:8020/webvh/v1/{}", uuid);
        let res = DidWebvhServiceImpl::new(datastore)
            .create_identifier(&path, true, keyring)
            .await
            .unwrap();
        assert_eq!(res.id.get_method(), "webvh");
        let webvh_did: DidWebvh = res.id.clone().try_into().unwrap();
        assert_eq!(webvh_did.get_did(), &res.id);
        let path = path.replace(":", "%3A").replace("/", ":");
        assert_eq!(webvh_did.get_uri(), path);
    }

    #[tokio::test]
    pub async fn test_resolve_did_log_entry() {
        let did =
            "did:webvh:QmNdPXibKi8PDG77Zr293iYsNdEkSau2XteitZkWboGALz:domain.examle.com:test:did"
                .parse::<DidWebvh>()
                .unwrap();
        let datastore = MockDataStore::new();
        let mut service = DidWebvhServiceImpl::new(datastore);
        let did_doc = service
            .resolve_identifier(did.get_did())
            .await
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(did_doc.unwrap().id, did.get_did().clone());
    }
}
