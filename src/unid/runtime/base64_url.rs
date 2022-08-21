use data_encoding::{ BASE64URL_NOPAD, BASE64URL };

use crate::unid::errors::UNiDError;

pub struct Base64Url {}

pub enum PaddingType {
    Padding,
    NoPadding,
}

impl Base64Url {
    pub fn encode(content: &[u8], padding: &PaddingType) -> String {
        match padding {
            PaddingType::Padding => {
                BASE64URL.encode(&content.to_vec())
            },
            PaddingType::NoPadding => {
                BASE64URL_NOPAD.encode(&content.to_vec())
            },
        }
    }

    pub fn decode_as_bytes(message: &str, padding: &PaddingType) -> Result<Vec<u8>, UNiDError> {
        match padding {
            PaddingType::Padding => {
                match BASE64URL.decode(message.as_bytes()) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(UNiDError{})
                }
            },
            PaddingType::NoPadding => {
                match BASE64URL_NOPAD.decode(message.as_bytes()) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(UNiDError{})
                }
            },
        }
    }

    pub fn decode_as_string(message: &str, padding: &PaddingType) -> Result<String, UNiDError> {
        let bytes = match padding {
            PaddingType::Padding => {
                match BASE64URL.decode(message.as_bytes()) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{})
                }
            },
            PaddingType::NoPadding => {
                match BASE64URL_NOPAD.decode(message.as_bytes()) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{})
                }
            },
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
        String::from("Hello, UNiD !")
    }

    #[test]
    fn test_base64url_encode() {
        let result = Base64Url::encode(&message().as_bytes().to_vec(), &PaddingType::Padding);

        assert_eq!(result, String::from("SGVsbG8sIFVOaUQgIQ=="));
    }

    #[test]
    fn test_base64url_encode_nopad() {
        let result = Base64Url::encode(&message().as_bytes().to_vec(), &PaddingType::NoPadding);

        assert_eq!(result, String::from("SGVsbG8sIFVOaUQgIQ"));
    }

    #[test]
    fn test_base64url_decode_byte() {
        let encoded = Base64Url::encode(&message().as_bytes().to_vec(), &PaddingType::Padding);
        let result = match Base64Url::decode_as_bytes(&encoded, &PaddingType::Padding) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, vec![
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x55, 0x4e, 0x69,
            0x44, 0x20, 0x21,
        ]);
    }

    #[test]
    fn test_base64url_decode_byte_nopad() {
        let encoded = Base64Url::encode(&message().as_bytes().to_vec(), &PaddingType::NoPadding);
        let result = match Base64Url::decode_as_bytes(&encoded, &PaddingType::NoPadding) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, vec![
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x55, 0x4e, 0x69,
            0x44, 0x20, 0x21,
        ]);
    }

    #[test]
    fn test_base64url_decode_string() {
        let encoded = Base64Url::encode(&message().as_bytes().to_vec(), &PaddingType::Padding);
        let result = match Base64Url::decode_as_string(&encoded, &PaddingType::Padding) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, message());
    }

    #[test]
    fn test_base64url_decode_string_nopad() {
        let encoded = Base64Url::encode(&message().as_bytes().to_vec(), &PaddingType::NoPadding);
        let result = match Base64Url::decode_as_string(&encoded, &PaddingType::NoPadding) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, message());
    }
}