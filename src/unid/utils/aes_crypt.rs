use core::ptr::null_mut;
use crate::unid::utils::bytes::{ c_to_rust_bytes, rust_to_c_bytes, DataT};
use alloc::vec::Vec;
use crate::allocator;

pub struct AesCrypt {
  encryptor  : extern "C" fn(*mut DataT, *mut DataT, *mut DataT) -> *mut allocator::c_void,
  decryptor : extern "C" fn() -> *mut allocator::c_void,
}

impl AesCrypt {
  pub const fn empty() -> AesCrypt {
    AesCrypt {
      encryptor : AesCrypt::noop_encryptor,
      decryptor : AesCrypt::noop_decryptor
    }
  }

  extern "C" fn noop_encryptor(_plaintext_data_t: *mut DataT, _key_data_t: *mut DataT, _iv_data_t: *mut DataT) -> *mut allocator::c_void {
    null_mut::<allocator::c_void>()
  }

  extern "C" fn noop_decryptor() -> *mut allocator::c_void {
    null_mut::<allocator::c_void>()
  }

  pub fn init(
    &mut self,
    encryptor: extern "C" fn(*mut DataT, *mut DataT, *mut DataT) -> *mut allocator::c_void,
    decryptor: extern "C" fn() -> *mut allocator::c_void
  ) {
    self.encryptor = encryptor;
    self.decryptor = decryptor;
  }

  pub fn encrypt(&self, plaintext_vec: Vec<u8>, key_vec: Vec<u8>, iv_vec: Vec<u8>) -> Vec<u8> {

    let mut plaintext_data_t : DataT = rust_to_c_bytes(plaintext_vec);

    let mut key_data_t : DataT = rust_to_c_bytes(key_vec);

    let mut iv_data_t : DataT = rust_to_c_bytes(iv_vec);

    let encrypted_ptr: *mut u8 = {
      (self.encryptor)(&mut key_data_t, &mut iv_data_t, &mut plaintext_data_t) as *mut u8
    };

    c_to_rust_bytes(encrypted_ptr, 32)
  }
}

