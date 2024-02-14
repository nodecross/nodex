use crate::nodex::runtime::base64_url::{Base64Url, PaddingType};
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::jcs::JcsError;

const MULTIHASH_SHA256_CODE: u8 = 0x12; // 0x12 = 18
const MULTIHASH_SHA256_SIZE: u8 = 0x20; // 0x20 = 32

pub struct Multihash {}

#[derive(Eq, PartialEq, Debug)]
pub struct DecodedContainer {
    hash: Vec<u8>,
    algorithm: u64,
}

#[derive(Debug, Error)]
pub enum MultihashError {
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    JsonCanonicalizationError(#[from] JcsError),
    #[error("InvalidLength: {0}")]
    InvalidLength(usize),
    #[error("expected length is {0}, but actual length is {1}")]
    SizeValidationFailed(usize, usize),
}

impl Multihash {
    pub fn hash_as_non_multihash_buffer(message: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();

        hasher.update(message);

        hasher.finalize().to_vec()
    }

    // [NOTE]: SHA2-256 ONLY
    pub fn hash(message: &[u8]) -> Vec<u8> {
        let mut prefix: Vec<u8> = Vec::from([MULTIHASH_SHA256_CODE, MULTIHASH_SHA256_SIZE]);

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

    pub fn canonicalize_then_double_hash_then_encode(
        message: &[u8],
    ) -> Result<String, MultihashError> {
        let plain = String::from_utf8(message.to_vec())?;

        let canonicalized = super::jcs::Jcs::canonicalize(&plain)?;

        let hashed = Multihash::hash_as_non_multihash_buffer(canonicalized.as_bytes());

        Ok(Multihash::hash_then_encode(&hashed))
    }

    #[allow(dead_code)]
    pub fn decode(encoded: &[u8]) -> Result<DecodedContainer, MultihashError> {
        // check for: [ code, size, digest... ]
        if encoded.len() < 2 {
            return Err(MultihashError::InvalidLength(encoded.len()));
        }

        let code = encoded[0];
        let length = encoded[1];
        let digest = encoded[2..].to_vec();

        if digest.len() != usize::from(length) {
            return Err(MultihashError::SizeValidationFailed(
                usize::from(length),
                digest.len(),
            ));
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
        String::from(r#"{"k":"0123456789abcdef"}"#)
    }

    #[test]
    fn test_hash() {
        let result = Multihash::hash(message().as_bytes());

        assert_eq!(
            result,
            vec![
                0x12, 0x20, 0x5f, 0x46, 0x25, 0xd4, 0xf6, 0x1e, 0xdb, 0x52, 0x78, 0x07, 0x45, 0x5f,
                0x48, 0xf8, 0xbe, 0x27, 0x8e, 0x71, 0xe8, 0x4a, 0xa9, 0x4d, 0x23, 0x11, 0x1f, 0xfa,
                0xb3, 0xb6, 0x30, 0x93, 0xa7, 0x13,
            ]
        );
    }

    #[test]
    fn test_hash_as_non_multihash_buffer() {
        let result = Multihash::hash_as_non_multihash_buffer(message().as_bytes());

        assert_eq!(
            result,
            vec![
                0x5f, 0x46, 0x25, 0xd4, 0xf6, 0x1e, 0xdb, 0x52, 0x78, 0x07, 0x45, 0x5f, 0x48, 0xf8,
                0xbe, 0x27, 0x8e, 0x71, 0xe8, 0x4a, 0xa9, 0x4d, 0x23, 0x11, 0x1f, 0xfa, 0xb3, 0xb6,
                0x30, 0x93, 0xa7, 0x13,
            ]
        );
    }

    #[test]
    fn test_canonicalize_then_double_hash_then_encode() {
        let result =
            match Multihash::canonicalize_then_double_hash_then_encode(message().as_bytes()) {
                Ok(v) => v,
                Err(_) => panic!(),
            };

        assert_eq!(
            result,
            String::from("EiAEX1W46vVid7IjJyFY5ibjmyrgepTjW0rYrw-wo4xLCw")
        );
    }

    #[test]
    fn test_hash_then_encode() {
        let result = Multihash::hash_then_encode(message().as_bytes());

        assert_eq!(
            result,
            String::from("EiBfRiXU9h7bUngHRV9I-L4njnHoSqlNIxEf-rO2MJOnEw")
        );
    }

    #[test]
    fn test_decode() {
        let encoded = Multihash::hash(message().as_bytes());
        let result = Multihash::decode(&encoded);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DecodedContainer {
                hash: vec![
                    0x5f, 0x46, 0x25, 0xd4, 0xf6, 0x1e, 0xdb, 0x52, 0x78, 0x07, 0x45, 0x5f, 0x48,
                    0xf8, 0xbe, 0x27, 0x8e, 0x71, 0xe8, 0x4a, 0xa9, 0x4d, 0x23, 0x11, 0x1f, 0xfa,
                    0xb3, 0xb6, 0x30, 0x93, 0xa7, 0x13,
                ],
                algorithm: 18,
            }
        );
    }
}
