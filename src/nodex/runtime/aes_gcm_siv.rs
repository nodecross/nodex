use crate::nodex::errors::NodeXError;
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use aes_gcm_siv::aead::{Aead, NewAead};

#[derive(Debug, Eq, PartialEq)]
pub struct AesGcmSiv {}

impl AesGcmSiv {
    #[allow(dead_code)]
    pub fn encrypt(_key: &[u8], _nonce: &[u8], _plain_text: &[u8]) -> Result<Vec<u8>, NodeXError> {
        let key = Key::from_slice(_key);
        let nonce = Nonce::from_slice(_nonce);

        let cipher = Aes256GcmSiv::new(key);

        match cipher.encrypt(nonce, _plain_text) {
            Ok(v) => Ok(v.to_vec()),
            Err(_) => Err(NodeXError{})
        }
    }

    #[allow(dead_code)]
    pub fn decrypt(_key: &[u8], _nonce: &[u8], _cipher_text: &[u8]) -> Result<Vec<u8>, NodeXError> {
        let key = Key::from_slice(_key);
        let nonce = Nonce::from_slice(_nonce);

        let cipher = Aes256GcmSiv::new(key);

        match cipher.decrypt(nonce, _cipher_text) {
            Ok(v) => Ok(v),
            Err(_) => Err(NodeXError{})
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
        Vec::from(String::from("0123456789abcdef"))
    }

    #[fixture]
    fn cipher_text() -> Vec<u8> {
        vec![
            0xd3, 0x58, 0x02, 0x26, 0xd1, 0xf7, 0x01, 0xe4, 0xe3, 0x42, 
            0x5e, 0x2e, 0x5d, 0x1f, 0x5a, 0x67, 0xae, 0x92, 0xfd, 0x2a, 
            0x23, 0xf1, 0x2d, 0xef, 0xbe, 0x2e, 0x26, 0x59, 0xc0, 0xb9, 
            0xd2, 0x54, 
        ]
    }
  
    #[test]
    pub fn it_should_success_encrypt() {
        let _cipher_text = match AesGcmSiv::encrypt(&key(), &nonce(), &plain_text()) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(_cipher_text, cipher_text())
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
