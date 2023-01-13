use arrayref::array_ref;
use chrono::Utc;
use serde_json::{Value, json};
use didcomm_rs::{Message, crypto::{SignatureAlgorithm, Signer, CryptoAlgorithm}};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{unid::{errors::UNiDError, keyring::{self}, schema::general::{GeneralVcDataModel, Issuer, CredentialSubject}, cipher::credential_signer::{CredentialSigner, CredentialSignerSuite}, runtime::{self, base64_url::{self, PaddingType}}}};

pub struct Internal {}

impl Internal {
    pub fn new() -> Self {
        Internal {  }
    }

    pub fn did_generate_vc(&self, message: &Value) -> Result<Value, UNiDError> {
        let keyring = match keyring::mnemonic::MnemonicKeyring::load_keyring() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let did = match keyring.get_identifier() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let r#type = "VerifiableCredential".to_string();
        let context = "https://www.w3.org/2018/credentials/v1".to_string();
        let issuance_date = Utc::now().to_rfc3339();

        let model = GeneralVcDataModel {
            id: None,
            issuer: Issuer { id: did.clone() },
            r#type: vec![ r#type ],
            context: vec![ context ],
            issuance_date,
            credential_subject: CredentialSubject {
                id: None,
                container: message.clone(),
            },
            expiration_date: None,
            proof: None,
        };

        let signed = match CredentialSigner::sign(&model, &CredentialSignerSuite {
            did: Some(did),
            key_id: Some("signingKey".to_string()),
            context: keyring.get_sign_key_pair(),
        }) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        Ok(json!(signed))
    }

