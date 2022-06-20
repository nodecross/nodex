use alloc::vec::Vec;
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

    use alloc::vec::Vec;
    use crate::unid::utils::random;

    #[fixture]
    fn message() -> String {
        String::from(r#"{"k":"UNiD"}"#)
    }

    #[test]
    fn test_generate_public_key() {
        let private_key = random::Random::bytes(&32).unwrap();

        let result = Secp256k1::generate_public_key(&private_key);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::from([
            3  , 180, 173, 205, 89 , 54, 63 , 78 , 185, 208,
            159, 42 , 52 , 205, 61 , 38, 168, 18 , 51 , 12 ,
            47 , 136, 124, 229, 248, 83, 137, 72 , 255, 172,
            116, 192, 115,
        ]));
    }

    #[test]
    fn test_convert_public_key() {
        let private_key = random::Random::bytes(&32).unwrap();

        let public_key = match Secp256k1::generate_public_key(&private_key) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result_1 = Secp256k1::convert_public_key(&public_key, true);

        assert!(result_1.is_ok());
        assert_eq!(result_1.unwrap(), Vec::from([
            3  , 180, 173, 205, 89 , 54, 63 , 78 , 185, 208,
            159, 42 , 52 , 205, 61 , 38, 168, 18 , 51 , 12 ,
            47 , 136, 124, 229, 248, 83, 137, 72 , 255, 172,
            116, 192, 115,
        ]));

        let result_2 = Secp256k1::convert_public_key(&public_key, false);

        assert!(result_2.is_ok());
        assert_eq!(result_2.unwrap(), Vec::from([
            4  , 180, 173, 205, 89 , 54 , 63 , 78 , 185, 208,
            159, 42 , 52 , 205, 61 , 38 , 168, 18 , 51 , 12 ,
            47 , 136, 124, 229, 248, 83 , 137, 72 , 255, 172,
            116, 192, 115, 214, 157, 80 , 18 , 144, 108, 254,
            87 , 41 , 168, 219, 148, 205, 146, 42 , 227, 197,
            31 , 159, 254, 46 , 109, 174, 44 , 141, 134, 85 ,
            162, 177, 119, 86 , 217,
        ]));
    }

    #[test]
    fn test_ecdsa_sign() {
        let message = String::from(&message()).as_bytes().to_vec();
        let private_key = random::Random::bytes(&32).unwrap();

        let result = Secp256k1::ecdsa_sign(&message, &private_key);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::from([
            38 , 44 , 74 , 233, 147, 222, 97 , 147, 130, 254,
            238, 192, 164, 25 , 148, 168, 187, 153, 212, 238,
            228, 247, 252, 242, 164, 130, 102, 26 , 48 , 153,
            133, 55 , 21 , 79 , 128, 113, 175, 160, 236, 157,
            66 , 230, 183, 12 , 111, 38 , 38 , 130, 118, 34 ,
            226, 168, 10 , 139, 11 , 220, 151, 253, 132, 127,
            188, 15 , 33 , 29 ,
        ]));
    }

    #[test]
    fn test_ecdsa_verify() {
        let message = String::from(&message()).as_bytes().to_vec();
        let private_key = random::Random::bytes(&32).unwrap();

        let signature = match Secp256k1::ecdsa_sign(&message, &private_key) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let public_key_compressed = match Secp256k1::generate_public_key(&private_key) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result_1 = Secp256k1::ecdsa_verify(&signature, &message, &public_key_compressed);

        assert!(result_1.is_ok());
        assert!(result_1.unwrap());

        let public_key_un_compressed = match Secp256k1::convert_public_key(&public_key_compressed, false) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result_2 = Secp256k1::ecdsa_verify(&signature, &message, &public_key_un_compressed);

        assert!(result_2.is_ok());
        assert!(result_2.unwrap());
    }
}