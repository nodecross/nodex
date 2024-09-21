use std::convert::TryInto;

use data_encoding::BASE64URL_NOPAD;
use sha2::{Digest, Sha256};

const MULTIHASH_SHA256_CODE: u8 = 0x12; // 0x12 = 18

// [NOTE]: SHA2-256 ONLY
pub fn hash(message: &[u8]) -> Vec<u8> {
    let mut prefix = Vec::from([MULTIHASH_SHA256_CODE]);
    let mut hashed = Sha256::digest(message).to_vec();
    prefix.push(hashed.len().try_into().unwrap());
    prefix.append(&mut hashed);
    prefix
}

pub fn double_hash_encode(message: &[u8]) -> String {
    let mes = Sha256::digest(message).to_vec();
    let mes = hash(&mes);
    BASE64URL_NOPAD.encode(&mes)
}

pub fn hash_encode(message: &[u8]) -> String {
    let mes = hash(message);
    BASE64URL_NOPAD.encode(&mes)
}

#[cfg(test)]
mod tests {

    use super::*;

    fn message() -> String {
        String::from(r#"{"k":"0123456789abcdef"}"#)
    }

    #[test]
    fn test_hash() {
        let result = hash(message().as_bytes());
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
    fn test_double_hash_then_encode() {
        let result = double_hash_encode(message().as_bytes());
        assert_eq!(result, String::from("EiAEX1W46vVid7IjJyFY5ibjmyrgepTjW0rYrw-wo4xLCw"));
    }

    #[test]
    fn test_hash_then_encode() {
        let result = hash_encode(message().as_bytes());
        assert_eq!(result, String::from("EiBfRiXU9h7bUngHRV9I-L4njnHoSqlNIxEf-rO2MJOnEw"));
    }
}
