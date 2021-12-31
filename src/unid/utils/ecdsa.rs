use crate::unid::utils::data_t::DataT;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use cstr_core::{CStr, CString, c_char};
// use crate::MUTEX_HANDLERS;

use libsecp256k1_core::curve::Scalar;
use crate::unid::utils::secp256k1::{PublicKey, PublicKeyFormat, Signature};

pub struct Ecdsa {
  // kp_generator: extern "C" fn(*mut DataT),
  signer  : extern "C" fn(*mut c_char, *mut u8, *mut c_char, *mut c_char),
  verifier : extern "C" fn(*mut c_char, *mut c_char, *mut c_char, *mut c_char, *mut DataT, *mut i32),
}

impl Ecdsa {
  pub const fn empty() -> Ecdsa {
    Ecdsa {
      // kp_generator: Ecdsa::noop_kp_generator,
      signer : Ecdsa::noop_signer,
      verifier : Ecdsa::noop_verifier
    }
  }
  
  // extern "C" fn noop_kp_generator(_kp_data_t: *mut DataT) {}

  extern "C" fn noop_signer(_secret_hex_string: *mut c_char, _msg_hashed_ptr: *mut u8, _r_hex_string: *mut c_char,_s_hex_string: *mut c_char) {}

  extern "C" fn noop_verifier(_x_hex_string: *mut c_char, _y_hex_string: *mut c_char, _r_hex_string: *mut c_char, _s_hex_string: *mut c_char, _msg_hashed_data_t: *mut DataT, _result: *mut i32) {}

  pub fn init(
    &mut self,
    // kp_generator: extern "C" fn(*mut DataT),
    signer: extern "C" fn(*mut c_char, *mut u8, *mut c_char, *mut c_char),
    verifier: extern "C" fn(*mut c_char, *mut c_char, *mut c_char, *mut c_char, *mut DataT, *mut i32)
  ) {
    // self.kp_generator = kp_generator;
    self.signer = signer;
    self.verifier = verifier;
  }

  // pub fn kp_gen(&self) -> Vec<u8> {
  //   let len = 500;
  //   let kp_vec: Vec<u8> = alloc::vec![0u8; len];
  //   let mut kp_data_t: DataT = DataT::new(kp_vec);
  //   (self.kp_generator)(&mut kp_data_t);
  //   kp_data_t.to_vec()
  // }

  pub fn sign(&self, secret_key_vec: Vec<u8>, msg_vec: Vec<u8> ) -> String {

    let secret_hex_string: String = hex::encode_upper(secret_key_vec);
    let secret_c_string = CString::new(secret_hex_string).unwrap();
    let secret_ptr = secret_c_string.into_raw();
    let r_c_string = unsafe { CString::from_vec_unchecked(alloc::vec![0u8;100]) };
    let s_c_string = unsafe { CString::from_vec_unchecked(alloc::vec![0u8;100]) };
    let r_ptr: *mut c_char = r_c_string.into_raw();
    let s_ptr: *mut c_char = s_c_string.into_raw();

    let msg_data_t: DataT = DataT::new(msg_vec);
    let msg_ptr: *mut u8 = msg_data_t.ptr;

    (self.signer)(secret_ptr, msg_ptr, r_ptr, s_ptr);


    let r_out = unsafe {
      assert!(! r_ptr.is_null());

      CStr::from_ptr(r_ptr)
    };
    let r_hex_string = r_out.to_str().unwrap().to_string();

    let s_out = unsafe {
      assert!(! s_ptr.is_null());

      CStr::from_ptr(s_ptr)
    };
    let s_hex_string = s_out.to_str().unwrap().to_string();
    let r_vec: Vec<u8> = hex::decode(&r_hex_string).unwrap();
    let s_vec: Vec<u8> = hex::decode(&s_hex_string).unwrap();
    if r_vec.len() != 32 || s_vec.len() != 32 {
        return "size of r, s are invalid".to_string();
    }
    let r_u8_32: &[u8;32] = unsafe {let ptr = r_vec.as_ptr() as *const [u8; 32]; &*ptr};
    let s_u8_32: &[u8;32] = unsafe {let ptr = s_vec.as_ptr() as *const [u8; 32]; &*ptr};

    let mut r_scalar = Scalar::default();
    let mut s_scalar = Scalar::default();
    let _ = r_scalar.set_b32(r_u8_32);
    let _ = s_scalar.set_b32(s_u8_32);

    let sig: Signature = Signature {
        r: r_scalar,
        s: s_scalar
    };

    let sig_u8 = sig.serialize();
    base64::encode(sig_u8.to_vec())
  }

