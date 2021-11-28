use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

use crate::unid::utils::random::*;
use scrypt::{
    password_hash::{Output, PasswordHash, SaltString, PasswordHasher },
    Params, Scrypt,
};
use crate::MUTEX_HANDLERS;


pub struct Cipher {}

impl Cipher {
    
    pub fn encrypt(plaintext: String, secret: String) -> String {
        // define 16 bytes long string, iv_str
        let iv_string: String = get_random_bytes(16);
        // convert the string into byte array and store in the variable iv
        let iv_u8: &[u8] = iv_string.as_bytes();
        // convert iv byte array into vec
        let mut iv_vec: Vec<u8> = iv_u8.to_vec();
        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("iv bytes = {:?}", iv_vec.clone()));
        }
        // define 32 bytes long string, salt_str
        let salt_string: String = get_random_bytes(32);
        //convert the string into byte slice
        let salt_u8: &[u8] = salt_string.as_bytes();
        //convert salt byte array into vec
        let mut salt_vec: Vec<u8> = salt_u8.to_vec();
        
        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("salt bytes = {:?}", salt_vec.clone()));
        }
        // create saltString from the salt byte slice
        let salt_ss: SaltString = SaltString::b64_encode(salt_u8).unwrap();

        // convert secret string into str slice
        let secret_str: &str = &secret;
        // convert secret str slice into byte slice
        let secret_u8: &[u8] = secret_str.as_bytes();

        // get modified Params for Scrypt Algorithm
        let params: Params = Params::new(7, 8, 1).unwrap();
        
        // get the phc string by hashing secret using salt with given parameters of scrypt algorithm
        let key_phc: PasswordHash = Scrypt
            .hash_password_customized(secret_u8, Some(scrypt::ALG_ID), None, params, &salt_ss)
            .unwrap();


        //get the key hash output only from the phc string
        let key_output: Output = key_phc.hash.unwrap();

        // convert the key hash output into byte slice
        let key_vec: Vec<u8> = key_output.as_bytes().to_vec();

        
        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("key bytes = {:?}", key_vec));
        }

        // convert plaintext from String to byte slice
        let plaintext_vec: Vec<u8> = plaintext.into_bytes();
        
        // encrypt the plaintext using the given arguments
        let mut ciphertext_vec: Vec<u8> = unsafe { crate::AES_CRYPT.encrypt(plaintext_vec, key_vec, iv_vec.clone()) };

        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("ciphertext bytes = {:?}", ciphertext_vec.clone()));
        }
        // start a vec buffer
        let mut buffer_vec = Vec::new();
        // add salt byte array as byte buffer to buffer variable
        buffer_vec.append(&mut salt_vec);
        // add the encrypted ciphertext bytes to the end of buffer
        buffer_vec.append(&mut ciphertext_vec);
        // add iv byte slice to the end of buffer
        buffer_vec.append(&mut iv_vec);

        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("buffer bytes = {:?}", buffer_vec.clone()));
        }
        // convert buffer into raw byte buffer aka byte vector
        // let buffer_vec: Vec<u8> = buffer.to_bytes();
        // encode buffer64 into base64 representation
        base64::encode(&buffer_vec)
    }

}
