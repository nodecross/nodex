use data_encoding::BASE64URL_NOPAD;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DidCommMessage {
    pub ciphertext: String,
    pub iv: String,
    pub protected: String,
    pub recipients: Vec<Recipient>,
    pub tag: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Recipient {
    pub encrypted_key: String,
    pub header: Header,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Header {
    pub alg: String,
    pub epk: Epk,
    pub iv: String,
    pub key_ops: Vec<String>,
    pub kid: String,
    pub tag: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Epk {
    pub crv: String,
    pub kty: String,
    pub x: String,
}

#[derive(Debug, Error)]
pub enum FindSenderError {
    #[error("failed serialize/deserialize: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to base64 decode protected: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("failed to base64 decode protected: {0}")]
    Decode(#[from] data_encoding::DecodeError),
    #[error("skid error")]
    Skid,
}

impl DidCommMessage {
    pub fn find_receivers(&self) -> Vec<String> {
        self.recipients.iter().map(|v| v.header.kid.clone()).collect()
    }

    pub fn find_sender(&self) -> Result<String, FindSenderError> {
        let protected = &self.protected;

        let decoded = BASE64URL_NOPAD.decode(protected.as_bytes())?;
        let decoded = String::from_utf8(decoded)?;
        let decoded = serde_json::from_str::<serde_json::Value>(&decoded)?;

        let from_did = decoded
            .get("skid")
            .ok_or(FindSenderError::Skid)?
            .as_str()
            .ok_or(FindSenderError::Skid)?
            .to_string();

        Ok(from_did)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MESSAGE: &str = include_str!("../../test_resources/collect_didcomm_message.json");

    const FROM_DID: &str = "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA";
    const TO_DID: &str = "did:nodex:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA";

    #[test]
    fn extract_from_did() {
        let message: DidCommMessage = serde_json::from_str(MESSAGE).unwrap();
        let result = message.find_sender().unwrap();
        assert_eq!(&result, FROM_DID);
    }

    #[test]
    fn extract_from_did_when_invalid_base64() {
        let message = include_str!("../../test_resources/invalid_didcomm_message.json");
        let message: DidCommMessage = serde_json::from_str(message).unwrap();
        let result = message.find_sender();
        assert!(result.is_err());
    }

    #[test]
    fn extract_to_did() {
        let message: DidCommMessage = serde_json::from_str(MESSAGE).unwrap();
        let result = message.find_receivers();
        let expected_did = vec![TO_DID.to_string()];
        assert_eq!(result, expected_did);
    }
}
