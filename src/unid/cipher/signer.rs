use serde_json::Value;

use crate::unid::{runtime, keyring::secp256k1::Secp256k1, errors::UNiDError};

pub struct Signer {}

impl Signer {
    pub fn sign(message: &str, context: &Secp256k1) -> Result<Vec<u8>, UNiDError> {
        let payload = message.to_string();
        let digest = runtime::sha2::SHA256::digest(&payload.as_bytes());

        match runtime::secp256k1::Secp256k1::ecdsa_sign(&digest, &context.get_secret_key()) {
            Ok(v) => Ok(v),
            Err(_) => Err(UNiDError{})
        }
    }

    pub fn verify(message: &str, signature: &[u8], context: &Secp256k1) -> Result<bool, UNiDError> {
        let payload = message.to_string();
        let digest = runtime::sha2::SHA256::digest(&payload.as_bytes());

        match runtime::secp256k1::Secp256k1::ecdsa_verify(&signature, &digest, &context.get_public_key()) {
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
    fn digest() -> Vec<u8> {
        vec![
            0x20, 0x0d, 0xf8, 0x94, 0x09, 0x47, 0x71, 0x20, 0xa0, 0x88,
            0x33, 0xc7, 0xc5, 0x43, 0x24, 0x5c, 0x4d, 0xc1, 0xe1, 0x2f,
            0xa6, 0xd1, 0xb8, 0x18, 0x6c, 0x7b, 0x3c, 0xe3, 0x59, 0x3e,
            0xa0, 0x6d, 0x32, 0x85, 0x43, 0x1b, 0xd4, 0x3c, 0x42, 0xa2,
            0xe4, 0xd2, 0xb2, 0xaa, 0xb0, 0xfe, 0x82, 0x80, 0x92, 0x0d,
            0x38, 0xda, 0xe6, 0x28, 0xdf, 0xa5, 0xbe, 0x71, 0x09, 0xc0,
            0x0c, 0xb3, 0x0d, 0x76,
        ]
    }

    #[test]
    pub fn test_sign() {
        let context = match keyring::secp256k1::Secp256k1::new(&Secp256k1Context {
            public: public_key(),
            secret: secret_key(),
        }) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result = match Signer::sign(&message(), &context) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result, digest())
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

        let result = match Signer::verify(&message(), &digest(), &context) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result, true)
    }
}