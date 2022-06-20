use sha2::{Sha256, Sha512};
use hmac::{Hmac, Mac, NewMac};
use alloc::vec::Vec;

use crate::unid::errors::UNiDError;

type _HmacSha256 = Hmac<Sha256>;
type _HmacSha512 = Hmac<Sha512>;

// pub trait HmacExecutable {
//     fn digest(secret: &[u8], message: &[u8]) -> Result<Vec<u8>, UNiDError>;
//     fn verify(secret: &[u8], message: &[u8], digest: &[u8]) -> Result<bool, UNiDError>;
// }

#[allow(dead_code)]
pub struct HmacSha256 {}

impl HmacSha256 {
    #[allow(dead_code)]
    pub fn digest(secret: &[u8], message: &[u8]) -> Result<Vec<u8>, UNiDError> {
        let mut mac = match _HmacSha256::new_from_slice(secret) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        mac.update(message);

        Ok(mac.finalize().into_bytes().to_vec())
    }

    #[allow(dead_code)]
    pub fn verify(secret: &[u8], message: &[u8], digest: &[u8]) -> Result<bool, UNiDError> {
        let computed = match HmacSha256::digest(secret, message) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok(computed.eq(digest))
    }
}

#[allow(dead_code)]
pub struct HmacSha512 {}

impl HmacSha512 {
    #[allow(dead_code)]
    pub fn digest(secret: &[u8], message: &[u8]) -> Result<Vec<u8>, UNiDError> {
        let mut mac = match _HmacSha512::new_from_slice(secret) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        mac.update(message);

        Ok(mac.finalize().into_bytes().to_vec())
    }

    #[allow(dead_code)]
    pub fn verify(secret: &[u8], message: &[u8], digest: &[u8]) -> Result<bool, UNiDError> {
        let computed = match HmacSha512::digest(secret, message) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
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
        String::from(r#"{"k":"UNiD"}"#)
    }

    fn secret() -> String {
        String::from(r#"secret"#)
    }

    #[test]
    fn test_hmac_sha256_digest() {
        let result = HmacSha256::digest(&secret().as_bytes().to_vec(), &message().as_bytes().to_vec());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::from([
            134, 222, 100, 203, 106, 139, 7  , 109, 249, 115,
            164, 201, 217, 117, 141, 247, 210, 39 , 30 , 106,
            78 , 223, 40 , 73 , 4  , 136, 85 , 220, 180, 35 ,
            10 , 239
        ]));
    }

    #[test]
    fn test_hmac_sha256_verify() {
        let digest = Vec::from([
            134, 222, 100, 203, 106, 139, 7  , 109, 249, 115,
            164, 201, 217, 117, 141, 247, 210, 39 , 30 , 106,
            78 , 223, 40 , 73 , 4  , 136, 85 , 220, 180, 35 ,
            10 , 239
        ]);

        let result = HmacSha256::verify(&secret().as_bytes().to_vec(), &message().as_bytes().to_vec(), &digest);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_hmac_sha512_digest() {
        let result = HmacSha512::digest(&secret().as_bytes().to_vec(), &message().as_bytes().to_vec());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::from([
            116, 195, 53 , 120, 221, 155, 175, 166, 122, 210,
            252, 116, 84 , 17 , 177, 176, 5  , 244, 199, 57 ,
            134, 74 , 33 , 251, 246, 194, 211, 12 , 235, 222,
            155, 183, 181, 249, 183, 138, 110, 160, 153, 30 ,
            87 , 196, 185, 227, 121, 207, 107, 103, 107, 163,
            233, 176, 176, 76 , 205, 129, 233, 75 , 150, 231,
            19 , 231, 130, 249
        ]));
    }

    #[test]
    fn test_hmac_sha512_verify() {
        let digest = Vec::from([
            116, 195, 53 , 120, 221, 155, 175, 166, 122, 210,
            252, 116, 84 , 17 , 177, 176, 5  , 244, 199, 57 ,
            134, 74 , 33 , 251, 246, 194, 211, 12 , 235, 222,
            155, 183, 181, 249, 183, 138, 110, 160, 153, 30 ,
            87 , 196, 185, 227, 121, 207, 107, 103, 107, 163,
            233, 176, 176, 76 , 205, 129, 233, 75 , 150, 231,
            19 , 231, 130, 249
        ]);

        let result = HmacSha512::verify(&secret().as_bytes().to_vec(), &message().as_bytes().to_vec(), &digest);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}