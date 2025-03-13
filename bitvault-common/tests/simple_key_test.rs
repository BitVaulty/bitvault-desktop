use std::sync::Once;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use bitvault_common::key_management::{
    generate_mnemonic_and_key,
    set_test_mode
};
use bdk::keys::bip39::Mnemonic;
use bdk::keys::ExtendedKey;
use bitvault_common::types::WalletError;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};
use generic_array::GenericArray;
use hmac::Hmac;
use pbkdf2;
use sha2::Sha256;
mod test_utils;
use test_utils::{direct_test_store_key, direct_test_decrypt_key};

// Static initialization
static INIT: Once = Once::new();

// Setup function to initialize the test environment
fn setup() {
    INIT.call_once(|| {
        // Initialize logger for tests
        let _ = env_logger::builder().is_test(true).try_init();
    });
    
    // Always ensure test mode is enabled for each test
    set_test_mode(true);
    println!("Test mode set to true");
}

// Ensure test functions are correctly defined
#[test]
fn test_sanity_check() {
    setup();
    println!("Running sanity check test");
    assert!(true);
}

#[test]
fn test_simple_key_encryption() {
    setup();
    println!("Starting simple key encryption test");
    
    // Create a temporary directory for our test file
    let dir = tempdir().expect("Failed to create temp directory");
    let file_path = dir.path().join("simple_test_key.dat");
    let file_path_str = file_path.to_str().unwrap();
    println!("Created temp file path: {}", file_path_str);
    
    // Generate a test key
    let password = "simple_test_password";
    println!("Generating test key with password: {}", password);
    let (mnemonic, _key) = match generate_mnemonic_and_key(password) {
        Ok(result) => {
            println!("Successfully generated test key");
            result
        },
        Err(e) => {
            println!("ERROR: Failed to generate test key: {:?}", e);
            panic!("Failed to generate test key: {:?}", e);
        }
    };
    println!("Successfully generated test key and mnemonic: {}", mnemonic.to_string());
    
    // Encrypt and store the key using our direct test implementation
    println!("Encrypting and storing key");
    let encrypt_result = direct_test_store_key(&mnemonic, password, file_path_str);
    println!("Encrypt result: {:?}", encrypt_result);
    assert!(encrypt_result.is_ok(), "Key encryption failed: {:?}", encrypt_result.err());
    println!("Successfully encrypted key");
    
    // Verify the file exists
    assert!(Path::new(file_path_str).exists(), "Encrypted key file was not created");
    let file_size = fs::metadata(file_path_str).map(|m| m.len()).unwrap_or(0);
    println!("Key file exists with size: {} bytes", file_size);
    
    // Dump file contents for debugging
    match fs::read(file_path_str) {
        Ok(file_data) => {
            println!("File contents (first 32 bytes): {:02X?}", &file_data[..std::cmp::min(32, file_data.len())]);
            if file_data.len() > 32 {
                println!("... and {} more bytes", file_data.len() - 32);
            }
        },
        Err(e) => {
            println!("ERROR: Failed to read encrypted key file: {:?}", e);
        }
    }
    
    // Decrypt the key using our direct test decryption function
    println!("Decrypting key");
    let decrypt_result = direct_test_decrypt_key(password, file_path_str);
    
    // Detailed error handling
    match decrypt_result {
        Ok((_decrypted_key, decrypted_mnemonic)) => {
            println!("Successfully decrypted key");
            
            // Verify the mnemonic is the same as the original
            assert_eq!(mnemonic.to_string(), decrypted_mnemonic.to_string(), 
                      "Decrypted mnemonic doesn't match original");
            println!("Mnemonic verification successful");
            
            println!("Simple key encryption test passed");
        },
        Err(err) => {
            println!("Decryption failed: {:?}", err);
            assert!(false, "Key decryption failed: {:?}", err);
        }
    }
}

// Add a panic hook to print panic messages
fn main() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("Panic occurred: {:?}", info);
    }));
    // Run tests
    test_sanity_check();
    test_simple_key_encryption();
} 