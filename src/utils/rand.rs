
use rand::seq::SliceRandom;

// Source to random number
const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";


// generate random string for IV
pub fn gen_ascii_chars(size: usize) -> String {
  let mut rng = &mut rand::thread_rng();
  String::from_utf8(
      BASE_STR.as_bytes()
          .choose_multiple(&mut rng, size)
          .cloned()
          .collect()
  ).unwrap()
}