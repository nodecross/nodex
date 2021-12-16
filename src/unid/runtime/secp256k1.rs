use alloc::vec::Vec;
use k256::{
    ecdsa::{SigningKey, Signature, signature::{Signer}},
};

use crate::unid::errors::UNiDError;

pub struct Secp256k1 {}

const COMPRESSED_PUBLIC_KEY_SIZE: usize = 33; // Buffer(0x04 + PublicKey (32 = 256 bit))
const UNCOMPRESSED_PUBLIC_KEY_SIZE: usize = 65; // Buffer(0x04 + PublicKey (64 = 512 bit))

impl Secp256k1 {
    pub fn public_key_convert(public_key: &Vec<u8>, compressed: bool) -> Vec<u8> {
        //return secp256k1.publicKeyConvert(publicKey, compressed)
        let x: Vec<u8> = Vec::from([]);

        x
    }

    pub fn ecdsa_sign(message: &Vec<u8>, private_key: &Vec<u8>) -> Result<Vec<u8>, UNiDError> {
        let signing_key = match SigningKey::from_bytes(&private_key.as_slice()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let signature: Signature = match signing_key.try_sign(&message.as_slice()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok(signature.as_ref().to_vec())
    }

    pub fn ecdsa_verify(signature: &Vec<u8>, message: &Vec<u8>, public_key: &Vec<u8>) -> bool {
        //return secp256k1.ecdsaVerify(signature, message, publicKey)

        true
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use crate::unid::utils::random;

    #[test]
    fn test_public_key_convert() {
    }

    #[test]
    fn test_ecdsa_sign() {
        let message = String::from("").as_bytes().to_vec();
        let private_key = random::Random::bytes(&32);

        let result = Secp256k1::ecdsa_sign(&message, &private_key);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), Vec::from([0]));
    }

    #[test]
    fn test_ecdsa_verify() {
    }
}