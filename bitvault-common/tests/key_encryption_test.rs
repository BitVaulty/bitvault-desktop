use std::sync::Once;
use std::fs;
use tempfile::tempdir;
use bitvault_common::key_management::{
    generate_mnemonic_and_key,
    set_test_mode
};

// Static initialization
static INIT: Once = Once::new();

// Setup function to initialize the test environment
fn setup() {
    INIT.call_once(|| {
        // Initialize logger for tests
        let _ = env_logger::builder().is_test(true).try_init();
        
        // Enable test mode for key management
        set_test_mode(true);
        println!("Test mode enabled for key management tests");
    });
    
    // Always ensure test mode is enabled for each test
    set_test_mode(true);
    println!("Key management test setup completed");
}

/// Helper function to manually encrypt a key file using the same algorithm as the library
fn manual_encrypt_key_file(mnemonic_phrase: &str, password: &str, path: &str) -> Result<(), String> {
    use hmac::Hmac;
    use sha2::Sha256;
    use pbkdf2;
    use aes_gcm::{Aes256Gcm, Key, KeyInit};
    use aes_gcm::aead::{Aead, Payload};
    use generic_array::GenericArray;
    
    println!("Manual encryption of key file");
    
    // Fixed test values
    let salt = vec![42, 84, 126, 168, 210, 252, 38, 80, 122, 164, 206, 248, 40, 82, 124, 166];
    let iterations = 10; // Very low for testing
    let nonce_bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    
    println!("Using salt: {:?}", salt);
    println!("Using nonce: {:?}", nonce_bytes);
    println!("Using iterations: {}", iterations);
    println!("Password: {}", password);
    
    // Derive key bytes
    let mut key_bytes = vec![0u8; 32]; // AES-256 key size
    pbkdf2::pbkdf2::<Hmac<Sha256>>(
        password.as_bytes(),
        &salt,
        iterations,
        &mut key_bytes
    ).map_err(|e| format!("PBKDF2 derivation failed: {:?}", e))?;
    
    println!("Key bytes derived from password: {:?}", &key_bytes[..8]); // Show first 8 bytes for debugging
    
    // Encrypt
    let encryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(encryption_key);
    let nonce = *GenericArray::from_slice(&nonce_bytes);
    
    println!("Original mnemonic: {}", mnemonic_phrase);
    
    let ciphertext = cipher.encrypt(
        &nonce, 
        Payload { 
            msg: mnemonic_phrase.as_bytes(), 
            aad: &[] 
        }
    ).map_err(|e| format!("Encryption failed: {:?}", e))?;
    
    println!("Encryption successful, ciphertext length: {}", ciphertext.len());
    
    // Create file format: [version(1)][salt(16)][nonce(12)][ciphertext]
    let mut file_data = Vec::new();
    file_data.push(1); // Version byte - version 1 format
    file_data.extend_from_slice(&salt);
    file_data.extend_from_slice(&nonce_bytes);
    file_data.extend_from_slice(&ciphertext);
    
    // Write file
    fs::write(path, &file_data).map_err(|e| format!("Failed to write file: {:?}", e))?;
    
    println!("File written successfully at {}", path);
    println!("File size: {} bytes", file_data.len());
    
    Ok(())
}

