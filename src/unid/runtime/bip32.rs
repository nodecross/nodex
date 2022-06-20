use bip32::{Mnemonic, Seed, XPrv};

#[derive(Debug)]
pub struct BIP32Interface {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

pub struct BIP32 {}

impl BIP32 {
    fn x() {


    }

    // fn convert_to_u8_64(data: &[u8]) -> [u8; 64] {
    //     data.try_into().expect("slice with incorrect length")
    // }

    // pub fn get_node(seed_vec: Vec<u8>, derivation_path_string: String) -> BIP32Interface {
    //     let seed_u8_64: [u8; 64];
    //     if seed_vec.len() == 64 {
    //         let seed_u8: &[u8] = &seed_vec[..];
    //         seed_u8_64 = BIP32::convert_to_u8_64(seed_u8);
    //     } else {
    //         panic!("unable to convert array 64 from slice")
    //     }

    //     let seed = Seed::new(seed_u8_64);
    //     let derivation_path_str: &str = &derivation_path_string;
    //     let child_path = derivation_path_str.parse().unwrap();
    //     let child_xprv = XPrv::derive_from_path(&seed, &child_path).unwrap();
    //     let child_xpub = child_xprv.public_key();
    //     let private_key_bytes: [u8; 32] = child_xprv.to_bytes();
    //     let private_key_u8: &[u8] = &private_key_bytes;
    //     let private_key_vec: Vec<u8> = private_key_u8.to_vec();
    //     let public_key_bytes: [u8; 33] = child_xpub.to_bytes();
    //     let public_key_u8: &[u8] = &public_key_bytes;
    //     let public_key_vec: Vec<u8> = public_key_u8.to_vec();

    //     BIP32Interface {
    //         public_key: public_key_vec,
    //         private_key: private_key_vec,
    //     }
    // }
}
