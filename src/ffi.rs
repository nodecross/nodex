use cstr_core::{c_char, CStr, CString};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::unid::errors::UNiDError;
use crate::MUTEX_HANDLERS;
use crate::logger::Logger;

#[allow(dead_code)]
pub struct Ffi {
}

impl Ffi {
    pub fn binary_from_ptr(ptr: *const c_char) -> Result<Vec<u8>, UNiDError> {
        let logger = unsafe { Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler()) };

        let value = unsafe {
            if ptr.is_null() {
                logger.err("is_null()");

                return Err(UNiDError{})
            }

            CStr::from_ptr(ptr)
        };

        Ok(value.to_bytes().to_vec())
    }

    // C-style string value (char*) to Rust-style string value
    pub fn string_from_ptr(ptr: *const c_char) -> Result<String, UNiDError> {
        let logger = unsafe { Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler()) };

        let value = unsafe {
            if ptr.is_null() {
                logger.err("is_null()");

                return Err(UNiDError{});
            }

            CStr::from_ptr(ptr)
        };

        let str = match value.to_str() {
            Ok(v) => v,
            Err(_) => unsafe {
                logger.err("to_str()");

                return Err(UNiDError{})
            }
        };

        Ok(str.to_string())
    }

    // Rust-style string value to C-style string value (char*)
    pub fn string_to_ptr(str: String) -> Result<*mut c_char, UNiDError> {
        let logger = unsafe { Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler()) };

        let c_str = match CString::new(str) {
            Ok(v) => v,
            Err(_) => unsafe {
                logger.err("CString::new()");

                return Err(UNiDError{})
            }
        };

        Ok(c_str.into_raw())
    }

    pub fn binary_to_ptr(binary: &[u8]) -> Result<*mut c_char, UNiDError> {
        let logger = unsafe { Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler()) };

        let c_str = match CString::new(binary) {
            Ok(v) => v,
            Err(_) => unsafe {
                logger.err("CString::new()");

                return Err(UNiDError{})
            }
        };

        Ok(c_str.into_raw())
    }

    pub fn disposer(ptr: *mut c_char) {
        let _ = unsafe { CString::from_raw(ptr) };
    }
}