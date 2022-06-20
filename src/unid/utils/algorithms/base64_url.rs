extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use data_encoding::BASE64URL_NOPAD;

use crate::unid::errors::UNiDError;

pub struct Base64Url {}

impl Base64Url {
    pub fn encode(content: &[u8]) -> String {
        BASE64URL_NOPAD.encode(&content.to_vec())
    }

    pub fn decode_as_bytes(message: &str) -> Result<Vec<u8>, UNiDError> {
        match BASE64URL_NOPAD.decode(message.as_bytes()) {
            Ok(v) => Ok(v),
            Err(_) => Err(UNiDError{})
        }
    }

    pub fn decode_as_string(message: &str) -> Result<String, UNiDError> {
        let bytes = match BASE64URL_NOPAD.decode(message.as_bytes()) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        match String::from_utf8(bytes) {
            Ok(v) => Ok(v),
            Err(_) => Err(UNiDError{})
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::unid::utils::logger::Logger;

    use super::*;
    use rstest::*;

    use alloc::string::String;
    use alloc::vec::Vec;

    #[fixture]
    fn message() -> String {
        String::from(r#"{"k":"UNiD"}"#)
    }

    #[test]
    fn test_base64url_encode() {
        let result = Base64Url::encode(&message().as_bytes().to_vec());

        let logger = Logger::new();

        assert_eq!(result, String::from("eyJrIjoiVU5pRCJ9"));
    }

    #[test]
    fn test_base64url_decode_byte() {
        let encoded = Base64Url::encode(&message().as_bytes().to_vec());
        let result = match Base64Url::decode_as_bytes(&encoded) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, Vec::from([
            123,  34, 107,  34, 58,
             34,  85,  78, 105, 68,
             34, 125
        ]));
    }

    #[test]
    fn test_base64url_decode_string() {
        let encoded = Base64Url::encode(&message().as_bytes().to_vec());
        let result = match Base64Url::decode_as_string(&encoded) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, String::from(r#"{"k":"UNiD"}"#));
    }
}