    pub async fn did_verify_vc(&self, message: &Value) -> Result<Value, UNiDError> {
        let service = crate::services::unid::UNiD::new();

        let model = match serde_json::from_value::<GeneralVcDataModel>(message.clone()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let did_document = match service.find_identifier(&model.issuer.id).await {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };
        let public_keys = match did_document.did_document.public_key {
            Some(v) => v,
            None => return Err(UNiDError{}),
        };

        // FIXME: workaround
        if public_keys.len() != 1 {
            return Err(UNiDError{})
        }

        let public_key = public_keys[0].clone();

        let context = match keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let (verified_model, verified) = match CredentialSigner::verify(&model, &CredentialSignerSuite {
            did: None,
            key_id: None,
            context,
        }) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        if !verified {
            return Err(UNiDError{})
        }

        Ok(verified_model)
    }

    pub fn did_generate_vp(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn did_verify_vp(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn didcomm_generate_plaintext_message(&self, to_did: &str, message: &Value) -> Result<Value, UNiDError> {
        let keyring = match keyring::mnemonic::MnemonicKeyring::load_keyring() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let did = match keyring.get_identifier() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let body = match self.did_generate_vc(&message) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let message = Message::new()
            .from(&did)
            .to(&[ &to_did ])
            .body(&body.to_string());

        match message.clone().as_raw_json() {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        }
    }

    pub fn didcomm_verify_plaintext_message(&self, message: &Value) -> Result<Value, UNiDError> {
        let message = match Message::receive(&message.to_string(), None, None, None) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        match message.clone().get_body() {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        }
    }

    pub fn didcomm_generate_signed_message(&self, to_did: &str, message: &Value) -> Result<Value, UNiDError> {
        let keyring = match keyring::mnemonic::MnemonicKeyring::load_keyring() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let did = match keyring.get_identifier() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let body = match self.did_generate_vc(&message) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let message = Message::new()
            .from(&did)
            .to(&[ &to_did ])
            .body(&body.to_string());

        match message.clone()
            .as_jws(&SignatureAlgorithm::Es256k)
            .sign(SignatureAlgorithm::Es256k.signer(), &keyring.get_sign_key_pair().get_secret_key()) {
                Ok(v) => {
                    match serde_json::from_str::<Value>(&v) {
                        Ok(v) => Ok(v),
                        Err(_) => Err(UNiDError{}),
                    }
                },
                Err(_) => Err(UNiDError{})
            }
    }

    pub async fn didcomm_verify_signed_message(&self, message: &Value) -> Result<Value, UNiDError> {
        let service = crate::services::unid::UNiD::new();

        let payload = match message.get("payload") {
            Some(v) => {
                match v.as_str() {
                    Some(v) => v.to_string(),
                    None => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };

        let decoded = match base64_url::Base64Url::decode_as_string(&payload, &PaddingType::NoPadding) {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        };

        let from_did = match decoded.get("from") {
            Some(v) => {
                match v.as_str() {
                    Some(v) => v.to_string(),
                    None => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };

        let did_document = match service.find_identifier(&from_did).await {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };
        let public_keys = match did_document.did_document.public_key {
            Some(v) => v,
            None => return Err(UNiDError{}),
        };

        // FIXME: workaround
        if public_keys.len() != 1 {
            return Err(UNiDError{})
        }

        let public_key = public_keys[0].clone();

        let context = match keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let message = match Message::verify(&message.to_string().as_bytes(), &context.get_public_key()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        match message.clone().get_body() {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        }
    }

    pub async fn didcomm_generate_encrypted_message(&self, to_did: &str, message: &Value) -> Result<Value, UNiDError> {
        let service = crate::services::unid::UNiD::new();

        // NOTE: recipient from
        let my_keyring = match keyring::mnemonic::MnemonicKeyring::load_keyring() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let my_did = match my_keyring.get_identifier() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        // NOTE: recipient to
        let did_document = match service.find_identifier(&to_did).await {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };
        let public_keys = match did_document.did_document.public_key {
            Some(v) => v,
            None => return Err(UNiDError{}),
        };

        // FIXME: workaround
        if public_keys.len() != 1 {
            return Err(UNiDError{})
        }

        let public_key = public_keys[0].clone();

        let other_key = match keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        // NOTE: ecdh
        let shared_key = match runtime::secp256k1::Secp256k1::ecdh(
            &my_keyring.get_sign_key_pair().get_secret_key(),
            &other_key.get_public_key(),
        ) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let sk = StaticSecret::from(array_ref!(shared_key, 0, 32).to_owned());
        let pk = PublicKey::from(&sk);

        // NOTE: message
        let body = match self.did_generate_vc(&message) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let message = Message::new()
            .from(&my_did)
            .to(&[ &to_did ])
            .body(&body.to_string());

        match message.clone()
            .as_jwe(&CryptoAlgorithm::XC20P, Some(pk.as_bytes().to_vec()))
            .seal_signed(
                &sk.to_bytes().to_vec(),
                Some(vec![ Some(pk.as_bytes().to_vec()) ]),
                SignatureAlgorithm::Es256k,
                &my_keyring.get_sign_key_pair().get_secret_key()
            ) {
                Ok(v) => {
                    match serde_json::from_str::<Value>(&v) {
                        Ok(v) => Ok(v),
                        Err(_) => return Err(UNiDError{}),
                    }
                },
                Err(_) => return Err(UNiDError{})
            }
    }

    pub async fn didcomm_verify_encrypted_message(&self, message: &Value) -> Result<Value, UNiDError> {
        let service = crate::services::unid::UNiD::new();

        // NOTE: recipient to
        let my_keyring = match keyring::mnemonic::MnemonicKeyring::load_keyring() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        // NOTE: recipient from
        let protected = match message.get("protected") {
            Some(v) => {
                match v.as_str() {
                    Some(v) => v.to_string(),
                    None => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{})
        };

        let decoded = match base64_url::Base64Url::decode_as_string(&protected, &PaddingType::NoPadding) {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        };

        let other_did = match decoded.get("skid") {
            Some(v) => {
                match v.as_str() {
                    Some(v) => v.to_string(),
                    None => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };

        let did_document = match service.find_identifier(&other_did).await {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let public_keys = match did_document.did_document.public_key {
            Some(v) => v,
            None => return Err(UNiDError{}),
        };

        // FIXME: workaround
        if public_keys.len() != 1 {
            return Err(UNiDError{})
        }

        let public_key = public_keys[0].clone();

        let other_key = match keyring::secp256k1::Secp256k1::from_jwk(&public_key.public_key_jwk) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        // NOTE: ecdh
        let shared_key = match runtime::secp256k1::Secp256k1::ecdh(
            &my_keyring.get_sign_key_pair().get_secret_key(),
            &other_key.get_public_key(),
        ) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let sk = StaticSecret::from(array_ref!(shared_key, 0, 32).to_owned());
        let pk = PublicKey::from(&sk);

        let message = match Message::receive(
            &message.to_string(),
            Some(&sk.to_bytes().to_vec()),
            Some(pk.as_bytes().to_vec()),
            Some(&other_key.get_public_key()),
        ) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        match message.clone().get_body() {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        }
    }
}