use alloc::format;
use alloc::string::String;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha512;

use crate::logger::Logger;
use crate::DEBUG_MESSAGE_HANDLER;

type HmacSha512 = Hmac<Sha512>;

pub struct Hasher {}

impl Hasher {
    /**
     */
    pub fn digest(content: String, secret: String) -> String {
        let secret_u8  = secret.as_bytes();
        let content_u8 = content.as_bytes();

        let mut mac = HmacSha512::new_from_slice(secret_u8).unwrap();

        mac.update(content_u8);

        let result = mac.finalize();

        unsafe {
            let logger = Logger::new(DEBUG_MESSAGE_HANDLER.get());

            logger.debug(format!("bytes = {:?}", result.clone().into_bytes()));
        }

        base64::encode(result.into_bytes())
    }

    /**
     */
    pub fn verify(content: String, digest: String, secret: String) -> bool {
        let _digest = Hasher::digest(content, secret);

        let left = _digest.as_bytes();
        let right = digest.as_bytes();

        left.eq(right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn it_should_hasher_digest_verify_1() {
        let data_serde: serde_json::Value = serde_json::json!({
            "a": "hello",
            "b": "world"
        });

        let data: &str = &data_serde.to_string();
        let secret: &str = "secret123";
        let digested: String = Hasher::digest(data.to_string(), secret.to_string());
        let verified: bool = Hasher::verify(data.to_string(), digested.to_string(), secret.to_string());

        assert!(verified);

        assert_eq!(
            digested,
            "OM+bDTbUVutMpKxggbcI5HvVJU+1XO1O4IM7jzE69oYKpICBbLU/PWe0ZC8icnk6O3/TdkVajVNmlpct6oRNkQ=="
                .to_string()
        );
    }
}
