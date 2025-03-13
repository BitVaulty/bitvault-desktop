use std::fs;
use std::path::Path;
use tempfile::tempdir;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};
use generic_array::GenericArray;
use hmac::Hmac;
use pbkdf2;
use sha2::Sha256;

#[test]
fn test_basic_aes_encryption() {
    println!("Starting minimal AES encryption test");
    
    // Create a temporary directory
    let dir = tempdir().expect("Failed to create temp directory");
    let file_path = dir.path().join("crypto_test.dat");
    let file_path_str = file_path.to_str().unwrap();
    println!("Test file path: {}", file_path_str);
    
    // Test data
    let password = "test_password";
    let plaintext = "this is a test message";
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
    
    // Encrypt
    let encryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(encryption_key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(
        nonce,
        Payload {
            msg: plaintext.as_bytes(),
            aad: &[],
        },
    ).expect("Encryption failed");
    println!("Encrypted plaintext, ciphertext length: {}", ciphertext.len());
    
    // Save to file
    let mut file_data = Vec::new();
    file_data.push(1); // Version
    file_data.extend_from_slice(&salt);
    file_data.extend_from_slice(&nonce_bytes);
    file_data.extend_from_slice(&ciphertext);
    fs::write(&file_path, &file_data).expect("Failed to write test file");
    println!("Wrote encrypted data to file");
    
    // Read back from file
    let read_data = fs::read(&file_path).expect("Failed to read test file");
    println!("Read back file, size: {}", read_data.len());
    
    // Parse the file format
    let salt_start = 1;
    let nonce_start = salt_start + salt.len();
    let ciphertext_start = nonce_start + nonce_bytes.len();
    
    let read_salt = &read_data[salt_start..nonce_start];
    let read_nonce = &read_data[nonce_start..ciphertext_start];
    let read_ciphertext = &read_data[ciphertext_start..];
    
    // Derive key again
    let mut key_bytes2 = vec![0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha256>>(
        password.as_bytes(),
        read_salt,
        iterations,
        &mut key_bytes2
    ).expect("PBKDF2 derivation failed");
    
    // Decrypt
    let decryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes2);
    let cipher2 = Aes256Gcm::new(decryption_key);
    let nonce2 = Nonce::from_slice(read_nonce);
    let decrypted_bytes = cipher2.decrypt(
        nonce2,
        Payload {
            msg: read_ciphertext,
            aad: &[],
        },
    ).expect("Decryption failed");
    println!("Decrypted ciphertext");
    
    // Convert to string
    let decrypted_text = std::str::from_utf8(&decrypted_bytes).expect("Invalid UTF-8");
    println!("Decrypted text: {}", decrypted_text);
    
    // Verify
    assert_eq!(decrypted_text, plaintext, "Decrypted text should match original plaintext");
    println!("Decryption successful - text matches");
    
    println!("Minimal AES encryption test passed");
} 