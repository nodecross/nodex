#![no_std]
#![no_main]
#![feature(libc)]
#![feature(default_alloc_error_handler)]
#![feature(const_fn_fn_ptr_basics)]

extern crate alloc;

use alloc::string::ToString;
use linked_list_allocator::LockedHeap;
use cstr_core::{CStr, CString, c_char};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[no_mangle]
pub extern "C" fn libunid_init() {
    let heap_start = 0x20000000;
    let heap_end   = 0x20000000 + (1024 * 10); /* 1024 bytes (1k) x 10 = 10k */
    let heap_size  = heap_end - heap_start;

    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

// UTILS

// CIPHERS
pub mod ciphers;

// TODO: FREE_MEMORY

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn ciphers_hasher_digest(content: *const c_char, secret: *const c_char) -> *mut c_char {
    // v1
    let v1 = {
        assert!(! content.is_null());

        CStr::from_ptr(content)
    };
    let v1_str = v1.to_str().unwrap().to_string();

    // v2
    let v2 = {
        assert!(! secret.is_null());

        CStr::from_ptr(secret)
    };
    let v2_str = v2.to_str().unwrap().to_string();

    // result
    let r = ciphers::hasher::Hasher::digest(v1_str, v2_str);
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

#[no_mangle]
pub extern "C" fn ciphers_hasher_verify(content: c_char, digest: c_char, secret: c_char) -> bool {
    let v1 = content.to_string();
    let v2 = digest.to_string();
    let v3 = secret.to_string();

    ciphers::hasher::Hasher::verify(v1, v2, v3)
}

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
pub extern "C" fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}