pub const TAG_PUBKEY_EVEN: u8 = 0x02;
pub const TAG_PUBKEY_ODD: u8 = 0x03;
pub const TAG_PUBKEY_FULL: u8 = 0x04;
pub const TAG_PUBKEY_HYBRID_EVEN: u8 = 0x06;
pub const TAG_PUBKEY_HYBRID_ODD: u8 = 0x07;

pub const MESSAGE_SIZE: usize = 32;
pub const SECRET_KEY_SIZE: usize = 32;
pub const RAW_PUBLIC_KEY_SIZE: usize = 64;
pub const FULL_PUBLIC_KEY_SIZE: usize = 65;
pub const COMPRESSED_PUBLIC_KEY_SIZE: usize = 33;
pub const SIGNATURE_SIZE: usize = 64;
pub const DER_MAX_SIGNATURE_SIZE: usize = 72;