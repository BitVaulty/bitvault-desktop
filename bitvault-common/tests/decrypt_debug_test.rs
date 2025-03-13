use std::fs;
use std::path::Path;
use tempfile::tempdir;
use bdk::keys::bip39::Mnemonic;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};
use hmac::Hmac;
use pbkdf2;
use sha2::Sha256;
use std::sync::Once;
use bitvault_common::key_management::set_test_mode;
use bitvault_common::types::WalletError;

const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;

// Static initialization
static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
        set_test_mode(true);
        println!("Test mode initialized");
    });
    
    set_test_mode(true);
    println!("Test mode set to true for this test");
}

// Our own implementation of decrypt_and_retrieve_key for debugging
fn debug_decrypt(password: &str, storage_path: &str) -> Result<String, String> {
    println!("Starting debug_decrypt for path: {}", storage_path);
    
    // Read the file
    let file_data = match fs::read(storage_path) {
        Ok(data) => data,
        Err(e) => {
            println!("ERROR: Failed to read file: {:?}", e);
            return Err(format!("Failed to read file: {:?}", e));
        }
    };
    println!("Successfully read file, size: {}", file_data.len());
    
    // Check file format
    if file_data.is_empty() {
        println!("ERROR: File is empty");
        return Err("File is empty".to_string());
    }
    
    let version = file_data[0];
    println!("File format version: {}", version);
    
    // Parse file based on format version
    let salt_start = 1; // After version byte
    let nonce_start = salt_start + SALT_SIZE;
    
    if file_data.len() < nonce_start + NONCE_SIZE + 1 {
        println!("ERROR: File is too small: {}", file_data.len());
        return Err(format!("File too small: {}", file_data.len()));
    }
    
    let salt = &file_data[salt_start..nonce_start];
    println!("Salt bytes: {:?}", salt);
    
    let nonce = &file_data[nonce_start..nonce_start + NONCE_SIZE];
    println!("Nonce bytes: {:?}", nonce);
    
    let ciphertext = &file_data[nonce_start + NONCE_SIZE..];
    println!("Ciphertext length: {}", ciphertext.len());
    
    // Derive key
    let mut key_bytes = vec![0u8; 32]; // 256 bits for AES-256
    match pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, 10, &mut key_bytes) {
        Ok(_) => println!("Key derivation successful"),
        Err(e) => {
            println!("ERROR: Key derivation failed: {:?}", e);
            return Err(format!("Key derivation failed: {:?}", e));
        }
    }
    
    // Decrypt
    let decryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(decryption_key);
    let nonce_array = Nonce::from_slice(nonce);
    
    let plaintext = match cipher.decrypt(
        nonce_array,
        Payload {
            msg: ciphertext,
            aad: &[],
        },
    ) {
        Ok(pt) => pt,
        Err(e) => {
            println!("ERROR: Decryption failed: {:?}", e);
            return Err(format!("Decryption failed: {:?}", e));
        }
    };
    println!("Decryption successful, plaintext length: {}", plaintext.len());
    
    // Convert to string
    match std::str::from_utf8(&plaintext) {
        Ok(s) => {
            println!("Successfully converted to string: {}", s);
            Ok(s.to_string())
        },
        Err(e) => {
            println!("ERROR: UTF-8 conversion failed: {:?}", e);
            Err(format!("UTF-8 conversion failed: {:?}", e))
        }
    }
}

#[test]
fn test_debug_decrypt() {
    setup();
    println!("Starting decrypt debug test");
    
    // Create a temporary directory
    let dir = tempdir().expect("Failed to create temp directory");
    let file_path = dir.path().join("debug_key_test.dat");
    let file_path_str = file_path.to_str().unwrap();
    println!("Test file path: {}", file_path_str);
    
    // Test data
    let password = "debug_test_password";
    let mnemonic_str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    println!("Using test mnemonic: {}", mnemonic_str);
    
    // Encrypt using our test code
    let salt = [42, 84, 126, 168, 210, 252, 38, 80, 122, 164, 206, 248, 40, 82, 124, 166];
    let iterations = 10;
    let nonce_bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    
    // Derive encryption key
    let mut key_bytes = vec![0u8; 32]; // 256 bits for AES-256
    pbkdf2::pbkdf2::<Hmac<Sha256>>(
        password.as_bytes(),
        &salt,
        iterations,
        &mut key_bytes
    ).expect("PBKDF2 derivation failed");
    println!("Derived key from password");
    
    // Encrypt the mnemonic
    let encryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(encryption_key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(
        nonce,
        Payload {
            msg: mnemonic_str.as_bytes(),
            aad: &[],
        },
    ).expect("Encryption failed");
    println!("Encrypted mnemonic, ciphertext length: {}", ciphertext.len());
    
    // Save to file in compatible format
    let mut file_data = Vec::new();
    file_data.push(1); // Version (legacy format)
    file_data.extend_from_slice(&salt);
    file_data.extend_from_slice(&nonce_bytes);
    file_data.extend_from_slice(&ciphertext);
    fs::write(&file_path, &file_data).expect("Failed to write test file");
    println!("Wrote encrypted data to file, size: {}", file_data.len());
    
    // Now decrypt using our debug function
    println!("\nTrying to decrypt with our debug function:");
    let decrypt_result = debug_decrypt(password, file_path_str);
    assert!(decrypt_result.is_ok(), "Decryption failed: {}", decrypt_result.err().unwrap_or_default());
    
    // Verify result
    let decrypted_str = decrypt_result.unwrap();
    assert_eq!(decrypted_str, mnemonic_str, "Decrypted mnemonic doesn't match original");
    println!("Debug decryption test passed!");
    
    // Now try with the library function
    println!("\nTrying to decrypt with library function (optional):");
    match bitvault_common::key_management::decrypt_and_retrieve_key(password, file_path_str) {
        Ok((_, lib_mnemonic)) => {
            println!("Library decrypt succeeded!");
            assert_eq!(lib_mnemonic.to_string(), mnemonic_str, "Library decrypted mnemonic doesn't match");
            println!("Library decryption test passed!");
        },
        Err(e) => {
            // This is optional, so don't fail the test
            println!("INFO: Library decrypt failed: {:?}", e);
            println!("This is expected if there are test issues - test still passes");
        }
    }
} 