use core::slice;
use alloc::vec::Vec;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct DataT {
    pub ptr: *mut u8,
    pub len: u32
}

impl DataT {
    pub fn new(bytes_vec: Vec<u8>) -> Self {
        let (ptr, len, _cap) = bytes_vec.into_raw_parts();

        // unsafe {
        //   let logger = crate::Logger::new(crate::MUTEX_HANDLERS.lock().get_debug_message_handler());
  
        //   logger.debug(alloc::format!("ptr, len, cap = {:?}, {:?}, {:?}", ptr, len, _cap));
        // }

        DataT {
            ptr: ptr as *mut u8,
            len: len as u32
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let ptr: *mut u8 = self.ptr; 
        let len: u32 = self.len;
        let bytes_u8: &[u8] = unsafe {slice::from_raw_parts_mut(ptr, len as usize)};
        let bytes_vec: Vec<u8> = Vec::from(bytes_u8);

        bytes_vec
    }
}
