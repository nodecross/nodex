use crate::nodex::{errors::NodeXError, runtime};

#[allow(dead_code)]
pub struct Hasher {}

impl Hasher {
    #[allow(dead_code)]
    pub fn digest(message: &[u8], secret: &[u8]) -> Result<String, NodeXError> {
        let digest = match runtime::hmac::HmacSha512::digest(secret, message) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError {}),
        };

        Ok(hex::encode(digest))
    }

    #[allow(dead_code)]
    pub fn verify(message: &[u8], digest: &[u8], secret: &[u8]) -> Result<bool, NodeXError> {
        let _digest = match hex::decode(digest) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError {}),
        };

        match runtime::hmac::HmacSha512::verify(secret, message, &_digest) {
            Ok(v) => Ok(v),
            Err(_) => Err(NodeXError {}),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn message() -> String {
        String::from("0123456789abcdef")
    }

    #[fixture]
    fn secret() -> String {
        String::from("secret")
    }

    #[fixture]
    fn digest() -> String {
        String::from("63aa09cf68fc541786276834496e96e57d2d9b41a0cbd8ce1a9f9f9478bfd5764fc8013bf47635a64367de6385b941d67662379b97cef7c0629ee871834472a8")
    }

    #[test]
    pub fn test_digest() {
        let result = match Hasher::digest(message().as_bytes(), secret().as_bytes()) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result, digest())
    }

    #[test]
    pub fn test_verify() {
        let result = match Hasher::verify(
            message().as_bytes(),
            digest().as_bytes(),
            secret().as_bytes(),
        ) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert!(result)
    }
}
