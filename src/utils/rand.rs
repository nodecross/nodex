use alloc::string::String;
use rand::seq::SliceRandom;

// Source to random number
const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

// generate random string for IV
pub fn gen_ascii_chars(size: usize) -> String {
    let mut rng = &mut rand::rngs::SmallRng::from_seed([0; 16]);

    String::from_utf8(
        BASE_STR.as_bytes()
            .choose_multiple(&mut rng, size)
            .cloned()
            .collect()
    ).unwrap()
}