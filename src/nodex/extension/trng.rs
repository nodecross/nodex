use crate::{app_config, config::Extension, nodex::runtime::random::Random};
use std::{ffi::CStr, num::NonZeroU32};

use thiserror::Error;

pub struct Trng {}

#[derive(Error, Debug)]
pub enum TrngError {
    #[error("Buffer length over")]
    BufferLengthOver,
    #[error("Library loading error")]
    LibraryLoadingError(#[from] libloading::Error),
    #[error("External function failed")]
    ExternalFunctionFailed(NonZeroU32),
    #[error("Random generation failed")]
    RandomGenerationFailed(Box<dyn std::error::Error>),
}

impl Trng {
    const MAX_BUFFER_LENGTH: usize = 1024;

    pub fn new() -> Trng {
        Trng {}
    }

    fn read_external(&self, extension: &Extension, size: &usize) -> Result<Vec<u8>, TrngError> {
        log::info!("Called: read_external");

        if Trng::MAX_BUFFER_LENGTH < *size {
            return Err(TrngError::BufferLengthOver);
        }

        unsafe {
            let buffer = [0u8; Trng::MAX_BUFFER_LENGTH + 1];
            let buffer_ptr: *const i8 = buffer.as_ptr().cast();

            let lib = libloading::Library::new(&extension.filename)?;

            let func: libloading::Symbol<
                unsafe extern "C" fn(buf: *const i8, bufsize: usize, size: usize) -> u32,
            > = lib.get(extension.symbol.as_bytes())?;

            let result = func(buffer_ptr, buffer.len(), *size);

            if let Some(exit_status) = NonZeroU32::new(result) {
                return Err(TrngError::ExternalFunctionFailed(exit_status));
            }

            Ok(CStr::from_ptr(buffer_ptr as *const core::ffi::c_char)
                .to_bytes()
                .to_vec())
        }
    }

    fn read_internal(&self, size: &usize) -> Result<Vec<u8>, TrngError> {
        log::info!("Called: read_internal");

        Random::bytes(size).map_err(TrngError::RandomGenerationFailed)
    }

    pub fn read(&self, size: &usize) -> Result<Vec<u8>, TrngError> {
        let config = app_config();
        let config = config.inner.lock().unwrap();

        if let Some(ref extension) = config.load_trng_read_sig() {
            self.read_external(extension, size)
        } else {
            self.read_internal(size)
        }
    }
}
