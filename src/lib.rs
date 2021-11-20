#![no_std]
#![no_main]
#![feature(libc)]
#![feature(default_alloc_error_handler)]
#![feature(const_fn_fn_ptr_basics)]

extern crate alloc;

mod unid;
mod allocator;

use alloc::string::{String, ToString};
use cstr_core::{CStr, CString, c_char};

#[global_allocator]
static mut ALLOCATOR: allocator::ExternalHeap = allocator::ExternalHeap::empty();

#[repr(C)]
pub struct UNiDConfig {
    client_id: *const c_char,
    client_secret: *const c_char,
}

#[repr(C)]
pub struct UNiDContext {
    client_id: *const c_char,
    client_secret: *const c_char,
}

/// [TODO]: FREE_MEMORY

/// unid :: init
#[no_mangle]
pub extern "C" fn unid_init(config: UNiDConfig, allocator: extern "C" fn(u32) -> *mut allocator::c_void, deallocator: extern "C" fn(*mut allocator:: c_void)) -> UNiDContext {
    unsafe {
        ALLOCATOR.init(allocator, deallocator);
    }

    // build context then return
    UNiDContext {
        client_id    : config.client_id,
        client_secret: config.client_secret,
    }
}

/// unid :: core :: create_did
#[no_mangle]
pub extern "C" fn unid_core_create_did(_context: UNiDContext) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: core :: resolve_did
#[no_mangle]
pub extern "C" fn unid_core_resolve_did(_context: UNiDContext) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: core :: update_did
#[no_mangle]
pub extern "C" fn unid_core_update_did(_context: UNiDContext) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: core :: revoke_did
#[no_mangle]
pub extern "C" fn unid_core_revoke_did(_context: UNiDContext) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: core :: verify_credentials
#[no_mangle]
pub extern "C" fn unid_core_verify_credentials(_context: UNiDContext) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: core :: verify_presentations
#[no_mangle]
pub extern "C" fn unid_core_verify_presentations(_context: UNiDContext) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: did :: create_credentials
#[no_mangle]
pub extern "C" fn unid_did_create_credentials(_context: UNiDContext) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: did :: create_presentations
#[no_mangle]
pub extern "C" fn unid_did_create_presentations(_context: UNiDContext) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: runtime :: bip39 :: generate_mnemonic
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_runtime_bip39_generate_mnemonic() -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: utils :: random :: get_random_bytes
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_utils_random_get_random_bytes(_length: i32) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: utils :: codec :: base64_encode
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_utils_codec_base64_encode(content: *const c_char) -> *mut c_char {
    let v1 = {
        assert!(! content.is_null());

        CStr::from_ptr(content)
    };
    let v1_str = v1.to_str().unwrap().to_string();

    let r = unid::utils::codec::Codec::base64_encode(v1_str);
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: utils :: codec :: base64_decode
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_utils_codec_base64_decode(content: *const c_char) -> *mut c_char {
    let v1 = {
        assert!(! content.is_null());

        CStr::from_ptr(content)
    };
    let v1_str = v1.to_str().unwrap().to_string();

    let r = unid::utils::codec::Codec::base64_decode(v1_str);
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: utils :: multihasher :: hash
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_utils_multihasher_hash(_content: *const c_char) -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: ciphers :: signer :: sign
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_ciphers_signer_sign() -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: ciphers :: signer :: verify
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_ciphers_signer_verify() -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: ciphers :: cipher :: encrypt
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_ciphers_cipher_encrypt() -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: ciphers :: cipher :: decrypt
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_ciphers_cipher_decrypt() -> *mut c_char {
    let r = String::from("WIP_FOR_ROT");
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: ciphers :: hasher :: digest
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_ciphers_hasher_digest(content: *const c_char, secret: *const c_char) -> *mut c_char {
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
    let r = unid::ciphers::hasher::Hasher::digest(v1_str, v2_str);
    let r_c_str = CString::new(r).unwrap();

    r_c_str.into_raw()
}

/// unid :: ciphers :: hasher :: verify
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn unid_ciphers_hasher_verify(content: *const c_char, digest: *const c_char, secret: *const c_char) -> bool {
    // v1
    let v1 = {
        assert!(! content.is_null());

        CStr::from_ptr(content)
    };
    let v1_str = v1.to_str().unwrap().to_string();

    // v2
    let v2 = {
        assert!(! digest.is_null());

        CStr::from_ptr(digest)
    };
    let v2_str = v2.to_str().unwrap().to_string();

    // v3
    let v3 = {
        assert!(! secret.is_null());

        CStr::from_ptr(secret)
    };
    let v3_str = v3.to_str().unwrap().to_string();

    // result
    unid::ciphers::hasher::Hasher::verify(v1_str, v2_str, v3_str)
}

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
pub extern "C" fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}