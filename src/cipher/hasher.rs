use base64::DecodeError;
use fastcmp::Compare;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha512;
use wasm_bindgen::prelude::*;

use super::super::utils::*;
use serde_json::json;

type HmacSha512 = Hmac<Sha512>;

#[wasm_bindgen]
pub struct Hasher {}

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

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn it_should_hasher_digest_verify_1() {
    let data_serde: serde_json::Value = serde_json::json!({
      "a": "hello",
      "b": "world"
    });
    let data: &str = &data_serde.to_string();
    let secret: &str = "secret123";
    let digested: String = Hasher::digest(data.to_string(), secret.to_string());
    let verified: bool = Hasher::verify(data.to_string(), digested.to_string(), secret.to_string());
    assert_eq!(verified, true);
    assert_eq!(
      digested.to_string(),
      "OM+bDTbUVutMpKxggbcI5HvVJU+1XO1O4IM7jzE69oYKpICBbLU/PWe0ZC8icnk6O3/TdkVajVNmlpct6oRNkQ=="
        .to_string()
    );
  }
}
