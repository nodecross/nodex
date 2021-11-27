use crate::allocator;
use cstr_core::c_char;

pub struct UNiDHandler {
    memory_alloc  : Option<extern "C" fn(u32) -> *mut allocator::c_void>,
    memory_dealloc: Option<extern "C" fn(*mut allocator::c_void)>,
    debug_message : Option<extern "C" fn(u32, *mut c_char)>,
}

impl UNiDHandler {
    pub const fn new() -> UNiDHandler {
        UNiDHandler {
            memory_alloc  : None,
            memory_dealloc: None,
            debug_message : None,
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
}