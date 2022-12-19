use crate::unid::{runtime, errors::UNiDError};

pub struct Hasher {}

impl Hasher {
    pub fn digest(message: &[u8], secret: &[u8]) -> Result<String, UNiDError> {
        let digest = match runtime::hmac::HmacSha512::digest(&secret, &message) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok(hex::encode(&digest))
    }

    pub fn verify(message: &[u8], digest: &[u8], secret: &[u8]) -> Result<bool, UNiDError> {
        let _digest = match hex::decode(&digest) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        match runtime::hmac::HmacSha512::verify(&secret, &message, &_digest) {
            Ok(v) => Ok(v),
            Err(_) => Err(UNiDError{})
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn message() -> String {
        String::from("Hello, UNiD!")
    }

    #[fixture]
    fn secret() -> String {
        String::from("secret")
    }

    #[fixture]
    fn digest() -> String {
        String::from("4ad3af2cb62dce43e46b0852af6109ef0447d45c7b3f17e482cccf5fc5c8de1ef22eb170431f9d91bb9d62253679111cf1cbd896466dc4e184ee6ff438289fe5")
    }

    #[test]
    pub fn test_digest() {
        let result = match Hasher::digest(&message().as_bytes(), &secret().as_bytes()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, digest())
    }

    #[test]
    pub fn test_verify() {
        let result = match Hasher::verify(&message().as_bytes(), &digest().as_bytes(), &secret().as_bytes()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, true)
    }
}