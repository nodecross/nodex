use chrono::Utc;
use serde_json::{Value, json};

use crate::{unid::{errors::UNiDError, keyring::{self}, schema::general::{GeneralVcDataModel, Issuer, CredentialSubject}, cipher::credential_signer::{CredentialSigner, CredentialSignerSuite}, runtime}};

pub struct Internal {}

impl Internal {
    pub fn new() -> Self {
        Internal {  }
    }

    pub fn did_generate_vc(&self, payload: &Value) -> Result<Value, UNiDError> {
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
                container: payload.clone(),
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

    pub async fn did_verify_vc(&self, verifiable_credential: &Value) -> Result<Value, UNiDError> {
        let service = crate::services::unid::UNiD::new();

        println!("verifiable_credential: {:?}", verifiable_credential);

        let model = match serde_json::from_value::<GeneralVcDataModel>(verifiable_credential.clone()) {
            Ok(v) => v,
            Err(err) => {
                println!("err: {}", err);
                return Err(UNiDError{})
            }
        };

        let did_document = match service.find_identifier(&model.issuer.id).await {
            Ok(v) => v,
            Err(err) => {
                println!("err: {:?}", err);
                return Err(UNiDError{})
            }
        };

        println!("did_document: {:?}", did_document);

        let public_keys = match did_document.did_document.public_key {
            Some(v) => v,
            None => return Err(UNiDError{}),
        };

        println!("public_keys: {:?}", public_keys);

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

        Ok(verified_model)
    }

    pub fn did_generate_vp(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn did_verify_vp(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn didcomm_generate_plaintext_message(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn didcomm_verify_plaintext_message(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn didcomm_generate_signed_message(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn didcomm_verify_signed_message(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn didcomm_generate_encrypted_message(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }

    pub fn didcomm_verify_encrypted_message(&self) -> Result<String, UNiDError> {
        Ok("NotImplemented".to_string())
    }
}