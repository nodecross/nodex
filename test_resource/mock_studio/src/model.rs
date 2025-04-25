use protocol::did_webvh::domain::did_log_entry::DidLogEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Device Registration Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_did: String,
    pub project_did: String,
}

// DIDComm Message related models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidCommMessage {
    pub ciphertext: Option<String>,
    pub iv: Option<String>,
    pub protected: Option<String>,
    pub recipients: Vec<Recipient>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    pub encrypted_key: Option<String>,
    pub header: Option<RecipientHeader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientHeader {
    pub alg: Option<String>,
    pub epk: Option<Epk>,
    pub iv: Option<String>,
    pub key_ops: Option<Vec<String>>,
    pub kid: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epk {
    pub crv: Option<String>,
    pub kty: Option<String>,
    pub x: Option<String>,
}

// Error response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

// Application state
pub type DidStore = HashMap<String, Vec<DidLogEntry>>; // uuid -> did_log_entry
pub type DeviceStore = HashMap<String, String>; // device_did -> project_did

// Shared state using Arc<Mutex<>>
pub struct AppState {
    pub did_store: Arc<Mutex<DidStore>>,
    pub device_store: Arc<Mutex<DeviceStore>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            did_store: Arc::new(Mutex::new(HashMap::new())),
            device_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn append_project_did(&self) {
        const JSONL: &str = r##"[{"versionId":"1-QmWigvMAtRZNfg9ucxuuhmMhdEbqroSTiQqVphUpHq1U2D","versionTime":"2025-04-03T09:14:00.479150470+00:00","parameters":{"portable":true,"updateKeys":["zEuo7xn7h7vT6SJaUr4DSqqPnv8QXDbwX3fT13SNVLKbK"],"nextKeyHashes":["QmZxPe5cJTL1hUfEirXQCvUz7U9ZTYv7fQruFFG6odv1cg"],"method":"did:webvh:0.5","scid":"QmVT3d3y8YrY5UsjmraJzUvtxbn3NhUygH57vRPkvy2js6"},"state":{"@context":["https://www.w3.org/ns/did/v1"],"id":"did:webvh:QmVT3d3y8YrY5UsjmraJzUvtxbn3NhUygH57vRPkvy2js6:localhost%3A8020:webvh:v1:4e73919e-34fb-4c38-a76d-950ed7cb3f2a","verificationMethod":[{"id":"#signingKey","type":"EcdsaSecp256k1VerificationKey2019","controller":"did:webvh:QmVT3d3y8YrY5UsjmraJzUvtxbn3NhUygH57vRPkvy2js6:localhost%3A8020:webvh:v1:4e73919e-34fb-4c38-a76d-950ed7cb3f2a","publicKeyJwk":{"kty":"EC","crv":"secp256k1","x":"KN5TQw6oRzV2tTTqhdjb0uzQmTixD9e42twdXeatR8E","y":"xxg_o55AyG3DcJpU_KrUprcD3bV9vG-MePuuz0tY06Q"}},{"id":"#encryptionKey","type":"X25519KeyAgreementKey2019","controller":"did:webvh:QmVT3d3y8YrY5UsjmraJzUvtxbn3NhUygH57vRPkvy2js6:localhost%3A8020:webvh:v1:4e73919e-34fb-4c38-a76d-950ed7cb3f2a","publicKeyJwk":{"kty":"OKP","crv":"X25519","x":"z2TUfU7ZHY6JHQRjtfAaOgJ-rVF8WGvL5K8Zt0ZaR28"}},{"id":"#signTimeSeriesKey","type":"Ed25519VerificationKey2018","controller":"did:webvh:QmVT3d3y8YrY5UsjmraJzUvtxbn3NhUygH57vRPkvy2js6:localhost%3A8020:webvh:v1:4e73919e-34fb-4c38-a76d-950ed7cb3f2a","publicKeyJwk":{"kty":"OKP","crv":"Ed25519","x":"CtLp4OYDEgVmmAFbQFtC2O2JE8dk0N_w5DDGBk59tiA"}}]},"proof":[{"type":"DataIntegrityProof","cryptosuite":"eddsa-jcs-2022","verificationMethod":"did:key:zEuo7xn7h7vT6SJaUr4DSqqPnv8QXDbwX3fT13SNVLKbK#zEuo7xn7h7vT6SJaUr4DSqqPnv8QXDbwX3fT13SNVLKbK","created":"2025-04-03T09:14:00.482007207+00:00","proofPurpose":"authentication","proofValue":"z25PGbzSYcSkgRqL8UvLBk55ez7qh6voopK9JKXagZrabKXekUMj6vnCRAgfy59XV1NP3EnsRfwTHqQAvHYd6SwJV"}]}]"##;
        const UUID: &str = "4e73919e-34fb-4c38-a76d-950ed7cb3f2a";

        let log_entry =
            serde_json::from_str::<Vec<DidLogEntry>>(JSONL).expect("Failed to parse JSONL string");

        self.did_store
            .lock()
            .unwrap()
            .insert(UUID.to_string(), log_entry);
    }
}
