use core::slice;
use alloc::vec::Vec;


pub struct DataT {
  pub bytes: *mut u8,
  pub bytes_length: u32
}

#[no_mangle]
pub fn c_to_rust_bytes(bytes: *mut u8, bytes_length: u32) -> Vec<u8> {
  let bytes_u8: &[u8] = unsafe {slice::from_raw_parts_mut(bytes, bytes_length as usize)};
  let bytes_vec: Vec<u8> = Vec::from(bytes_u8);
  bytes_vec
}

#[no_mangle]
pub fn rust_to_c_bytes(bytes_vec: Vec<u8>) ->  DataT {
  let (ptr, len, _cap) = bytes_vec.into_raw_parts();

  let ptr = ptr as *mut u8;

  DataT {
    bytes: ptr,
    bytes_length: len as u32
  }
}