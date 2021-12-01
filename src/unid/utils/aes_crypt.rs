use crate::unid::utils::data_t::DataT;
use alloc::vec::Vec;
use crate::MUTEX_HANDLERS;

pub struct AesCrypt {
  encryptor  : extern "C" fn(*mut DataT, *mut DataT, *mut DataT, *mut u8, u32),
  decryptor : extern "C" fn(),
}

impl AesCrypt {
  pub const fn empty() -> AesCrypt {
    AesCrypt {
      encryptor : AesCrypt::noop_encryptor,
      decryptor : AesCrypt::noop_decryptor
    }
  }

  extern "C" fn noop_encryptor(_plaintext_data_t: *mut DataT, _key_data_t: *mut DataT, _iv_data_t: *mut DataT, _encrypt_ptr: *mut u8, _len: u32) {}

  extern "C" fn noop_decryptor() {}

  pub fn init(
    &mut self,
    encryptor: extern "C" fn(*mut DataT, *mut DataT, *mut DataT, *mut u8, u32),
    decryptor: extern "C" fn()
  ) {
    self.encryptor = encryptor;
    self.decryptor = decryptor;
  }

  pub fn encrypt(&self, plaintext_vec: Vec<u8>, key_vec: Vec<u8>, iv_vec: Vec<u8> ) -> Vec<u8> {
    
    // buffer must have enough space for message+padding
    let pos = plaintext_vec.len();
    let len = (pos + 15) & !15;
    let diff = len - pos;
    unsafe {
        let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

        logger.debug(alloc::format!("len, pos, diff = {:?} {:?} {:?}", len, pos, diff));
    }

    let mut plaintext_buffer_vec : Vec<u8> = alloc::vec![diff as u8; len];

    plaintext_buffer_vec[..pos].copy_from_slice(&plaintext_vec);

    unsafe {
        let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
        logger.debug(alloc::format!("buffered text = {:?}", plaintext_buffer_vec));
    }

    let mut plaintext_data_t : DataT = DataT::new(plaintext_buffer_vec);
    unsafe {
      let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
  
      logger.debug(alloc::format!("plaintext_data_t = {:?}", plaintext_data_t.ptr));
    }

    let mut key_data_t : DataT = DataT::new(key_vec);
    unsafe {
      let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
  
      logger.debug(alloc::format!("key_data_t = {:?}", key_data_t.ptr));
    }

    let mut iv_data_t : DataT = DataT::new(iv_vec);
    unsafe {
      let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
  
      logger.debug(alloc::format!("iv_data_t = {:?}", iv_data_t.ptr));
    }

    let encrypt_vec: Vec<u8> = alloc::vec![0u8; len];
    let encrypt_data_t: DataT = DataT::new(encrypt_vec);
    let encrypt_ptr: *mut u8 = encrypt_data_t.ptr;
    let encrypt_len: u32 = encrypt_data_t.len;

    (self.encryptor)(&mut key_data_t, &mut iv_data_t, &mut plaintext_data_t, encrypt_ptr, encrypt_len);

    unsafe {
      let logger = crate::Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());
  
      logger.debug(alloc::format!("encrypt len = {:?}", encrypt_len));
    }

    encrypt_data_t.to_vec()
  }
}


#[cfg(test)]
pub mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
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

    assert_eq!(ciphertext_base64, "ZSPQ8GvQwxwO0iXFZzmOd6+KBe7nAyvSIdqESgezciU=".to_string());
  }
}
