use hmac::digest::generic_array::GenericArray;
use k256::{
    ecdsa::{SigningKey, VerifyingKey, Signature, signature::{Signer, Verifier}},
    PublicKey, elliptic_curve::sec1::ToEncodedPoint,
};

use crate::unid::errors::UNiDError;

pub struct Secp256k1 {}

impl Secp256k1 {
    pub fn generate_public_key(private_key: &[u8]) -> Result<Vec<u8>, UNiDError> {
        let signing_key = match SigningKey::from_bytes(private_key.to_vec().as_slice()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok(signing_key.verifying_key().to_bytes().to_vec())
    }

    #[allow(dead_code)]
    pub fn convert_public_key(public_key: &[u8], compress: bool) -> Result<Vec<u8>, UNiDError> {
        let public_key = match PublicKey::from_sec1_bytes(&public_key.to_vec()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok(public_key.to_encoded_point(compress).as_bytes().to_vec())
    }

    #[allow(dead_code)]
    pub fn ecdsa_sign(message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, UNiDError> {
        let signing_key = match SigningKey::from_bytes(private_key.to_vec().as_slice()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let signature: Signature = match signing_key.try_sign(message.to_vec().as_slice()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok(signature.as_ref().to_vec())
    }

    #[allow(dead_code)]
    pub fn ecdsa_verify(signature: &[u8], message: &[u8], public_key: &[u8]) -> Result<bool, UNiDError> {
        let verify_key = match VerifyingKey::from_sec1_bytes(&public_key.to_vec()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        if signature.len() != 64 {
            return Err(UNiDError{})
        }

        let r = GenericArray::from_slice(&signature[0..32]);
        let s = GenericArray::from_slice(&signature[32..]);

        let wrapped_signature = match Signature::from_scalars(*r, *s) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        match verify_key.verify(&message.to_vec(), &wrapped_signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn message() -> String {
        String::from("Hello, UNiD!")
    }

    #[fixture]
    fn private_key() -> Vec<u8> {
        vec![
            0x03, 0xb4, 0xad, 0xcd, 0x59, 0x36, 0x3f, 0x4e, 0xb9, 0xd0,
            0x9f, 0x2a, 0x34, 0xcd, 0x3d, 0x26, 0xa8, 0x12, 0x33, 0x0c,
            0x2f, 0x88, 0x7c, 0xe5, 0xf8, 0x53, 0x89, 0x48, 0xff, 0xac,
            0x74, 0xc0,
        ]
    }

    #[test]
    fn test_generate_public_key() {
        let result = match Secp256k1::generate_public_key(&private_key()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(vec![
            0x02, 0x51, 0x84, 0x22, 0x3f, 0xe8, 0x5d, 0x53, 0x20, 0x3c,
            0xf9, 0xd3, 0x7f, 0x68, 0x22, 0xe6, 0x33, 0xe8, 0xd7, 0x9f,
            0x48, 0xb1, 0x32, 0xdf, 0x0b, 0x8c, 0x8a, 0x64, 0x11, 0x41,
            0xf3, 0x19, 0xb6,
        ], result);
    }

    #[test]
    fn test_convert_public_key() {
        let public_key = match Secp256k1::generate_public_key(&private_key()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result_1 = match Secp256k1::convert_public_key(&public_key, true) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(vec![
            0x02, 0x51, 0x84, 0x22, 0x3f, 0xe8, 0x5d, 0x53, 0x20, 0x3c,
            0xf9, 0xd3, 0x7f, 0x68, 0x22, 0xe6, 0x33, 0xe8, 0xd7, 0x9f,
            0x48, 0xb1, 0x32, 0xdf, 0x0b, 0x8c, 0x8a, 0x64, 0x11, 0x41,
            0xf3, 0x19, 0xb6,
        ], result_1);

        let result_2 = match Secp256k1::convert_public_key(&public_key, false) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(vec![
            0x04, 0x51, 0x84, 0x22, 0x3f, 0xe8, 0x5d, 0x53, 0x20, 0x3c,
            0xf9, 0xd3, 0x7f, 0x68, 0x22, 0xe6, 0x33, 0xe8, 0xd7, 0x9f,
            0x48, 0xb1, 0x32, 0xdf, 0x0b, 0x8c, 0x8a, 0x64, 0x11, 0x41,
            0xf3, 0x19, 0xb6, 0xa3, 0x70, 0x7c, 0xa5, 0x18, 0x61, 0xe1,
            0xe2, 0xde, 0xa4, 0x3c, 0x23, 0x84, 0xf1, 0x79, 0xed, 0x44,
            0xe9, 0x8c, 0x4a, 0xd0, 0x38, 0x89, 0x21, 0xd9, 0x6a, 0x1e,
            0x05, 0x93, 0x15, 0xe7, 0x54,
        ], result_2);
    }

    #[test]
    fn test_ecdsa_sign() {
        let message = String::from(&message()).as_bytes().to_vec();

        let result = match Secp256k1::ecdsa_sign(&message, &private_key()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(vec![
            0xda, 0x69, 0x53, 0xea, 0xce, 0x56, 0x31, 0x22, 0x91, 0xc5,
            0x2b, 0x74, 0x78, 0x3b, 0xa0, 0x81, 0xe9, 0x1f, 0x8c, 0x72,
            0x77, 0x0a, 0x1c, 0x78, 0xeb, 0x8d, 0x77, 0xce, 0xb5, 0x1f,
            0x27, 0x76, 0x6e, 0x2d, 0xd7, 0x9d, 0x23, 0xca, 0x02, 0xf2,
            0x02, 0x7d, 0xf6, 0xe7, 0xea, 0xde, 0xa8, 0x0f, 0xa8, 0x08,
            0x24, 0x24, 0xda, 0xaa, 0x47, 0x9b, 0x10, 0x70, 0x7b, 0x20,
            0xd9, 0xda, 0x9f, 0x3e,
        ], result);
    }

    #[test]
    fn test_ecdsa_verify() {
        let message = String::from(&message()).as_bytes().to_vec();

        let signature = match Secp256k1::ecdsa_sign(&message, &private_key()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let public_key_compressed = match Secp256k1::generate_public_key(&private_key()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result_1 = match Secp256k1::ecdsa_verify(&signature, &message, &public_key_compressed) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert!(result_1);

        let public_key_un_compressed = match Secp256k1::convert_public_key(&public_key_compressed, false) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result_2 = match Secp256k1::ecdsa_verify(&signature, &message, &public_key_un_compressed) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert!(result_2);
    }
}