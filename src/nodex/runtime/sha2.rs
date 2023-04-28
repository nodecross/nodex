use sha2::{Sha256, Digest};

#[allow(dead_code)]
pub struct SHA256 {}

impl SHA256 {
    #[allow(dead_code)]
    pub fn digest(message: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();

        hasher.update(&message);

        hasher.finalize().to_vec()
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

    #[test]
    pub fn test_sha256() {
        let result = SHA256::digest(message().as_bytes());

        assert_eq!(result, vec![
            0x9f, 0x9f, 0x51, 0x11, 0xf7, 0xb2, 0x7a, 0x78, 0x1f, 0x1f,
            0x1d, 0xdd, 0xe5, 0xeb, 0xc2, 0xdd, 0x2b, 0x79, 0x6b, 0xfc,
            0x73, 0x65, 0xc9, 0xc2, 0x8b, 0x54, 0x8e, 0x56, 0x41, 0x76,
            0x92, 0x9f,
        ])
    }
}