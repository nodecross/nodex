use super::interfaces::did_document::*;

#[derive(Debug)]
pub struct UNiDDidDocument {
    pub document: DidDocument,
}

impl UNiDDidDocument {
    pub fn new(document: DidDocument) -> Self {
        UNiDDidDocument { document }
    }
    pub fn document(&self) -> DidDocument {
        self.document.clone()
    }
    pub fn identifier(&self) -> String {
        self.document.id.to_string()
    }
    pub fn public_keys(&self) -> Vec<DidPublicKey> {
        self.document.public_key.clone()
    }
    pub fn services(&self) -> Vec<::serde_json::Value> {
        self.document.service.clone()
    }

    pub fn get_public_key(&self, key_id: String) -> DidPublicKey {
        let ks: Vec<DidPublicKey> = self.document().public_key;
        let filtered_ks = ks
            .iter()
            .cloned()
            .filter(|x| x.id == format!("#{}", key_id))
            .collect::<Vec<DidPublicKey>>();
        let k = filtered_ks.first();
        assert!(k.is_some());
        k.unwrap().clone()
    }
}