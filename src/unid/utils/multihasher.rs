use alloc::vec::Vec;
use alloc::string::String;
use sha2::{Digest, Sha256};

use crate::unid::errors::UNiDError;

use super::codec;

const MULTIHASH_SHA256_CODE: u8 = 0x12; // 0x12 = 18
const MULTIHASH_SHA256_SIZE: u8 = 0x20; // 0x20 = 32

pub struct Multihash {}

#[derive(PartialEq, Debug)]
pub struct DecodedContainer {
    hash: Vec<u8>,
    algorithm: u64,
}

impl Multihash {
    pub fn hash_as_non_multihash_buffer(message: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();

        hasher.update(message);

        hasher.finalize().to_vec()
    }

    // [NOTE]: SHA2-256 ONLY
    pub fn hash(message: &[u8]) -> Vec<u8> {
        let mut prefix: Vec<u8> = Vec::from([
            MULTIHASH_SHA256_CODE,
            MULTIHASH_SHA256_SIZE,
        ]);
        let mut hashed: Vec<u8> = Multihash::hash_as_non_multihash_buffer(message);
        let mut joined: Vec<u8> = Vec::from([]);

        joined.append(&mut prefix);
        joined.append(&mut hashed);

        joined
    }

    pub fn hash_then_encode(message: &[u8]) -> String {
        let hashed = Multihash::hash(message);

        codec::Base64Url::encode(&hashed)
    }

    pub fn canonicalize_then_double_hash_then_encode(message: &[u8]) -> String {
        // [FIXME]: SHOLD CANONICALIZE
        let canonicalized = message;

        let hashed = Multihash::hash_as_non_multihash_buffer(canonicalized);

        Multihash::hash_then_encode(&hashed)
    }

    #[allow(dead_code)]
    pub fn decode(encoded: &[u8]) -> Result<DecodedContainer, UNiDError> {
        // check for: [ code, size, digest... ]
        if encoded.len() < 2 {
            return Err(UNiDError{});
        }

        let code = encoded[0];
        let length = encoded[1];
        let digest = encoded[2..].to_vec();

        if digest.len() != usize::from(length) {
            return Err(UNiDError{})
        }

        Ok(DecodedContainer {
            hash: digest,
            algorithm: u64::from(code),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    use alloc::string::String;
    use alloc::vec::Vec;

    #[fixture]
    fn message() -> String {
        String::from(r#"{"k":"UNiD"}"#)
    }

    #[test]
    fn test_hash() {
        let result = Multihash::hash(&message().as_bytes().to_vec());

        assert_eq!(result, Vec::from([
             18,  32, 149, 251,  20, 117,  69, 224,
            249, 150,  61, 113,  40, 179, 134, 141,
             24, 101,  36, 138, 136,  63,  94, 207,
            142, 233, 191,   8, 181,  99,  70, 255,
             74,  12
        ]));
    }

    #[test]
    fn test_hash_as_non_multihash_buffer() {
        let result = Multihash::hash_as_non_multihash_buffer(&message().as_bytes().to_vec());

        assert_eq!(result, Vec::from([
           149, 251,  20, 117,  69, 224, 249, 150,
            61, 113,  40, 179, 134, 141,  24, 101,
            36, 138, 136,  63,  94, 207, 142, 233,
           191,   8, 181,  99,  70, 255,  74,  12
        ]));
    }

    #[test]
    fn test_canonicalize_then_double_hash_then_encode() {
        let result = Multihash::canonicalize_then_double_hash_then_encode(&message().as_bytes().to_vec());

        assert_eq!(result, String::from("EiAkB6db3wB049pqz8eml0uwHzIJOEadoAOFPgyNhXFdmw"));
        //assert_eq!(result, String::from("EiAkB6db3wB049pqz8eml0uwHzIJOEadoAOFPgyNhXFdmw=="));
    }

    #[test]
    fn test_hash_then_encode() {
        let result = Multihash::hash_then_encode(&message().as_bytes().to_vec());
        assert_eq!(result, String::from("EiCV-xR1ReD5lj1xKLOGjRhlJIqIP17Pjum_CLVjRv9KDA"));

        //assert_eq!(result, String::from("EiCV-xR1ReD5lj1xKLOGjRhlJIqIP17Pjum_CLVjRv9KDA=="));
    }

    #[test]
    fn test_decode() {
        let encoded = Multihash::hash(&message().as_bytes().to_vec());
        let result = Multihash::decode(&encoded);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DecodedContainer {
            hash: vec![
               149, 251,  20, 117,  69, 224, 249, 150,
                61, 113,  40, 179, 134, 141,  24, 101,
                36, 138, 136,  63,  94, 207, 142, 233,
               191,   8, 181,  99,  70, 255,  74,  12
            ],
            algorithm: 18,
        });
    }
}