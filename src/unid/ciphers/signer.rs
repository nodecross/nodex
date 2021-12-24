use alloc::string::{String, ToString};
use crate::unid::utils::secp256k1::{sign as signer_sign, verify as signer_verify, Message, PublicKey, PublicKeyFormat, SecretKey, Signature};
use alloc::vec::Vec;
use serde_json::json;
use sha2::{ Digest, Sha256 };
use crate::MUTEX_HANDLERS;
use alloc::format;

use crate::unid::utils::ecdsa::Ecdsa;
use cstr_core::{CStr, CString, c_char};


const PROOF_KEY: &str = "proof";
const VM_KEY: &str = "verificationMethod";
const JWS_KEY: &str = "jws";

struct SuiteSign {
    did: String,
    key_id: String,
    secret_key64: String,
}

struct SuiteVerify {
    _did: Option<String>,
    key_id: String,
    pub_key64: String,
}

pub struct Signer {}

impl Signer {
    // pub fn kp_gen() -> String {
    //     let kp: Vec<u8> = unsafe {
    //         crate::ECDSA.kp_gen()
    //     };
    //     base64::encode(kp)
    // }

    pub fn signew(message: String, secret_key64: String) -> String {
        let message_u8 = message.as_bytes();
        let message_vec: Vec<u8> = message_u8.to_vec();
    
        let secret_u8 = secret_key64.as_bytes();
        let secret_key_vec: Vec<u8> = base64::decode(secret_u8).unwrap();
        
        let output: String = unsafe { crate::ECDSA.sign(secret_key_vec.clone(), message_vec.clone()) };

        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("output signature = {:?}", output));
        }

        output
    }
    pub fn sign(message: String, secret_key64: String) -> String {
        let message_u8 = message.as_bytes();
        let secret_u8 = secret_key64.as_bytes();

        let digested = Sha256::digest(message_u8);
        let digested_u8: &[u8] = &digested.to_vec()[..];
        let digested_message = Message::parse_slice(digested_u8).unwrap();

        let secret_key_vec: Vec<u8> = base64::decode(secret_u8).unwrap();
        let secret_key_u8: &[u8] = &secret_key_vec[..];


        let secret_key_sk = SecretKey::parse_slice(secret_key_u8).unwrap();

        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("secret_key_sk = {:?}", secret_key_sk));
        }
        let sig_tuple = signer_sign(&digested_message, &secret_key_sk);

        unsafe {
            let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

            logger.debug(format!("sig tuple = {:?}", sig_tuple));
        }
        let sig = sig_tuple.0;
        let sig_u8 = sig.serialize();

        base64::encode(sig_u8.to_vec())
    }

    pub fn verify(message: String, signature64: String, pub_key64: String) -> bool {
        let message_str: &str = &message;
        let message_u8: &[u8] = message_str.as_bytes();

        let digested = Sha256::digest(message_u8);
        let digested_u8: &[u8] = &digested.to_vec()[..];
        let digested_message = Message::parse_slice(digested_u8).unwrap();

        let signature_vec: Vec<u8> = base64::decode(signature64.as_bytes()).unwrap();
        let signature_u8: &[u8] = &signature_vec[..];
        let sig = Signature::parse_standard_slice(signature_u8).unwrap();

        let pub_key_vec: Vec<u8> = base64::decode(pub_key64.as_bytes()).unwrap();
        let pub_key_u8: &[u8] = &pub_key_vec[..];
        let pub_key_pk = PublicKey::parse_slice(pub_key_u8, Some(PublicKeyFormat::Full)).unwrap();

        signer_verify(&digested_message, &sig, &pub_key_pk)
    }

    pub fn verifynew(message: String, signature64: String, pub_key64: String) -> bool {

        let signature_vec: Vec<u8> = base64::decode(signature64.as_bytes()).unwrap();
        let pub_key_vec: Vec<u8> = base64::decode(pub_key64.as_bytes()).unwrap();
        
        let message_u8 = message.as_bytes();

        let digested = Sha256::digest(message_u8);
        let digested_vec: Vec<u8> = digested.to_vec();
        
        
        let output: bool = unsafe { crate::ECDSA.verify(pub_key_vec.clone(), signature_vec.clone(), digested_vec.clone()) };

        output
    }
}

#[cfg(test)]
pub mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const D: &str = "TFuxm1wXoGUlO+CDpJkw+9kUc8YPc1k4nisoC1y6/J4=";
    const XY: &str =
        "BNpc0uIAkafgMJBcSVJByl7ejx4rKgTDxijwM1mGMXwkZiGu2CIQ7XPa9SImgqSs2H8tQqQssYNPzNNu07tVUJI=";

    #[test]
    fn it_should_signer_sign_verify_1() {
        // let data_serde: serde_json::Value = json!({
        //     "id" : "did:self:0x0123456789012345678901234567890123456789"
        // });
        let data = "did:self:0x0123456789012345678901234567890123456789";
        let data: &str = &data.to_string();
        let signature: String = Signer::sign(data.to_string(), D.to_string());
        println!("{:?}",data.clone());
        println!("{}",signature.clone());
        let verified: bool = Signer::verify(data.to_string(), signature.clone(), XY.to_string());
        println!("{}   {}",signature.clone(),verified.clone());
        assert!(verified);

    }
}
