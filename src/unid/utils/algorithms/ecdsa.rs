use alloc::vec::Vec;
use alloc::string::{String, ToString};
use cstr_core::{CStr, CString, c_char};

use libsecp256k1_core::curve::Scalar;
use crate::unid::utils::algorithms::secp256k1::{PublicKey, PublicKeyFormat, Signature};

pub struct Ecdsa {}

impl Ecdsa {
    pub fn sign(secret_key_vec: &[u8], msg_vec: &[u8] ) -> String {
        String::from("")
        // let secret_hex_string: String = hex::encode_upper(secret_key_vec.to_vec());
        // let secret_c_string = CString::new(secret_hex_string).unwrap();
        // let secret_ptr = secret_c_string.into_raw();
        // let r_c_string = unsafe { CString::from_vec_unchecked(alloc::vec![0u8;100]) };
        // let s_c_string = unsafe { CString::from_vec_unchecked(alloc::vec![0u8;100]) };
        // let r_ptr: *mut c_char = r_c_string.into_raw();
        // let s_ptr: *mut c_char = s_c_string.into_raw();

        // let msg_data_t: DataT = DataT::new(msg_vec.to_vec());
        // let msg_ptr: *mut u8 = msg_data_t.ptr;

        // let handler = unsafe { MUTEX_HANDLERS.lock().get_ecdsa_signer_handler() };

        // handler.unwrap()(secret_ptr, msg_ptr, r_ptr, s_ptr);

        // let r_out = unsafe {
        //     assert!(! r_ptr.is_null());

        //     CStr::from_ptr(r_ptr)
        // };
        // let r_hex_string = r_out.to_str().unwrap().to_string();

        // let s_out = unsafe {
        //     assert!(! s_ptr.is_null());

        //     CStr::from_ptr(s_ptr)
        // };
        // let s_hex_string = s_out.to_str().unwrap().to_string();
        // let r_vec: Vec<u8> = hex::decode(&r_hex_string).unwrap();
        // let s_vec: Vec<u8> = hex::decode(&s_hex_string).unwrap();
        // if r_vec.len() != 32 || s_vec.len() != 32 {
        //     return "size of r, s are invalid".to_string();
        // }
        // let r_u8_32: &[u8;32] = unsafe {let ptr = r_vec.as_ptr() as *const [u8; 32]; &*ptr};
        // let s_u8_32: &[u8;32] = unsafe {let ptr = s_vec.as_ptr() as *const [u8; 32]; &*ptr};

        // let mut r_scalar = Scalar::default();
        // let mut s_scalar = Scalar::default();
        // let _ = r_scalar.set_b32(r_u8_32);
        // let _ = s_scalar.set_b32(s_u8_32);

        // let sig: Signature = Signature {
        //     r: r_scalar,
        //     s: s_scalar
        // };

        // let sig_u8 = sig.serialize();
        // base64::encode(sig_u8.to_vec())
    }

    pub fn verify(pub_key_u8: &[u8], signature_u8: &[u8], msg_hashed_u8: &[u8]) -> bool {
        true
        // let signature_u8: &[u8] = &signature_vec[..];
        // let sig: Signature = Signature::parse_standard_slice(signature_u8).unwrap();

        // let r_scalar: Scalar = sig.r;
        // let s_scalar: Scalar = sig.s;

        // let r_u8_32 = r_scalar.b32();
        // let s_u8_32 = s_scalar.b32();

        // let r_vec = r_u8_32.to_vec();
        // let s_vec = s_u8_32.to_vec();

        // let r_hex_string: String = hex::encode_upper(r_vec);
        // let s_hex_string: String = hex::encode_upper(s_vec);

        // // let pub_key_u8: &[u8] = &pub_key_vec[..];
        // let pub_key_pk = PublicKey::parse_slice(pub_key_u8, Some(PublicKeyFormat::Full)).unwrap();

        // let x_u8_32: [u8;32] = pub_key_pk.0.x.b32();
        // let y_u8_32: [u8;32] = pub_key_pk.0.y.b32();

        // let x_vec = x_u8_32.to_vec();
        // let y_vec = y_u8_32.to_vec();

        // let x_hex_string: String = hex::encode_upper(x_vec);
        // let y_hex_string: String = hex::encode_upper(y_vec);

        // let mut msg_hashed_data_t: DataT = DataT::new(msg_hashed_u8.to_vec());

        // let x_c_string = CString::new(x_hex_string).unwrap();
        // let x_ptr = x_c_string.into_raw();

        // let y_c_string = CString::new(y_hex_string).unwrap();
        // let y_ptr = y_c_string.into_raw();

        // let r_c_string = CString::new(r_hex_string).unwrap();
        // let r_ptr = r_c_string.into_raw();

        // let s_c_string = CString::new(s_hex_string).unwrap();
        // let s_ptr = s_c_string.into_raw();

        // let mut result: i32 = -1;

        // let handler = unsafe { MUTEX_HANDLERS.lock().get_ecdsa_verifier_handler() };

        // handler.unwrap()(x_ptr, y_ptr, r_ptr, s_ptr, &mut msg_hashed_data_t, &mut result);

        // result == 0
    }
}
