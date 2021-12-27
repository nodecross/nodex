use alloc::{format, string::ToString};
use crate::unid::runtime::secp256k1::Secp256k1;
use crate::logger::Logger;
use crate::MUTEX_HANDLERS;
// use crate::ffi::FFI;
use crate::unid::utils::random::Random;
// use alloc::vec::Vec;

#[no_mangle]
pub unsafe extern "C" fn unid_test() {
    let logger = Logger::new(MUTEX_HANDLERS.lock().get_debug_message_handler());

    let random = match Random::trng_bytes(&32) {
        Ok(v) => v,
        Err(_) => panic!()
    };

    let public = match Secp256k1::generate_public_key(&random) {
        Ok(v) => v,
        Err(_) => panic!()
    };

    logger.debug(format!("public key = {:?}", public));

    // let handler = MUTEX_HANDLERS.lock().get_crypto_trng();

    // if let Some(..) = handler {
    //     let random = handler.unwrap()(32);

    //     let output = match FFI::binary_from_ptr(random) {
    //         Ok(v) => v,
    //         Err(_) => Vec::from([0])
    //     };

    //     logger.info(format!("{:?}", output));
    // }
}