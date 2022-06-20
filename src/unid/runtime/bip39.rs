use bip39::Mnemonic;

pub struct BIP39 {}

impl BIP39 {
    pub fn generate_mnemonic(_strength: usize) -> String {
        let mnemonic_type = match strength {
            12 => MnemonicType::Words12,
            15 => MnemonicType::Words15,
            18 => MnemonicType::Words18,
            21 => MnemonicType::Words21,
            24 => MnemonicType::Words24,
            _ => panic!("wrong strength size"),
        };

        let mnemonic = Mnemonic::generate(12).unwrap();

        mnemonic.to_string()
    }

    pub fn mnemonic_to_seed(mnemonic_string: String, password_option: Option<String>) -> Vec<u8> {
        let password: String = match password_option {
            Some(v) => v,
            None => "".to_string(),
        };

        let mnemonic: Mnemonic = Mnemonic::from_phrase(&mnemonic_string, Language::English).unwrap();
        let seed = Seed::new(&mnemonic, &password);
        let seed_bytes: &[u8] = seed.as_bytes();

        seed_bytes.to_vec()
    }
}
