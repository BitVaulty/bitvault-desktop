use anyhow::Result;
use bip39::{Language, Mnemonic};
use rand::RngCore;
use zeroize::Zeroize;

pub fn new_12_word_seed() -> Result<String> {
    let mut entropy = [0u8; 16];
    rand::rng().fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
    entropy.zeroize();
    Ok(mnemonic.to_string())
}

// pub fn new_24_word_seed() -> Result<String> {
//     let mut entropy = [0u8; 32];
//     rand::thread_rng().fill_bytes(&mut entropy);
//     let mnemonic = Mnemonic::from_entropy(&entropy)?;
//     entropy.zeroize();
//     Ok(mnemonic.to_string())
// }
