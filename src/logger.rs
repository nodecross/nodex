use cstr_core::{CString, c_char};
use alloc::{format, string::String};

#[repr(u32)]
enum LogLevel {
    #[allow(dead_code)]
    Emerg   = 0x00,
    #[allow(dead_code)]
    Alert   = 0x10,
    #[allow(dead_code)]
    Crit    = 0x20,
    #[allow(dead_code)]
    Err     = 0x30,
    #[allow(dead_code)]
    Warning = 0x40,
    #[allow(dead_code)]
    Notice  = 0x50,
    #[allow(dead_code)]
    Info    = 0x60,
    Debug   = 0x70,
}

pub struct Logger {
    handler: Option<extern "C" fn(u32, *mut c_char)>
}

impl Logger {
    pub const fn new(handler: Option<&extern "C" fn(u32, *mut c_char)>) -> Logger {
        if let Some(..) = handler {
            Logger {
                handler: Some(*(handler.unwrap())),
            }
        } else {
            Logger {
                handler: None,
            }
        }
    }

    /// # Safety
    unsafe fn write<S: Into<String>>(&self, level: LogLevel, message: S) {
        if let Some(..) = self.handler {
            let handler = self.handler.unwrap();

            let m = message.into();
            let c = CString::new(m).unwrap();
            let ptr = c.into_raw();

            handler(level as u32, ptr);

            let _ = CString::from_raw(ptr);
        }
    }

    /// # Safety
    #[allow(dead_code)]
    pub unsafe fn emerg<S: Into<String>>(&self, message: S) {
        self.write(LogLevel::Emerg, format!("{} {}", "[ERR]", message.into()));
    }

    /// # Safety
    #[allow(dead_code)]
    pub unsafe fn alert<S: Into<String>>(&self, message: S) {
        self.write(LogLevel::Alert, format!("{} {}", "[ERR]", message.into()));
    }

    /// # Safety
    #[allow(dead_code)]
    pub unsafe fn crit<S: Into<String>>(&self, message: S) {
        self.write(LogLevel::Crit, format!("{} {}", "[ERR]", message.into()));
    }

    /// # Safety
    #[allow(dead_code)]
    pub unsafe fn err<S: Into<String>>(&self, message: S) {
        self.write(LogLevel::Err, format!("{} {}", "[ERR]", message.into()));
    }

    /// # Safety
    #[allow(dead_code)]
    pub unsafe fn warn<S: Into<String>>(&self, message: S) {
        self.write(LogLevel::Warning, format!("{} {}", "[WARN]", message.into()));
    }

    /// # Safety
    #[allow(dead_code)]
    pub unsafe fn notice<S: Into<String>>(&self, message: S) {
        self.write(LogLevel::Notice, format!("{} {}", "[ERR]", message.into()));
    }

    /// # Safety
    #[allow(dead_code)]
    pub unsafe fn info<S: Into<String>>(&self, message: S) {
        self.write(LogLevel::Info, format!("{} {}", "[INFO]", message.into()));
    }

    /// # Safety
    pub unsafe fn debug<S: Into<String>>(&self, message: S) {
        self.write(LogLevel::Debug, format!("{} {}", "[DEBUG]", message.into()));
    }
}