use sha2::{Digest, Sha256};

use crate::unid::runtime::base64_url::{Base64Url, PaddingType};
use crate::unid::errors::UNiDError;

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

        Base64Url::encode(&hashed, &PaddingType::NoPadding)
    }

    pub fn canonicalize_then_double_hash_then_encode(message: &[u8]) -> Result<String, UNiDError> {
        let plain = match String::from_utf8(message.to_vec()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };
        let canonicalized = match super::jcs::JCS::canonicalize(&plain) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let hashed = Multihash::hash_as_non_multihash_buffer(&canonicalized.as_bytes());

        Ok(Multihash::hash_then_encode(&hashed))
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

    #[fixture]
    fn message() -> String {
        String::from(r#"{"k":"UNiD"}"#)
    }

    #[test]
    fn test_hash() {
        let result = Multihash::hash(&message().as_bytes().to_vec());

        assert_eq!(result, vec![
            0x12, 0x20, 0x95, 0xfb, 0x14, 0x75, 0x45, 0xe0, 0xf9, 0x96,
            0x3d, 0x71, 0x28, 0xb3, 0x86, 0x8d, 0x18, 0x65, 0x24, 0x8a,
            0x88, 0x3f, 0x5e, 0xcf, 0x8e, 0xe9, 0xbf, 0x08, 0xb5, 0x63,
            0x46, 0xff, 0x4a, 0x0c,
        ]);
    }

    #[test]
    fn test_hash_as_non_multihash_buffer() {
        let result = Multihash::hash_as_non_multihash_buffer(&message().as_bytes().to_vec());

        assert_eq!(result, vec![
            0x95, 0xfb, 0x14, 0x75, 0x45, 0xe0, 0xf9, 0x96, 0x3d, 0x71,
            0x28, 0xb3, 0x86, 0x8d, 0x18, 0x65, 0x24, 0x8a, 0x88, 0x3f,
            0x5e, 0xcf, 0x8e, 0xe9, 0xbf, 0x08, 0xb5, 0x63, 0x46, 0xff,
            0x4a, 0x0c,
        ]);
    }

    #[test]
    fn test_canonicalize_then_double_hash_then_encode() {
        let result = match Multihash::canonicalize_then_double_hash_then_encode(&message().as_bytes().to_vec()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, String::from("EiAkB6db3wB049pqz8eml0uwHzIJOEadoAOFPgyNhXFdmw"));
    }

    #[test]
    fn test_hash_then_encode() {
        let result = Multihash::hash_then_encode(&message().as_bytes().to_vec());

        assert_eq!(result, String::from("EiCV-xR1ReD5lj1xKLOGjRhlJIqIP17Pjum_CLVjRv9KDA"));
    }

    #[test]
    fn test_decode() {
        let encoded = Multihash::hash(&message().as_bytes().to_vec());
        let result = Multihash::decode(&encoded);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DecodedContainer {
            hash: vec![
                0x95, 0xfb, 0x14, 0x75, 0x45, 0xe0, 0xf9, 0x96, 0x3d, 0x71,
                0x28, 0xb3, 0x86, 0x8d, 0x18, 0x65, 0x24, 0x8a, 0x88, 0x3f,
                0x5e, 0xcf, 0x8e, 0xe9, 0xbf, 0x08, 0xb5, 0x63, 0x46, 0xff,
                0x4a, 0x0c,
            ],
            algorithm: 18,
        });
    }
}