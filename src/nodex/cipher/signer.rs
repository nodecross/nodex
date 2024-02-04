use crate::nodex::{keyring::secp256k1::Secp256k1, runtime};
use thiserror::Error;

pub struct Signer {}

#[derive(Debug, Error)]
pub enum SignerError {
    #[error(transparent)]
    Secp256k1Error(#[from] runtime::secp256k1::Secp256k1Error),
}

impl Signer {
    pub fn sign(message: &str, context: &Secp256k1) -> Result<Vec<u8>, SignerError> {
        Ok(runtime::secp256k1::Secp256k1::ecdsa_sign(
            message.as_bytes(),
            &context.get_secret_key(),
        )?)
    }

    pub fn verify(
        message: &str,
        signature: &[u8],
        context: &Secp256k1,
    ) -> Result<bool, SignerError> {
        Ok(runtime::secp256k1::Secp256k1::ecdsa_verify(
            signature,
            message.as_bytes(),
            &context.get_public_key(),
        )?)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::nodex::keyring::{self, secp256k1::Secp256k1Context};

    use super::*;
    use rstest::*;

    #[fixture]
    fn secret_key() -> Vec<u8> {
        vec![
            0xc7, 0x39, 0x80, 0x5a, 0xb0, 0x3d, 0xa6, 0x2d, 0xdb, 0xe0, 0x33, 0x90, 0xac, 0xdf,
            0x76, 0x15, 0x64, 0x0a, 0xa6, 0xed, 0x31, 0xb8, 0xf1, 0x82, 0x43, 0xf0, 0x4a, 0x57,
            0x2c, 0x52, 0x8e, 0xdb,
        ]
    }

    #[fixture]
    fn public_key() -> Vec<u8> {
        vec![
            0x02, 0x70, 0x96, 0x45, 0x32, 0xf0, 0x83, 0xf4, 0x5f, 0xe8, 0xe8, 0xcc, 0xea, 0x96,
            0xa2, 0x2f, 0x60, 0x18, 0xd4, 0x6a, 0x40, 0x6f, 0x58, 0x3a, 0xb2, 0x26, 0xb1, 0x92,
            0x83, 0xaa, 0x60, 0x5c, 0x44,
        ]
    }

    #[fixture]
    fn message() -> String {
        String::from(r#"{"k":"0123456789abcdef"}"#)
    }

    #[fixture]
    fn digest() -> Vec<u8> {
        vec![
            0xf7, 0x85, 0xd1, 0x25, 0xdd, 0x45, 0x64, 0x1b, 0xad, 0x3c, 0x54, 0x67, 0x4b, 0xf6,
            0xc1, 0xdf, 0xef, 0xf9, 0xe0, 0x05, 0xc8, 0xe0, 0xcf, 0x23, 0x5e, 0x29, 0x79, 0x28,
            0xb2, 0xa4, 0x54, 0x6e, 0x37, 0x4f, 0xcf, 0x9f, 0x09, 0xb9, 0x2d, 0x9c, 0x71, 0x6f,
            0xf5, 0x58, 0xd4, 0x30, 0x2b, 0xa6, 0x5c, 0x5c, 0xf5, 0xfb, 0x8a, 0xac, 0xdd, 0x26,
            0xb0, 0xc2, 0x10, 0xfa, 0xe1, 0x4c, 0xd9, 0x10,
        ]
    }

    #[test]
    pub fn test_sign() {
        let context = match keyring::secp256k1::Secp256k1::new(&Secp256k1Context {
            public: public_key(),
            secret: secret_key(),
        }) {
            Ok(v) => v,
            Err(_) => panic!(),
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
            Err(_) => panic!(),
        };

        let result = match Signer::verify(&message(), &digest(), &context) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(result)
    }
}
