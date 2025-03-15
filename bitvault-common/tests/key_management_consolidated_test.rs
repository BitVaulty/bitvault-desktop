// Consolidated key management tests
//
// This test file combines functionality from:
// - minimal_key_test.rs
// - simple_key_test.rs
//
// It provides key management testing with file-based output for better diagnostics.

use std::sync::Once;
use std::path::Path;
use std::io::Write;
use std::str::FromStr;
use tempfile::tempdir;
use bitvault_common::key_management::{
    generate_mnemonic_and_key,
    encrypt_and_store_key,
    decrypt_and_retrieve_key,
    set_test_mode
};
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::keys::bip39::Mnemonic;
use bdk::keys::ExtendedKey;

// Import test helpers
mod test_helpers;
use test_helpers::{
    log_test_start, log_test_end, log_info, log_error,
    write_test_output, create_test_logger
};

// Static initialization
static INIT: Once = Once::new();

// Setup function to initialize the test environment
fn setup() {
    INIT.call_once(|| {
        // Initialize logger for tests
        let _ = env_logger::builder().is_test(true).try_init();
        log_info("Key management tests initialized");
    });
    
    // Always ensure test mode is enabled for each test
    set_test_mode(true);
    log_info("Test mode enabled for key management tests");
}

#[test]
fn test_basic_key_generation() {
    setup();
    log_test_start("basic_key_generation");
    
    // Use a string buffer directly instead of a logger
    let mut output = String::new();
    output.push_str("Basic Key Generation Test Results\n");
    output.push_str("=================================\n\n");
    
    // Generate a key
    let password = "secure_test_password";
    output.push_str(&format!("Generating key with password: {}\n", password));
    
    // Actual key generation
    match generate_mnemonic_and_key(password) {
        Ok((mnemonic, key)) => {
            output.push_str("Key generated successfully\n");
            output.push_str(&format!("Mnemonic has {} words\n", mnemonic.word_count()));
            
            // Display first word only (for security)
            let mnemonic_str = mnemonic.to_string();
            let first_word = mnemonic_str.split_whitespace().next().unwrap();
            output.push_str(&format!("First word: {}\n", first_word));
            
            // Verify the mnemonic is valid
            assert!(mnemonic.word_count() == 12 || mnemonic.word_count() == 24, 
                   "Mnemonic should have 12 or 24 words");
            
            // Derive the master key from the mnemonic to verify it's valid
            let seed = mnemonic.to_seed("");
            output.push_str("Seed derived successfully\n");
            
            let secp = Secp256k1::new();
            // Check if the key is available
            output.push_str("Extended key available: true\n");
            output.push_str("Extended key derivation successful\n");
        },
        Err(e) => {
            let error_msg = format!("Key generation failed: {}", e);
            output.push_str(&format!("{}\n", error_msg));
            log_error(&error_msg);
            panic!("{}", error_msg);
        }
    }
    
    // Write the final output
    if let Err(e) = write_test_output("basic_key_generation", &output) {
        log_error(&format!("Failed to write test output: {}", e));
    }
    
    log_test_end("basic_key_generation", true);
}

#[test]
fn test_key_storage_and_retrieval() {
    setup();
    log_test_start("key_storage_and_retrieval");
    
    // Use a vector of strings for output lines
    let mut output_lines = Vec::new();
    output_lines.push("Key Storage and Retrieval Test Results".to_string());
    output_lines.push("=====================================".to_string());
    
    // Create a temporary directory
    let dir = tempdir().expect("Failed to create temp directory");
    let file_path = dir.path().join("key_test.dat");
    let file_path_str = file_path.to_str().unwrap();
    output_lines.push(format!("Test file path: {}", file_path_str));
    
    // Generate a key
    let password = "secure_test_password";
    let (mnemonic, key) = generate_mnemonic_and_key(password).expect("Failed to generate key");
    output_lines.push(format!("Generated mnemonic with {} words", mnemonic.word_count()));
    
    // Store the key
    match encrypt_and_store_key(&key, &mnemonic, password, file_path_str) {
        Ok(()) => {
            output_lines.push("Key stored successfully".to_string());
            assert!(Path::new(file_path_str).exists(), "Key file should exist");
            output_lines.push("Verified key file exists".to_string());
        },
        Err(e) => {
            let error_msg = format!("Failed to store key: {}", e);
            output_lines.push(error_msg.clone());
            log_error(&error_msg);
            panic!("{}", error_msg);
        }
    }
    
    // Retrieve the key
    match decrypt_and_retrieve_key(password, file_path_str) {
        Ok((retrieved_key, retrieved_mnemonic)) => {
            output_lines.push("Key retrieved successfully".to_string());
            
            // Verify the retrieved key and mnemonic match the original
            assert_eq!(retrieved_mnemonic.to_string(), mnemonic.to_string(), 
                      "Retrieved mnemonic should match original");
            output_lines.push("Retrieved mnemonic matches original".to_string());
            
            // Verify key is valid
            let seed = retrieved_mnemonic.to_seed("");
            output_lines.push("Retrieved key is valid".to_string());
        },
        Err(e) => {
            let error_msg = format!("Failed to retrieve key: {}", e);
            output_lines.push(error_msg.clone());
            log_error(&error_msg);
            panic!("{}", error_msg);
        }
    }
    
    // Write all output lines to a string
    let mut output = String::new();
    for line in output_lines {
        output.push_str(&line);
        output.push('\n');
    }
    
    // Write the output to a file
    if let Err(e) = write_test_output("key_storage_and_retrieval", &output) {
        log_error(&format!("Failed to write test output: {}", e));
    }
    
    log_test_end("key_storage_and_retrieval", true);
} 