use alloc::vec::Vec;
use crate::unid::errors::UNiDError;
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use aes_gcm_siv::aead::{Aead, NewAead};

#[derive(Debug,PartialEq)]
pub struct AesGcmSiv {}

impl AesGcmSiv {
    pub fn encrypt(_key: &[u8], _nonce: &[u8], _plain_text: &[u8]) -> Result<Vec<u8>, UNiDError> {
        let key = Key::from_slice(_key);
        let nonce = Nonce::from_slice(_nonce);

        let cipher = Aes256GcmSiv::new(key);

        match cipher.encrypt(&nonce, _plain_text) {
            Ok(v) => Ok(v.to_vec()),
            Err(_) => Err(UNiDError{})
        }
    }

    pub fn decrypt(_key: &[u8], _nonce: &[u8], _cipher_text: &[u8]) -> Result<Vec<u8>, UNiDError> {
        let key = Key::from_slice(_key);
        let nonce = Nonce::from_slice(_nonce);

        let cipher = Aes256GcmSiv::new(key);

        match cipher.decrypt(nonce, _cipher_text) {
            Ok(v) => Ok(v),
            Err(_) => Err(UNiDError{})
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn key() -> Vec<u8> {
        vec![
            0xcd, 0x33, 0x90, 0x46, 0x12, 0x34, 0x25, 0x3d, 0x96, 0x6f,
            0xf7, 0x89, 0x8b, 0xc1, 0x73, 0xfa, 0x1a, 0xa6, 0xbb, 0xc2,
            0x3d, 0xdd, 0x90, 0x37, 0x93, 0x03, 0x1c, 0x4d, 0x65, 0x54,
            0x74, 0x0c,
        ]
    }

    #[fixture]
    fn nonce() -> Vec<u8> {
        vec![
            0x79, 0x6d, 0xc7, 0x33, 0x43, 0x54, 0xeb, 0x3b, 0x22, 0x32,
            0xde, 0x60,
        ]
    }

    #[fixture]
    fn plain_text() -> Vec<u8> {
        Vec::from(String::from("Hello, UNiD!"))
    }

    #[fixture]
    fn cipher_text() -> Vec<u8> {
        vec![
            0x2b, 0xfe, 0x12, 0x7a, 0x88, 0x08, 0x24, 0xd8, 0x48, 0x3f,
            0xf4, 0x40, 0xc0, 0xdb, 0xd5, 0xde, 0x78, 0x7c, 0x1e, 0x49,
            0x6b, 0x34, 0x06, 0xc1, 0x39, 0x40, 0x51, 0x53,
        ]
    }
  
    #[test]
    pub fn it_should_success_encrypt() {
        let _cipher_text = match AesGcmSiv::encrypt(&key(), &nonce(), &plain_text()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(cipher_text(), _cipher_text)
    }

    #[test]
    pub fn it_should_success_decrypt() {
        let _plain_text = match AesGcmSiv::decrypt(&key(), &nonce(), &cipher_text()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(plain_text(), _plain_text)
    }
}
