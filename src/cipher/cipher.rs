
use wasm_bindgen::prelude::*;
use std::str;
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use base64::DecodeError;

use scrypt::{
    Params,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, Ident, Output},
    Scrypt
};
use serde_json::json;
use super::super::utils::*;



// declare alias for block cipher with AES-CBC and PKCS7 padding
type AesCbc = Cbc<Aes256, Pkcs7>;

#[wasm_bindgen]
pub struct Cipher {
}

#[wasm_bindgen]
impl Cipher {
  pub fn encrypt(plaintext: String, secret: String) -> String {
    // define 16 bytes long string, iv_str
    let iv_string: String = gen_ascii_chars(16);
    // convert the string into byte array and store in the variable iv
    let iv_u8: &[u8] = iv_string.as_bytes();
    // define 32 bytes long string, salt_str
    let salt_string: String = gen_ascii_chars(32);
    //convert the string into byte slice
    let salt_u8: &[u8] = salt_string.as_bytes();
    // create saltString from the salt byte slice
    let salt_ss: SaltString = SaltString::b64_encode(salt_u8).unwrap();
    // convert secret string into str slice
    let secret_str: &str = &secret;
    // convert secret str slice into byte slice
    let secret_u8: &[u8] = secret_str.as_bytes();
    // get modified Params for Scrypt Algorithm
    let params: Params = Params::new(14, 8, 1).unwrap();
    
    // get the phc string by hashing secret using salt with given parameters of scrypt algorithm 
    let key_phc: PasswordHash = Scrypt.hash_password(secret_u8,Some(scrypt::ALG_ID),params, &salt_ss).unwrap();
    
    //alternative to the previous command using default params
    // let key_phc = PasswordHash::generate(Scrypt, secret_bytes,&salt_str).unwrap();
    
    //get the key hash output only from the phc string
    let key_output: Output = key_phc.hash.unwrap();

    //convert the key hash output into byte slice
    let key_u8: &[u8] = key_output.as_bytes();

    // convert plaintext from String to byte slice
    let plaintext_u8: &[u8] = (&plaintext).as_bytes();
    // create block cipher instance with AES-CBC with PKCS7 padding
    let cipher = AesCbc::new_from_slices(key_u8, iv_u8).unwrap();
    
    // encrypt the byte slice into encrypted byte vector
    let ciphertext_vec: Vec<u8> = cipher.encrypt_vec(plaintext_u8);
    // convert the byte vector into byte slice
    let ciphertext_u8: &[u8] = &ciphertext_vec[..];

    // add salt byte array as byte buffer to buffer variable
    let mut buffer = bytebuffer::ByteBuffer::from_bytes(salt_u8);
    // add the encrypted ciphertext bytes to the end of buffer
    buffer.write_bytes(ciphertext_u8);
    // add iv byte slice to the end of buffer
    buffer.write_bytes(iv_u8);
    // convert buffer into raw byte buffer aka byte vector
    let buffer_vec: Vec<u8> = buffer.to_bytes();
    // encode buffer64 into base64 representation
    return base64::encode(buffer_vec);
  }

  pub fn decrypt(ciphertext: String, secret: String) -> String {

    let ciphertext_vec: Vec<u8> = base64::decode(ciphertext.as_bytes()).unwrap();

    // convert ciphertext vector into byte slice
    let ciphertext_u8: &[u8] = &ciphertext_vec[..];

    assert_eq!(ciphertext_u8.len() > 48, true);
    
    
    // extract salt byte slice from the ciphertext
    let salt_u8: &[u8] = &ciphertext_u8[..32];

    
    // convert secret string into byte slice
    let secret_u8: &[u8] = (&secret).as_bytes();
    
    // let password_hash = Scrypt.hash_password_simple(key_bytes, &salt_in).unwrap();
    let salt_ss: SaltString = SaltString::b64_encode(salt_u8).unwrap();
    // let ident = Ident::new("Scrypt");
    let params: Params = Params::new(14, 8, 1).unwrap();
    
    // get the phc string by hashing secret using salt with given parameters of scrypt algorithm 
    let key_phc: PasswordHash = Scrypt.hash_password(secret_u8,Some(scrypt::ALG_ID),params, &salt_ss).unwrap();
    
    //alternative to the previous command using default params
    // let key_phc = PasswordHash::generate(Scrypt, secret_bytes,&salt_str).unwrap();
    
    //get the key hash output only from the phc string
    let key_output: Output = key_phc.hash.unwrap();

    //convert the key hash output into byte slice
    let key_u8: &[u8] = key_output.as_bytes();

    // create block cipher instance with AES-CBC with PKCS7 padding
    let cipher = AesCbc::new_from_slices(key_u8, &ciphertext_u8[ciphertext_u8.len()-16..]).unwrap();

    // decipher the byte slice into decrypted byte vector
    let decrypted_ciphertext_vec: Vec<u8> = cipher.decrypt_vec(&ciphertext_u8[32..ciphertext_u8.len()-16]).unwrap();

    // Convert the decrypted u8 byte vector into string and return
    let result: String = String::from_utf8(decrypted_ciphertext_vec).unwrap();

    
    return result;
  }

}


#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn it_should_cipher_encrypt_decrypt_1() {
    let data: &str = "hello";
    let secret: &str = "secret";
    let enc: String = Cipher::encrypt(data.to_string(), secret.to_string());
    let dec: String = Cipher::decrypt(enc, secret.to_string());
    
    assert_eq!(data.to_string(), dec);
  }

  #[test]
  fn it_should_cipher_encrypt_decrypt_2() {
    let data_serde: serde_json::Value = json!({
      "hello0" : "hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello",
      "hello1" : "hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello",
      "hello2" : "hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello",
      "hello3" : "hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello",
      "hello4" : "hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello"
    });
    let data: &str = &data_serde.to_string();
    let secret: &str = "secret";
    let enc: String = Cipher::encrypt(data.to_string(), secret.to_string());
    let dec: String = Cipher::decrypt(enc, secret.to_string());
    
    assert_eq!(data.to_string(), dec);
  }

  #[test]
  #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: BlockModeError")]
  fn it_should_cipher_encrypt_decrypt_3() {
    let data: &str = "hello";
    let secret1: &str = "secret1";
    let secret2: &str = "secret2";
    let enc: String = Cipher::encrypt(data.to_string(), secret1.to_string());
    let dec: String = Cipher::decrypt(enc, secret2.to_string());
  }
  
  #[test]
  #[should_panic]
  fn it_should_cipher_decrypt_1() {
    const SALT_LENGTH: usize = 32;
    const IV_LENGTH: usize = 16;
    let data: String = gen_ascii_chars(SALT_LENGTH+IV_LENGTH-1);
    let secret: &str = "secret";
    Cipher::decrypt(data, secret.to_string());
  }

  #[test]
  fn it_should_cipher_decrypt_2() {
    let data: &str = "Hello world!";
    let enc_data: &str = "d21mUXlyTWtuZEVwQ2pBMXRQT2VxTlVLbGMyVzV2Qmiodr/+/GkoY8WUiz17vj8BUWJvQklTWERaaE94SldybQ==";
    let secret: &str = "secret";

    let dec: String = Cipher::decrypt(enc_data.to_string(), secret.to_string());
    assert_eq!(data.to_string(), dec);
  }
}