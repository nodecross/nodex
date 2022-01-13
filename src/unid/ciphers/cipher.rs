use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

use crate::unid::utils::random::*;
use scrypt::{
    password_hash::{Output, PasswordHash, SaltString, PasswordHasher },
    Params, Scrypt,
};

use crate::unid::utils::aes_crypt::AesCrypt;
use crate::MUTEX_HANDLERS;

#[allow(dead_code)]
pub struct Cipher {}

impl Cipher {
    #[allow(dead_code)]
    pub fn encrypt(plaintext: String, secret: String) -> String {
        let salt_vec: Vec<u8> = unsafe { Random::trng_bytes(&(32_usize)).unwrap() };
        let salt_u8: &[u8] = &salt_vec[..];
        let salt_ss: SaltString = SaltString::b64_encode(salt_u8).unwrap();

        let secret_u8: &[u8] = (&secret).as_bytes();

        let params: Params = Params::new(7, 8, 1).unwrap();
        
        let key_phc: PasswordHash = Scrypt
            .hash_password_customized(secret_u8, Some(scrypt::ALG_ID), None, params, &salt_ss)
            .unwrap();

        let key_output: Output = key_phc.hash.unwrap();
        let key_u8: &[u8] = key_output.as_bytes();

        let iv_vec: Vec<u8> = unsafe { Random::trng_bytes(&(16_usize)).unwrap() };
        let iv_u8: &[u8] = &iv_vec[..];



        let plaintext_u8: &[u8] = plaintext.as_bytes();
        
        let ciphertext_vec: Vec<u8> = AesCrypt::encrypt(plaintext_u8.to_vec(), key_u8.to_vec(), iv_u8.to_vec());
        let ciphertext_u8: &[u8] = &ciphertext_vec[..];

        let mut buffered_ciphertext_vec = Vec::new();
        buffered_ciphertext_vec.append(&mut salt_u8.to_vec());
        buffered_ciphertext_vec.append(&mut ciphertext_u8.to_vec());
        buffered_ciphertext_vec.append(&mut iv_u8.to_vec());

        let buffered_ciphertext_u8: &[u8] = &buffered_ciphertext_vec[..];

        let buffered_ciphertext_base64: String = base64::encode(buffered_ciphertext_u8.to_vec());

        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("salt bytes = {:?}", salt_u8));
            logger.debug(format!("salt base64 = {:?}", base64::encode(salt_u8.to_vec())));

            logger.debug(format!("iv bytes = {:?}", iv_u8));
            logger.debug(format!("iv base64 = {:?}", base64::encode(iv_u8.to_vec())));

            logger.debug(format!("key bytes = {:?}", key_u8));
            logger.debug(format!("key base64 = {:?}", base64::encode(key_u8.to_vec())));

            logger.debug(format!("ciphertext bytes = {:?}", ciphertext_u8));
            logger.debug(format!("ciphertext base64 = {:?}", base64::encode(ciphertext_u8.to_vec())));

            logger.debug(format!("buffered ciphertext bytes = {:?}", buffered_ciphertext_u8));

            logger.debug(format!("buffered ciphertext base64 = {:?}", buffered_ciphertext_base64));
        }

        buffered_ciphertext_base64
    }

    #[allow(dead_code)]
    pub fn decrypt(buffered_ciphertext_base64: String, secret: String) -> String {

        let buffered_ciphertext_vec: Vec<u8> = base64::decode(buffered_ciphertext_base64.as_bytes()).unwrap();
        let buffered_ciphertext_u8: &[u8] = &buffered_ciphertext_vec[..];
        let buffered_ciphertext_len: usize = buffered_ciphertext_u8.len();

        assert!(buffered_ciphertext_len >= 64);

        let salt_u8: &[u8] = &buffered_ciphertext_u8[..32];
        let ciphertext_u8: &[u8] = &buffered_ciphertext_u8[32..buffered_ciphertext_len-16];
        let iv_u8: &[u8] = &buffered_ciphertext_u8[buffered_ciphertext_len-16..];
  

        let secret_u8: &[u8] = (&secret).as_bytes();
        let salt_ss: SaltString = SaltString::b64_encode(salt_u8).unwrap();
        let params: Params = Params::new(7, 8, 1).unwrap();

        let key_phc: PasswordHash = Scrypt
            .hash_password_customized(secret_u8, Some(scrypt::ALG_ID), None, params, &salt_ss)
            .unwrap();

        let key_output: Output = key_phc.hash.unwrap();
        let key_u8: &[u8] = key_output.as_bytes();
        
        let plaintext_vec: Vec<u8> = AesCrypt::decrypt(ciphertext_u8.to_vec(), key_u8.to_vec(), iv_u8.to_vec());
        let plaintext_u8: &[u8] = &plaintext_vec[..];

        let plaintext = String::from_utf8(plaintext_u8.to_vec()).unwrap();

        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("salt bytes = {:?}", salt_u8));
            logger.debug(format!("salt base64 = {:?}", base64::encode(salt_u8.to_vec())));

            logger.debug(format!("iv bytes = {:?}", iv_u8));
            logger.debug(format!("iv base64 = {:?}", base64::encode(iv_u8.to_vec())));

            logger.debug(format!("ciphertext bytes = {:?}", ciphertext_u8));
            logger.debug(format!("ciphertext base64 = {:?}", base64::encode(ciphertext_u8.to_vec())));

            logger.debug(format!("key bytes = {:?}", key_u8));
            logger.debug(format!("key base64 = {:?}", base64::encode(key_u8.to_vec())));

            logger.debug(format!("plaintext bytes = {:?}", plaintext_u8));

            logger.debug(format!("plaintext = {:?}", plaintext));
        }

        plaintext
    }
}
