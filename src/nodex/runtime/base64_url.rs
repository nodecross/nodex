use data_encoding::{BASE64URL, BASE64URL_NOPAD};
use thiserror::Error;

pub struct Base64Url {}

pub enum PaddingType {
    #[allow(dead_code)]
    Padding,
    NoPadding,
}

#[derive(Error, Debug)]
pub enum Base64UrlError {
    #[error("decode failed")]
    DecodeFailed(#[from] data_encoding::DecodeError),
    #[error("convert from utf8 failed")]
    ConvertFromUtf8Error(#[from] std::string::FromUtf8Error),
}

impl Base64Url {
    pub fn encode(content: &[u8], padding: &PaddingType) -> String {
        match padding {
            PaddingType::Padding => BASE64URL.encode(content),
            PaddingType::NoPadding => BASE64URL_NOPAD.encode(content),
        }
    }

    pub fn decode_as_bytes(
        message: &str,
        padding: &PaddingType,
    ) -> Result<Vec<u8>, Base64UrlError> {
        (match padding {
            PaddingType::Padding => BASE64URL.decode(message.as_bytes()),
            PaddingType::NoPadding => BASE64URL_NOPAD.decode(message.as_bytes()),
        })
        .map_err(|e| e.into())
    }

    pub fn decode_as_string(
        message: &str,
        padding: &PaddingType,
    ) -> Result<String, Base64UrlError> {
        let bytes = (match padding {
            PaddingType::Padding => BASE64URL.decode(message.as_bytes()),
            PaddingType::NoPadding => BASE64URL_NOPAD.decode(message.as_bytes()),
        })?;

        String::from_utf8(bytes).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn message() -> String {
        String::from("0123456789abcdef")
    }

    #[test]
    fn test_base64url_encode() {
        let result = Base64Url::encode(message().as_bytes(), &PaddingType::Padding);

        assert_eq!(result, String::from("MDEyMzQ1Njc4OWFiY2RlZg=="));
    }

    #[test]
    fn test_base64url_encode_nopad() {
        let result = Base64Url::encode(message().as_bytes(), &PaddingType::NoPadding);

        assert_eq!(result, String::from("MDEyMzQ1Njc4OWFiY2RlZg"));
    }

    #[test]
    fn test_base64url_decode_byte() {
        let encoded = Base64Url::encode(message().as_bytes(), &PaddingType::Padding);
        let result = match Base64Url::decode_as_bytes(&encoded, &PaddingType::Padding) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(
            result,
            vec![
                0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63, 0x64,
                0x65, 0x66,
            ]
        );
    }

    #[test]
    fn test_base64url_decode_byte_nopad() {
        let encoded = Base64Url::encode(message().as_bytes(), &PaddingType::NoPadding);
        let result = match Base64Url::decode_as_bytes(&encoded, &PaddingType::NoPadding) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(
            result,
            vec![
                0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63, 0x64,
                0x65, 0x66,
            ]
        );
    }

    #[test]
    fn test_base64url_decode_string() {
        let encoded = Base64Url::encode(message().as_bytes(), &PaddingType::Padding);
        let result = match Base64Url::decode_as_string(&encoded, &PaddingType::Padding) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result, message());
    }

    #[test]
    fn test_base64url_decode_string_nopad() {
        let encoded = Base64Url::encode(message().as_bytes(), &PaddingType::NoPadding);
        let result = match Base64Url::decode_as_string(&encoded, &PaddingType::NoPadding) {
            Ok(v) => v,
            Err(_) => panic!(),
        };

        assert_eq!(result, message());
    }
}
