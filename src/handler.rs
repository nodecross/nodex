use crate::allocator;
use cstr_core::c_char;

pub struct UNiDHandler {
    memory_alloc      : Option<extern "C" fn(u32) -> *mut allocator::c_void>,
    memory_dealloc    : Option<extern "C" fn(*mut allocator::c_void)>,
    debug_message     : Option<extern "C" fn(u32, *mut c_char)>,
    crypto_trng       : Option<extern "C" fn(u32) -> *mut c_char>,
    https_post_request: Option<extern "C" fn(*mut c_char, *mut c_char, *mut c_char) -> *mut c_char>,
}

impl UNiDHandler {
    pub const fn new() -> UNiDHandler {
        UNiDHandler {
            memory_alloc      : None,
            memory_dealloc    : None,
            debug_message     : None,
            crypto_trng       : None,
            https_post_request: None,
        }
    }

    // setter: memory_alloc_handler
    pub fn set_memory_alloc_handler(&mut self, handler: extern "C" fn(u32) -> *mut allocator::c_void) {
        self.memory_alloc = Some(handler)
    }

    // getter: memory_alloc_handler
    pub fn get_memory_alloc_handler(&self) -> Option<extern "C" fn(u32) -> *mut allocator::c_void> {
        self.memory_alloc
    }

    // setter: memory_dealloc_handler
    pub fn set_memory_dealloc_handler(&mut self, handler: extern "C" fn(*mut allocator::c_void)) {
        self.memory_dealloc = Some(handler)
    }

    // getter: memory_dealloc_handler
    pub fn get_memory_dealloc_handler(&self) -> Option<extern "C" fn(*mut allocator::c_void)> {
        self.memory_dealloc
    }

    // setter: debug_message_handler
    pub fn set_debug_message_handler(&mut self, handler: extern "C" fn(u32, *mut c_char)) {
        self.debug_message = Some(handler)
    }

    // getter: debug_message_handler
    pub fn get_debug_message_handler(&self) -> Option<extern "C" fn(u32, *mut c_char)> {
        self.debug_message
    }

    // setter: crypto_trng
    pub fn set_crypto_trng(&mut self, handler: extern "C" fn(u32) -> *mut c_char) {
        self.crypto_trng = Some(handler)
    }

    // getter: crypto_trng
    pub fn get_crypto_trng(&self) -> Option<extern "C" fn(u32) -> *mut c_char> {
        self.crypto_trng
    }

    // setter: https_get_request
    pub fn set_https_post_request(&mut self, handler: extern "C" fn(*mut c_char, *mut c_char, *mut c_char) -> *mut c_char) {
        self.https_post_request = Some(handler)
    }

    // getter: https_post_request
    pub fn get_https_post_request(&self) -> Option<extern "C" fn(*mut c_char, *mut c_char, *mut c_char) -> *mut c_char> {
        self.https_post_request
    }
}