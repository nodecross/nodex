use wasm_bindgen::prelude::*;
use sha2::{Sha512};
use hmac::{Hmac, Mac, NewMac};
use base64::DecodeError;
use fastcmp::Compare;

type HmacSha512 = Hmac<Sha512>;

#[wasm_bindgen]
pub struct Hasher {
}


#[wasm_bindgen]
impl Hasher {
  pub fn digest(content: String, secret: String) -> String {
    let secret_str: &str = &secret;
    let secret_u8: &[u8] = secret_str.as_bytes();
    let mut mac = HmacSha512::new_from_slice(secret_u8).unwrap();
  
    let content_str: &str = &content;
    let content_u8: &[u8] = content_str.as_bytes();
    mac.update(content_u8);
    let result = mac.finalize();
    let result_u8: &[u8] = &result.into_bytes();
  
    return base64::encode(result_u8.to_vec());
  }
  
  pub fn verify(content: String, digest: String, secret: String) -> bool {
    let secret_str: &str = &secret;
    let secret_u8: &[u8] = secret_str.as_bytes();
    let mut mac = HmacSha512::new_from_slice(secret_u8).unwrap();
  
    let content_str: &str = &content;
    let content_u8: &[u8] = content_str.as_bytes();
    mac.update(content_u8);
    let result = mac.finalize();
    let result_u8 = &result.into_bytes();
  
    let digest_decoded_vec: Vec<u8> = base64::decode(digest.as_bytes()).unwrap(); 
    if (digest_decoded_vec.feq(result_u8)) {
      return true;
    } else {
      return false;
    }
  }
}


