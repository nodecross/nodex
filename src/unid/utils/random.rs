use alloc::string::{String, ToString};

// Create small, cheap to initialize and fast RNG with a random seed.
// The randomness is supplied by the operating system.

// Source to random number

pub fn get_random_bytes(length: usize) -> String {
    let mut len = length;
    let base = String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");

    if base.len() < len {
        len = base.len();
    }

    base.get(0..len).unwrap().to_string()
}