  pub fn verify(&self, pub_key_vec: Vec<u8>, signature_vec: Vec<u8>, msg_hashed_vec: Vec<u8>) -> bool {
    
    let signature_u8: &[u8] = &signature_vec[..];
    let sig: Signature = Signature::parse_standard_slice(signature_u8).unwrap();

    let r_scalar: Scalar = sig.r;
    let s_scalar: Scalar = sig.s;
    
    let r_u8_32 = r_scalar.b32();
    let s_u8_32 = s_scalar.b32();

    let r_vec = r_u8_32.to_vec();
    let s_vec = s_u8_32.to_vec();

    let r_hex_string: String = hex::encode_upper(r_vec);
    let s_hex_string: String = hex::encode_upper(s_vec);


    let pub_key_u8: &[u8] = &pub_key_vec[..];
    let pub_key_pk = PublicKey::parse_slice(pub_key_u8, Some(PublicKeyFormat::Full)).unwrap();

    let x_u8_32: [u8;32] = pub_key_pk.0.x.b32();
    let y_u8_32: [u8;32] = pub_key_pk.0.y.b32();

    let x_vec = x_u8_32.to_vec();
    let y_vec = y_u8_32.to_vec();

    let x_hex_string: String = hex::encode_upper(x_vec);
    let y_hex_string: String = hex::encode_upper(y_vec);


    let mut msg_hashed_data_t: DataT = DataT::new(msg_hashed_vec);

    let x_c_string = CString::new(x_hex_string).unwrap();
    let x_ptr = x_c_string.into_raw();

    let y_c_string = CString::new(y_hex_string).unwrap();
    let y_ptr = y_c_string.into_raw();

    let r_c_string = CString::new(r_hex_string).unwrap();
    let r_ptr = r_c_string.into_raw();

    let s_c_string = CString::new(s_hex_string).unwrap();
    let s_ptr = s_c_string.into_raw();

    let mut result: i32 = -1;

    (self.verifier)(x_ptr, y_ptr, r_ptr, s_ptr, &mut msg_hashed_data_t, &mut result);

    result == 0
  }
}


#[cfg(test)]
pub mod tests {
  use super::*;
  
  #[test]
  #[ignore]
  pub fn it_should_aes_encrypt() {
    let plaintext: &str= "hello";
    let key_base64: &str = "ZEj+lI1NEpwbxqpMfFTwxIK8/XbHsSJtj+dam59NavI=";
    let iv_base64: &str = "QUJDREVGR0hJSktMTU5PUA==";
    let plaintext_vec: Vec<u8> = plaintext.as_bytes().to_vec();
    let key_vec: Vec<u8> = base64::decode(key_base64).unwrap();
    let iv_vec: Vec<u8> = base64::decode(iv_base64).unwrap();

    let ciphertext_vec: Vec<u8> = unsafe { crate::AES_CRYPT.encrypt(plaintext_vec, key_vec, iv_vec) };
    let ciphertext_base64 = base64::encode(ciphertext_vec);

    assert_eq!(ciphertext_base64, "5FBuToCO9PiApjHbK+25Vg==".to_string());
  }

  #[test]
  #[ignore]
  pub fn it_should_aes_decrypt() {
    let ciphertext_base64: &str= "5FBuToCO9PiApjHbK+25Vg==";
    let key_base64: &str = "ZEj+lI1NEpwbxqpMfFTwxIK8/XbHsSJtj+dam59NavI=";
    let iv_base64: &str = "QUJDREVGR0hJSktMTU5PUA==";
    let ciphertext_vec: Vec<u8> = base64::decode(ciphertext_base64).unwrap();
    let key_vec: Vec<u8> = base64::decode(key_base64).unwrap();
    let iv_vec: Vec<u8> = base64::decode(iv_base64).unwrap();

    let plaintext_vec: Vec<u8> = unsafe { crate::AES_CRYPT.decrypt(ciphertext_vec, key_vec, iv_vec) };
    let plaintext_string: String = String::from_utf8(plaintext_vec).unwrap();

    assert_eq!(plaintext_string, "hello");
  }
}
