use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use crate::unid::{keyring::secp256k1::Secp256k1, runtime::{self, base64_url::PaddingType}, errors::UNiDError};

use super::signer::Signer;

#[derive(Debug, Serialize, Deserialize)]
struct JWSHeader {
    alg: String,
    b64: bool,
    crit: Vec<String>,
}

pub struct JWS {}

impl JWS {
    pub fn encode(object: &Value, context: &Secp256k1) -> Result<String, UNiDError> {
        // NOTE: header
        let header = JWSHeader {
            alg: "ES256K".to_string(),
            b64: false,
            crit: vec![ "b64".to_string() ],
        };
        let _header = runtime::base64_url::Base64Url::encode(
            &json!(&header).to_string().as_bytes(), &PaddingType::NoPadding
        );

        // NOTE: payload
        let _payload = runtime::base64_url::Base64Url::encode(
            &object.to_string().as_bytes(), &PaddingType::NoPadding
        );

        // NOTE: message
        let message = [ _header.clone(), _payload.clone() ].join(".");

        // NOTE: signature
        let signature = match Signer::sign(&message, &context) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };
        let _signature = runtime::base64_url::Base64Url::encode(
            &signature, &PaddingType::NoPadding
        );

        Ok([ _header.clone(), "".to_string(), _signature.clone() ].join("."))
    }


    pub fn verify(object: &Value, jws: &str, context: &Secp256k1) -> Result<bool, UNiDError> {
        let splitted: Vec<String> = jws.to_string().split('.').map(|v| v.to_string()).collect();

        if splitted.len() != 3 {
            return Err(UNiDError{})
        }

        let _header = splitted[0].clone();
        let __payload = splitted[1].clone();
        let _signature = splitted[2].clone();

        // NOTE: header
        let header = match runtime::base64_url::Base64Url::decode_as_string(&_header, &PaddingType::NoPadding) {
            Ok(v) => {
                match serde_json::from_str::<JWSHeader>(&v) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{})
        };

        if header.alg != "ES256K".to_string() {
            return Err(UNiDError{})
        }
        if header.b64 != false {
            return Err(UNiDError{})
        }
        match header.crit.iter().position(|v| v == &"b64".to_string()) {
            Some(_) => {},
            None => return Err(UNiDError{})
        };

        // NOTE: payload
        if __payload != "".to_string() {
            return Err(UNiDError{})
        }
        let _payload = runtime::base64_url::Base64Url::encode(
            &object.to_string().as_bytes(), &PaddingType::NoPadding
        );

        // NOTE: message
        let message = [ _header, _payload ].join(".");

        // NOTE: signature
        let signature = match runtime::base64_url::Base64Url::decode_as_bytes(&_signature, &PaddingType::NoPadding) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        // NOTE: verify
        match Signer::verify(&message, &signature, &context) {
            Ok(v) => Ok(v),
            Err(_) => Err(UNiDError{})
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::unid::keyring::{self, secp256k1::Secp256k1Context};

    use super::*;
    use rstest::*;

    #[fixture]
    fn secret_key() -> Vec<u8> {
        vec![
            0xc7, 0x39, 0x80, 0x5a, 0xb0, 0x3d, 0xa6, 0x2d, 0xdb, 0xe0,
            0x33, 0x90, 0xac, 0xdf, 0x76, 0x15, 0x64, 0x0a, 0xa6, 0xed,
            0x31, 0xb8, 0xf1, 0x82, 0x43, 0xf0, 0x4a, 0x57, 0x2c, 0x52,
            0x8e, 0xdb,
        ]
    }

    #[fixture]
    fn public_key() -> Vec<u8> {
        vec![
            0x02, 0x70, 0x96, 0x45, 0x32, 0xf0, 0x83, 0xf4, 0x5f, 0xe8,
            0xe8, 0xcc, 0xea, 0x96, 0xa2, 0x2f, 0x60, 0x18, 0xd4, 0x6a,
            0x40, 0x6f, 0x58, 0x3a, 0xb2, 0x26, 0xb1, 0x92, 0x83, 0xaa,
            0x60, 0x5c, 0x44,
        ]
    }

    #[fixture]
    fn message() -> String {
        String::from(r#"{"message":"Hello, UNiD!"}"#)
    }

    #[fixture]
    fn signature() -> String {
        String::from("eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..Z_JtpHSRyACClPOP7xv1QlZCumHnH-i9g26rGKj2AxYinziRxcRze1Q5e4WRHsakFbQ-qoB0itGrJfTbVK54MA")
    }

    #[test]
    pub fn test_encode() {
        let context = match keyring::secp256k1::Secp256k1::new(&Secp256k1Context {
            public: public_key(),
            secret: secret_key(),
        }) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let json: Value = match serde_json::from_str(&message()) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let result = match JWS::encode(&json, &context) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, signature())
    }

    #[test]
    pub fn test_verify() {
        let context = match keyring::secp256k1::Secp256k1::new(&Secp256k1Context {
            public: public_key(),
            secret: secret_key(),
        }) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let json: Value = match serde_json::from_str(&message()) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        let result = match JWS::verify(&json, &signature(), &context) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result, true)
    }
}