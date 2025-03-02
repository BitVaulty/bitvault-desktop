use anyhow::Result;
use bip39::{Language, Mnemonic};
use zeroize::Zeroize;

pub fn new_12_word_seed() -> Result<String> {
    let mut entropy = [0u8; 16];
    getrandom::fill(&mut entropy)?;
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
    entropy.zeroize();
    Ok(mnemonic.to_string())
}

// pub fn new_24_word_seed() -> Result<String> {
//     let mut entropy = [0u8; 32];
//     getrandom::fill(&mut entropy)?;
//     let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
//     entropy.zeroize();
//     Ok(mnemonic.to_string())
// }
