use sha2::Sha256;
use hmac::Hmac;
use alloc::vec::Vec;

struct HmacSha256 {}

type THmacSha256 = Hmac<Sha256>;

impl HmacSha256 {
    pub fn digest(secret: &Vec<u8>, message: &Vec<u8>) -> Vec<u8> {
        Vec::from([])
    }

    pub fn verify(message: &Vec<u8>, digest: &Vec<u8>) -> bool {
        true
    }
}


#[cfg(test)]
mod tests {
    // use super::*;
    // use alloc::vec::Vec;

    #[test]
    fn test_digest() {
    }

    #[test]
    fn test_verify() {
    }
}