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
    use super::*;
    use rstest::*;

    #[fixture]
    fn message() -> String {
        String::from("Hello, UNiD!")
    }

    #[test]
    fn test_base64url_encode() {
        let result = Base64Url::encode(&message().as_bytes().to_vec());

        assert_eq!(result, String::from("SGVsbG8sIFVOaUQh"));
    }

    #[test]
    fn test_base64url_decode_byte() {
        let encoded = Base64Url::encode(&message().as_bytes().to_vec());
        let result = match Base64Url::decode_as_bytes(&encoded) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, vec![
            72, 101, 108, 108, 111,
            44,  32,  85,  78, 105,
            68,  33,
        ]);
    }

    #[test]
    fn test_base64url_decode_string() {
        let encoded = Base64Url::encode(&message().as_bytes().to_vec());
        let result = match Base64Url::decode_as_string(&encoded) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, message());
    }
}