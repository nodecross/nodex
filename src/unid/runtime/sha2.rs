use sha2::{Sha256, Digest};

pub struct SHA256 {}

impl SHA256 {
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
        String::from("Hello, UNiD!")
    }

    #[test]
    pub fn test_sha256() {
        let result = SHA256::digest(&message().as_bytes());

        assert_eq!(result, vec![
            0xc6, 0x72, 0xc8, 0x43, 0xd5, 0xfd, 0x4c, 0x8c, 0x9f, 0xa4,
            0x7f, 0xa0, 0x02, 0x1d, 0x87, 0x6b, 0xb0, 0x2b, 0x1c, 0xe6,
            0xa6, 0x1e, 0xcc, 0xf8, 0xf5, 0x1c, 0x2f, 0xb0, 0xe6, 0x26,
            0xe4, 0xef,
        ])
    }
}