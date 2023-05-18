use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512};

use crate::nodex::errors::NodeXError;

type _HmacSha256 = Hmac<Sha256>;
type _HmacSha512 = Hmac<Sha512>;

// pub trait HmacExecutable {
//     fn digest(secret: &[u8], message: &[u8]) -> Result<Vec<u8>, NodeXError>;
//     fn verify(secret: &[u8], message: &[u8], digest: &[u8]) -> Result<bool, NodeXError>;
// }

#[allow(dead_code)]
pub struct HmacSha256 {}

impl HmacSha256 {
    #[allow(dead_code)]
    pub fn digest(secret: &[u8], message: &[u8]) -> Result<Vec<u8>, NodeXError> {
        let mut mac = match _HmacSha256::new_from_slice(secret) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError {}),
        };

        mac.update(message);

        Ok(mac.finalize().into_bytes().to_vec())
    }

    #[allow(dead_code)]
    pub fn verify(secret: &[u8], message: &[u8], digest: &[u8]) -> Result<bool, NodeXError> {
        let computed = match HmacSha256::digest(secret, message) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError {}),
        };

        Ok(computed.eq(digest))
    }
}

#[allow(dead_code)]
pub struct HmacSha512 {}

impl HmacSha512 {
    #[allow(dead_code)]
    pub fn digest(secret: &[u8], message: &[u8]) -> Result<Vec<u8>, NodeXError> {
        let mut mac = match _HmacSha512::new_from_slice(secret) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError {}),
        };

        mac.update(message);

        Ok(mac.finalize().into_bytes().to_vec())
    }

    #[allow(dead_code)]
    pub fn verify(secret: &[u8], message: &[u8], digest: &[u8]) -> Result<bool, NodeXError> {
        let computed = match HmacSha512::digest(secret, message) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError {}),
        };

        Ok(computed.eq(digest))
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

    fn secret() -> String {
        String::from(r#"secret"#)
    }

    #[test]
    fn test_hmac_sha256_digest() {
        let result = HmacSha256::digest(secret().as_bytes(), message().as_bytes());

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Vec::from([
                0xb6, 0x3f, 0x68, 0x0e, 0x14, 0x87, 0xa7, 0x55, 0x0b, 0x1b, 0x8a, 0x39, 0x27, 0x4d,
                0x6b, 0x4f, 0xd7, 0x1b, 0x1e, 0xe1, 0x14, 0xec, 0xef, 0x07, 0xb3, 0x59, 0xfb, 0xa4,
                0x10, 0x45, 0x3e, 0x60,
            ])
        );
    }

    #[test]
    fn test_hmac_sha256_verify() {
        let digest = Vec::from([
            0xb6, 0x3f, 0x68, 0x0e, 0x14, 0x87, 0xa7, 0x55, 0x0b, 0x1b, 0x8a, 0x39, 0x27, 0x4d,
            0x6b, 0x4f, 0xd7, 0x1b, 0x1e, 0xe1, 0x14, 0xec, 0xef, 0x07, 0xb3, 0x59, 0xfb, 0xa4,
            0x10, 0x45, 0x3e, 0x60,
        ]);

        let result = HmacSha256::verify(secret().as_bytes(), message().as_bytes(), &digest);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_hmac_sha512_digest() {
        let result = HmacSha512::digest(secret().as_bytes(), message().as_bytes());

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Vec::from([
                0x56, 0x60, 0x97, 0x8a, 0xef, 0x9a, 0xaa, 0xf1, 0x87, 0x46, 0xb8, 0xff, 0x03, 0x71,
                0x37, 0x70, 0xc2, 0xbd, 0x49, 0x1b, 0x90, 0xe1, 0x72, 0xa9, 0x47, 0xee, 0x54, 0x17,
                0xb9, 0x27, 0x0c, 0x3c, 0x21, 0x42, 0x62, 0x04, 0xd4, 0x48, 0xb1, 0x02, 0xfa, 0x0d,
                0x80, 0x2b, 0xe4, 0xc6, 0xfd, 0x11, 0xf7, 0x98, 0x4c, 0x6e, 0x85, 0x22, 0xca, 0xa1,
                0x38, 0x38, 0xb5, 0xe9, 0x0d, 0x75, 0x34, 0xe7,
            ])
        );
    }

    #[test]
    fn test_hmac_sha512_verify() {
        let digest = Vec::from([
            0x56, 0x60, 0x97, 0x8a, 0xef, 0x9a, 0xaa, 0xf1, 0x87, 0x46, 0xb8, 0xff, 0x03, 0x71,
            0x37, 0x70, 0xc2, 0xbd, 0x49, 0x1b, 0x90, 0xe1, 0x72, 0xa9, 0x47, 0xee, 0x54, 0x17,
            0xb9, 0x27, 0x0c, 0x3c, 0x21, 0x42, 0x62, 0x04, 0xd4, 0x48, 0xb1, 0x02, 0xfa, 0x0d,
            0x80, 0x2b, 0xe4, 0xc6, 0xfd, 0x11, 0xf7, 0x98, 0x4c, 0x6e, 0x85, 0x22, 0xca, 0xa1,
            0x38, 0x38, 0xb5, 0xe9, 0x0d, 0x75, 0x34, 0xe7,
        ]);

        let result = HmacSha512::verify(secret().as_bytes(), message().as_bytes(), &digest);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
