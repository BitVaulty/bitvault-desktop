use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, ParamsBuilder, Version,
};
use phc::Salt;
use serde::{Deserialize, Serialize};
use rand::RngCore;
use rand::thread_rng;

#[derive(Serialize, Deserialize)]
pub struct EncryptedData {
    ciphertext: String, // hex encoded
    nonce: String,      // hex encoded
    salt: String,       // phc encoded
}

pub fn encrypt_seed(seed: &str, pin: &str) -> Result<String, String> {
    // Generate a random salt
    let mut salt_bytes = [0u8; 16];
    thread_rng().fill_bytes(&mut salt_bytes);
    let salt = Salt::from(&salt_bytes);

    // Configure Argon2id with strong parameters
    let params = ParamsBuilder::new()
        .m_cost(64 * 1024) // 64MB memory cost
        .t_cost(3) // 3 iterations
        .p_cost(4) // 4 parallel threads
        .output_len(32) // 32 bytes output for AES-256
        .build()
        .map_err(|e| format!("Failed to build Argon2 params: {}", e))?;

    // Create Argon2id instance
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    // Create salt string from base64
    let salt =
        SaltString::from_b64(&salt.to_string()).map_err(|e| format!("Invalid salt: {}", e))?;

    // Derive key from PIN
    let key = argon2
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|e| format!("Failed to derive key: {}", e))?
        .hash
        .ok_or("No hash value generated")?
        .as_bytes()
        .to_vec();

    // Create AES-GCM cipher
    let key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the seed
    let ciphertext = cipher
        .encrypt(nonce, seed.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Encode the encrypted data and nonce in base64
    let encrypted_data = EncryptedData {
        ciphertext: hex::encode(ciphertext),
        nonce: hex::encode(nonce_bytes),
        salt: salt.to_string(),
    };

    // Serialize to JSON string
    serde_json::to_string(&encrypted_data).map_err(|e| format!("Serialization failed: {}", e))
}

pub fn decrypt_seed(encrypted_data_str: &str, pin: &str) -> Result<String, String> {
    // Deserialize the encrypted data
    let encrypted_data: EncryptedData = serde_json::from_str(encrypted_data_str)
        .map_err(|e| format!("Failed to parse encrypted data: {}", e))?;

    // Decode the base64 values
    let ciphertext = hex::decode(encrypted_data.ciphertext.as_bytes())
        .map_err(|e| format!("Failed to decode ciphertext: {}", e))?;
    let nonce_bytes = hex::decode(encrypted_data.nonce.as_bytes())
        .map_err(|e| format!("Failed to decode nonce: {}", e))?;

    // Create salt string from stored value
    let salt =
        SaltString::from_b64(&encrypted_data.salt).map_err(|e| format!("Invalid salt: {}", e))?;

    // Configure Argon2id with same parameters
    let params = ParamsBuilder::new()
        .m_cost(64 * 1024) // 64MB memory cost
        .t_cost(3) // 3 iterations
        .p_cost(4) // 4 parallel threads
        .output_len(32) // 32 bytes output for AES-256
        .build()
        .map_err(|e| format!("Failed to build Argon2 params: {}", e))?;

    // Create Argon2id instance
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    // Derive key from PIN using stored salt
    let key = argon2
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|e| format!("Failed to derive key: {}", e))?
        .hash
        .ok_or("No hash value generated")?
        .as_bytes()
        .to_vec();

    // Create AES-GCM cipher
    let key = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    // Create nonce from decoded bytes
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decrypt the seed
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    // Convert plaintext bytes to string
    String::from_utf8(plaintext).map_err(|e| format!("Failed to decode seed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let seed = "math tomorrow must labor noodle cost cattle place intact enforce method layer";
        let pin = "123456";
        let encrypted_data = encrypt_seed(seed, pin).unwrap();
        let decrypted_data = decrypt_seed(&encrypted_data, pin).unwrap();
        assert_eq!(seed, decrypted_data);
    }
}
