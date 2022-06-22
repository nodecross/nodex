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
            3  , 180, 173, 205, 89 , 54, 63 , 78 , 185, 208,
            159, 42 , 52 , 205, 61 , 38, 168, 18 , 51 , 12 ,
            47 , 136, 124, 229, 248, 83, 137, 72 , 255, 172,
            116, 192,
        ]
    }

    #[test]
    fn test_generate_public_key() {
        let result = match Secp256k1::generate_public_key(&private_key()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(vec![
            2  , 81 , 132, 34 , 63, 232, 93 , 83 , 32 , 60 ,
            249, 211, 127, 104, 34, 230, 51 , 232, 215, 159,
            72 , 177, 50 , 223, 11, 140, 138, 100, 17 , 65 ,
            243, 25 , 182,
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
            2  , 81 , 132, 34 , 63, 232, 93 , 83 , 32 , 60 ,
            249, 211, 127, 104, 34, 230, 51 , 232, 215, 159,
            72 , 177, 50 , 223, 11, 140, 138, 100, 17 , 65 ,
            243, 25 , 182,
        ], result_1);

        let result_2 = match Secp256k1::convert_public_key(&public_key, false) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(vec![
            4  , 81 , 132, 34 , 63 , 232, 93 , 83 , 32 , 60 ,
            249, 211, 127, 104, 34 , 230, 51 , 232, 215, 159,
            72 , 177, 50 , 223, 11 , 140, 138, 100, 17 , 65 ,
            243, 25 , 182, 163, 112, 124, 165, 24 , 97 , 225,
            226, 222, 164, 60 , 35 , 132, 241, 121, 237, 68 ,
            233, 140, 74 , 208, 56 , 137, 33 , 217, 106, 30 ,
            5  , 147, 21 , 231, 84,
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
            218, 105, 83 , 234, 206, 86 , 49 , 34 , 145, 197,
            43 , 116, 120, 59 , 160, 129, 233, 31 , 140, 114,
            119, 10 , 28 , 120, 235, 141, 119, 206, 181, 31 ,
            39 , 118, 110, 45 , 215, 157, 35 , 202, 2  , 242,
            2  , 125, 246, 231, 234, 222, 168, 15 , 168, 8  ,
            36 , 36 , 218, 170, 71 , 155, 16 , 112, 123, 32 ,
            217, 218, 159, 62 ,
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