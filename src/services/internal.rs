use serde_json::{Value, json};

use crate::unid::{errors::UNiDError, keyring::{self, secp256k1::Secp256k1Context}, schema::general::{GeneralVcDataModel, Issuer, CredentialSubject}, cipher::credential_signer::{CredentialSigner, CredentialSignerSuite}};

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

        let model = GeneralVcDataModel {
            id: "id".to_string(),
            r#type: vec![ "type".to_string() ],
            issuer: Issuer { id: "issuer".to_string() },
            context: vec![ "context".to_string() ],
            issuance_date: "issuance_date".to_string(),
            credential_subject: CredentialSubject {
                id: "credential_subject.id".to_string(),
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

    pub fn did_verify_vc(&self, verifiable_credential: &Value) -> Result<String, UNiDError> {
        // let verified = match CredentialSigner::verify(verifiable_credential.clone(), &CredentialSignerSuite {
        //     did: None,
        //     key_id: None,
        //     context: ()
        // }) {
        //     Ok(v) => v,
        //     Err(_) => return Err(UNiDError{})
        // };

        Ok("NotImplemented".to_string())
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