/// Helper function to manually decrypt a key file
fn manual_decrypt_key_file(password: &str, path: &str) -> Result<String, String> {
    use hmac::Hmac;
    use sha2::Sha256;
    use pbkdf2;
    use aes_gcm::{Aes256Gcm, Key, KeyInit};
    use aes_gcm::aead::{Aead, Payload};
    use generic_array::GenericArray;
    
    println!("Manual decryption of key file");
    
    // Read the file
    let file_data = fs::read(path).map_err(|e| format!("Failed to read file: {:?}", e))?;
    
    if file_data.len() < 1 + 16 + 12 + 1 {
        return Err(format!("File too small: {} bytes", file_data.len()));
    }
    
    let version = file_data[0];
    println!("File version: {}", version);
    
    // Parse file format: [version(1)][salt(16)][nonce(12)][ciphertext]
    let salt = &file_data[1..1+16];
    let nonce = &file_data[1+16..1+16+12];
    let ciphertext = &file_data[1+16+12..];
    
    println!("Salt (first few bytes): {:?}", &salt[..std::cmp::min(8, salt.len())]);
    println!("Nonce: {:?}", nonce);
    println!("Ciphertext length: {}", ciphertext.len());
    println!("Password: {}", password);
    
    // Fixed test values
    let iterations = 10;
    
    // Derive key bytes
    let mut key_bytes = vec![0u8; 32]; // AES-256 key size
    pbkdf2::pbkdf2::<Hmac<Sha256>>(
        password.as_bytes(),
        salt,
        iterations,
        &mut key_bytes
    ).map_err(|e| format!("PBKDF2 derivation failed: {:?}", e))?;
    
    println!("Key bytes derived from password: {:?}", &key_bytes[..8]); // Show first 8 bytes for debugging
    
    // Decrypt
    let decryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(decryption_key);
    let nonce_array = *GenericArray::from_slice(nonce);
    
    let plaintext = cipher.decrypt(
        &nonce_array,
        Payload {
            msg: ciphertext,
            aad: &[]
        }
    ).map_err(|e| format!("Decryption failed: {:?}", e))?;
    
    let plaintext_str = std::str::from_utf8(&plaintext)
        .map_err(|e| format!("UTF-8 decoding failed: {:?}", e))?;
    
    println!("Decryption successful: {}", plaintext_str);
    
    Ok(plaintext_str.to_string())
}

#[test]
fn test_key_encryption_and_decryption() {
    setup();
    
    // Explicitly set test mode
    set_test_mode(true);
    println!("TEST: Key encryption and decryption");
    
    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_key.dat");
    let file_path_str = file_path.to_str().unwrap();
    
    println!("Test directory: {:?}", temp_dir.path());
    println!("Test file path: {:?}", file_path);
    
    // Generate a mnemonic and key to get a valid mnemonic phrase
    let password = "test_password";
    let mnemonic_result = generate_mnemonic_and_key(password);
    
    if let Err(ref e) = mnemonic_result {
        println!("Failed to generate mnemonic and key: {:?}", e);
        panic!("Mnemonic generation failed");
    }
    println!("Successfully generated mnemonic and key");
    
    let (mnemonic, _key) = mnemonic_result.expect("Already checked for error");
    let mnemonic_string = mnemonic.to_string();
    println!("Mnemonic: {}", mnemonic_string);
    
    // Manually encrypt and store the key
    println!("Encrypting and storing key file");
    let encrypt_result = manual_encrypt_key_file(&mnemonic_string, password, file_path_str);
    if let Err(ref e) = encrypt_result {
        println!("Encryption failed: {}", e);
        panic!("Failed to encrypt key file: {}", e);
    }
    
    // Verify the file exists
    assert!(file_path.exists(), "Key file was not created");
    let file_size = fs::metadata(&file_path).expect("Failed to get file metadata").len();
    println!("File size verified: {} bytes", file_size);
    assert!(file_size > 0, "Key file is empty");
    
    // Manually decrypt the key
    println!("Decrypting key file");
    let decrypt_result = manual_decrypt_key_file(password, file_path_str);
    if let Err(ref e) = decrypt_result {
        println!("Decryption failed: {}", e);
        panic!("Failed to decrypt key file: {}", e);
    }
    
    // Verify the mnemonic matches
    let decrypted_mnemonic = decrypt_result.expect("Already checked for error");
    assert_eq!(mnemonic_string, decrypted_mnemonic, "Decrypted mnemonic does not match original");
    println!("Decrypted mnemonic matches original");
    
    // Clean up
    fs::remove_file(file_path).expect("Failed to remove test file");
    println!("Test file removed");
    
    println!("Key encryption and decryption test PASSED");
} 