use std::panic;
use std::str::FromStr;
use bitcoin::{Address, Amount, Network, Txid};
use bdk::FeeRate;
use bitvault_common::types::*;
use bitvault_common::bitcoin_utils;
use bitvault_common::config::Config;
use anyhow::Result;
use std::sync::Once;
use bitvault_common::logging::{self, LogConfig, LogLevel};
use std::fs;
use std::path::Path;
use bdk::keys::bip39::Mnemonic;
use bdk::keys::ExtendedKey;
use bitvault_common::types::WalletError;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};
use generic_array::GenericArray;
use hmac::Hmac;
use pbkdf2;
use sha2::Sha256;
use bitvault_common::key_management::{
    AES_KEY_SIZE, 
    STORAGE_FORMAT_VERSION,
    TEST_SALT_BYTES,
    TEST_NONCE_BYTES,
    TEST_PBKDF2_ITERATIONS
};

// Define constants needed for test utilities
const SALT_SIZE: usize = 16;   // 128 bits
const NONCE_SIZE: usize = 12;  // 96 bits for AES-GCM

// Global initialization for all tests
static GLOBAL_TEST_INIT: Once = Once::new();

/// Initialize test environment
/// 
/// This function ensures that global resources like logging are only initialized once
/// across all tests. Each test module should call this function in its setup.
pub fn init_test_environment() {
    GLOBAL_TEST_INIT.call_once(|| {
        // Configure minimal logging for tests
        let config = LogConfig {
            level: LogLevel::Error, // Use Error level to minimize output
            log_file: None,         // No file logging in tests
            include_timestamps: false,
            include_source_location: false,
            max_file_size: 1024 * 1024,
            console_logging: false, // Disable console logging for tests
            json_format: false,
        };

        // Initialize logging with test configuration
        // Ignore any errors - tests should work even if logging fails
        let _ = logging::init(&config);
    });
}

// Setup function to run at the beginning of each test to capture panics and log them
pub fn test_with_logging<T, F: FnOnce() -> T + panic::UnwindSafe>(
    name: &str,
    test_fn: F,
) -> Result<T, String> {
    eprintln!("===== STARTING TEST: {} =====", name);

    let result = panic::catch_unwind(|| test_fn());

    match result {
        Ok(value) => {
            eprintln!("===== TEST PASSED: {} =====", name);
            Ok(value)
        }
        Err(e) => {
            let panic_msg = if let Some(msg) = e.downcast_ref::<String>() {
                format!("{}", msg)
            } else if let Some(msg) = e.downcast_ref::<&str>() {
                format!("{}", msg)
            } else {
                "Unknown panic".to_string()
            };

            eprintln!("===== TEST FAILED: {} - PANIC: {} =====", name, panic_msg);
            Err(panic_msg)
        }
    }
}

/// Sample addresses for different networks and formats
pub struct TestAddresses;

impl TestAddresses {
    /// Returns a valid P2PKH address for the given network
    pub fn p2pkh(network: Network) -> Address {
        let addr_str = match network {
            Network::Bitcoin => "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
            Network::Testnet => "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn",
            Network::Regtest => "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn",
            Network::Signet => "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            _ => panic!("Unsupported network for test address"),
        };
        
        bitcoin_utils::parse_address(addr_str, network).unwrap()
    }

    /// Returns a valid P2WPKH (bech32) address for the given network
    pub fn p2wpkh(network: Network) -> Address {
        let addr_str = match network {
            Network::Bitcoin => "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
            Network::Testnet => "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            Network::Regtest => "bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080",
            Network::Signet => "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            _ => panic!("Unsupported network for test address"),
        };
        
        bitcoin_utils::parse_address(addr_str, network).unwrap()
    }

    /// Returns a valid P2WSH (bech32) address for the given network
    pub fn p2wsh(network: Network) -> Address {
        let addr_str = match network {
            Network::Bitcoin => "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3",
            Network::Testnet => "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7",
            Network::Regtest => "bcrt1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qzf4jry",
            Network::Signet => "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7",
            _ => panic!("Unsupported network for test address"),
        };
        
        bitcoin_utils::parse_address(addr_str, network).unwrap()
    }

    /// Returns an invalid address string that has correct format but wrong checksum
    pub fn invalid_checksum() -> String {
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t5".to_string() // Changed last char
    }

    /// Returns an address string with completely invalid format
    pub fn invalid_format() -> String {
        "invalid-bitcoin-address-format".to_string()
    }

    /// Returns an address from the wrong network (testnet address when Bitcoin network is expected)
    pub fn wrong_network(expected: Network) -> String {
        match expected {
            Network::Bitcoin => "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            _ => "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        }.to_string()
    }
}

/// Test transaction utilities
pub struct TestTransactions;

impl TestTransactions {
    /// Returns a sample transaction ID string
    pub fn sample_txid_str() -> &'static str {
        "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16"
    }

    /// Returns a sample Txid
    pub fn sample_txid() -> Txid {
        Txid::from_str(Self::sample_txid_str()).unwrap()
    }
}

/// Utility for creating wallet settings for testing
pub fn create_test_wallet_settings(network: Network) -> WalletSettings {
    WalletSettings {
        network,
        use_tor: false,
        default_fee_level: FeePriority::Medium,
        custom_fee_rate: Some(FeeRate::from_sat_per_vb(5.0)),
        display_as_bitcoin: true,
        preferred_fiat: "USD".to_string(),
        show_fiat_amounts: true,
        custom_settings: std::collections::HashMap::new(),
    }
}

/// Utility to create a test AddressInfo
pub fn create_test_address_info(network: Network, is_owned: bool) -> AddressInfo {
    let address = TestAddresses::p2wpkh(network);
    let info = AddressInfo::new(address, is_owned);
    
    if is_owned {
        return info.with_label("Test address".to_string());
    }
    
    info
}

/// Utility to create sample WalletTransaction objects for testing
pub fn create_test_wallet_transaction(is_sent: bool) -> WalletTransaction {
    let txid = TestTransactions::sample_txid();
    let timestamp = 1617184224; // Some fixed timestamp
    
    let amount = if is_sent {
        Amount::from_sat(150000) // 0.0015 BTC
    } else {
        Amount::from_sat(250000) // 0.0025 BTC
    };
    
    // Create the BDK transaction details
    let details = bdk::TransactionDetails {
        transaction: None, // We don't need the full transaction for most tests
        txid,
        received: if is_sent { 0 } else { amount.to_sat() },
        sent: if is_sent { amount.to_sat() } else { 0 },
        fee: if is_sent { Some(10000) } else { None },
        confirmation_time: None, // We'll avoid using ConfirmationTime directly
    };
    
    // Create a sample address info for the transaction
    let address_info = create_test_address_info(
        Network::Bitcoin,
        !is_sent, // If receiving, it's our address
    );
    
    WalletTransaction {
        details,
        memo: None,
        addresses: vec![address_info],
        timestamp: timestamp as u64, // We still use the timestamp in our struct
    }
}

pub fn load_default_config() -> Result<Config> {
    Config::load("path/to/default/config.toml")
}

pub fn load_custom_config(network: &str, fiat: &str) -> Result<Config> {
    let mut config = load_default_config()?;
    config.wallet.network = network.to_string();
    config.wallet.fiat_currency = fiat.to_string();
    Ok(config)
}

pub fn direct_test_store_key(
    mnemonic: &Mnemonic, 
    password: &str, 
    file_path: &str
) -> Result<(), WalletError> {
    println!("TEST MODE: Using direct test implementation for storage");
    
    // Get the mnemonic phrase as a string
    let mnemonic_phrase = mnemonic.to_string();
    println!("TEST MODE: Mnemonic phrase length: {}", mnemonic_phrase.len());
    
    // Use the constants from key_management for consistent testing
    let salt = TEST_SALT_BYTES.to_vec();
    let iterations = TEST_PBKDF2_ITERATIONS; 
    let nonce_bytes = TEST_NONCE_BYTES.to_vec();
    println!("TEST MODE: Using fixed salt and nonce for testing");
    println!("TEST MODE: Salt: {:02X?}", salt);
    println!("TEST MODE: Nonce: {:02X?}", nonce_bytes);
    
    // Derive key bytes directly
    let mut key_bytes = vec![0u8; AES_KEY_SIZE];
    match pbkdf2::pbkdf2::<Hmac<Sha256>>(
        password.as_bytes(),
        &salt,
        iterations,
        &mut key_bytes
    ) {
        Ok(_) => println!("TEST MODE: PBKDF2 derivation successful, key: {:02X?}", key_bytes),
        Err(e) => {
            println!("TEST MODE: PBKDF2 derivation failed: {:?}", e);
            return Err(WalletError::Crypto(format!("Test PBKDF2 derivation failed: {:?}", e)));
        }
    }
    
    // Simple encryption with fixed nonce
    let encryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(encryption_key);
    let nonce = *GenericArray::from_slice(&nonce_bytes);
    println!("TEST MODE: Created cipher and nonce");
    
    // Encrypt the mnemonic
    let ciphertext = match cipher.encrypt(
        &nonce, 
        Payload { 
            msg: mnemonic_phrase.as_bytes(), 
            aad: &[] 
        }
    ) {
        Ok(ct) => {
            println!("TEST MODE: Successfully encrypted mnemonic, ciphertext length: {}", ct.len());
            println!("TEST MODE: Ciphertext (first 32 bytes): {:02X?}", &ct[..std::cmp::min(32, ct.len())]);
            ct
        },
        Err(e) => {
            println!("TEST MODE: Encryption failed: {:?}", e);
            return Err(WalletError::Crypto(format!("Test encryption failed: {:?}", e)));
        }
    };
    
    // Create a legacy format file (version 1)
    // Format: [version(1)][salt(16)][nonce(12)][ciphertext]
    let mut file_data = Vec::with_capacity(
        1 + salt.len() + nonce_bytes.len() + ciphertext.len()
    );
    
    // Add version byte, salt, nonce, and ciphertext
    file_data.push(STORAGE_FORMAT_VERSION); // Use legacy format version 1
    file_data.extend_from_slice(&salt);
    file_data.extend_from_slice(&nonce_bytes);
    file_data.extend_from_slice(&ciphertext);
    println!("TEST MODE: Prepared file data of length {}", file_data.len());
    println!("TEST MODE: File format: [version(1)][salt(16)][nonce(12)][ciphertext]");
    
    // Make sure parent directory exists
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.exists() {
            match fs::create_dir_all(parent) {
                Ok(_) => println!("TEST MODE: Created parent directory"),
                Err(e) => {
                    println!("TEST MODE: Failed to create parent directory: {:?}", e);
                    return Err(WalletError::IoError(format!("Failed to create parent directory: {:?}", e)));
                }
            }
        }
    }
    
    // Write the data to the file
    match fs::write(file_path, &file_data) {
        Ok(_) => {
            println!("TEST MODE: Successfully wrote file to {}", file_path);
            println!("TEST MODE: File data first 32 bytes: {:02X?}", &file_data[..std::cmp::min(32, file_data.len())]);
        },
        Err(e) => {
            println!("TEST MODE: File write failed: {:?}", e);
            return Err(WalletError::IoError(format!("Failed to write test key file: {:?}", e)));
        }
    }
    
    // Verify the file was created
    match Path::new(file_path).exists() {
        true => println!("TEST MODE: Verified file exists at {}", file_path),
        false => {
            println!("TEST MODE: File verification failed - file doesn't exist");
            return Err(WalletError::IoError(format!("File verification failed - file doesn't exist")));
        }
    }
    
    println!("TEST MODE: Successfully stored test key to {}", file_path);
    Ok(())
}

/// Direct test decryption function to match the direct_test_store_key function
pub fn direct_test_decrypt_key(
    password: &str, 
    file_path: &str
) -> Result<(ExtendedKey, Mnemonic), WalletError> {
    println!("TEST UTILS: Using direct test implementation for decryption");
    
    // Read the file
    let file_data = fs::read(file_path)
        .map_err(|e| {
            println!("TEST UTILS: Failed to read encrypted key: {:?}", e);
            WalletError::IoError(format!("Failed to read encrypted key: {:?}", e))
        })?;
    
    println!("TEST UTILS: File read successfully, size: {} bytes", file_data.len());
    
    if file_data.len() < 1 + SALT_SIZE + NONCE_SIZE + 1 {
        println!("TEST UTILS: File too small: {} bytes", file_data.len());
        return Err(WalletError::Crypto(format!("File too small: {} bytes", file_data.len())));
    }
    
    // Check if format matches what we expect (version 1)
    if file_data[0] != STORAGE_FORMAT_VERSION {
        println!("TEST UTILS: Unexpected format version: {}", file_data[0]);
        return Err(WalletError::Crypto(format!("Unexpected format version: {}", file_data[0])));
    }
    
    // Extract salt, nonce, and ciphertext
    let salt = &file_data[1..1 + SALT_SIZE];
    let nonce_start = 1 + SALT_SIZE;
    let nonce = &file_data[nonce_start..nonce_start + NONCE_SIZE];
    let ciphertext = &file_data[nonce_start + NONCE_SIZE..];
    
    println!("TEST UTILS: Extracted salt: {:02X?}", salt);
    println!("TEST UTILS: Extracted nonce: {:02X?}", nonce);
    println!("TEST UTILS: Ciphertext length: {}", ciphertext.len());
    
    // Derive key bytes directly with the consistent test iterations
    let mut key_bytes = vec![0u8; AES_KEY_SIZE];
    match pbkdf2::pbkdf2::<Hmac<Sha256>>(
        password.as_bytes(),
        salt,
        TEST_PBKDF2_ITERATIONS, // Use the constant instead of hardcoded 10
        &mut key_bytes
    ) {
        Ok(_) => println!("TEST UTILS: PBKDF2 derivation successful, key: {:02X?}", key_bytes),
        Err(e) => {
            println!("TEST UTILS: PBKDF2 derivation failed: {:?}", e);
            return Err(WalletError::Crypto(format!("Test PBKDF2 derivation failed: {:?}", e)));
        }
    }
    
    // Create cipher and nonce
    let decryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(decryption_key);
    let nonce_array = GenericArray::from_slice(nonce);
    println!("TEST UTILS: Created cipher and nonce for decryption");
    
    // Decrypt the ciphertext
    let plaintext = match cipher.decrypt(
        nonce_array, 
        Payload { 
            msg: ciphertext, 
            aad: &[] 
        }
    ) {
        Ok(pt) => {
            println!("TEST UTILS: Decryption successful, plaintext length: {}", pt.len());
            pt
        },
        Err(e) => {
            println!("TEST UTILS: Decryption failed: {:?}", e);
            println!("TEST UTILS: Key: {:02X?}", key_bytes);
            println!("TEST UTILS: Salt: {:02X?}", salt);
            println!("TEST UTILS: Nonce: {:02X?}", nonce);
            println!("TEST UTILS: Ciphertext (first 32 bytes): {:02X?}", &ciphertext[..std::cmp::min(32, ciphertext.len())]);
            return Err(WalletError::Crypto(format!("Test decryption failed: {:?}", e)));
        }
    };
    
    // Convert to UTF-8 and parse as mnemonic
    let mnemonic_phrase = match std::str::from_utf8(&plaintext) {
        Ok(phrase) => {
            println!("TEST UTILS: UTF-8 decoding successful: '{}'", phrase);
            phrase
        },
        Err(e) => {
            println!("TEST UTILS: UTF-8 decoding failed: {:?}", e);
            return Err(WalletError::Crypto(format!("Failed to decode mnemonic UTF-8: {:?}", e)));
        }
    };
    
    // Parse the mnemonic
    let mnemonic = match Mnemonic::parse(mnemonic_phrase) {
        Ok(m) => {
            println!("TEST UTILS: Mnemonic parsing successful");
            m
        },
        Err(e) => {
            println!("TEST UTILS: Mnemonic parsing failed: {:?}", e);
            return Err(WalletError::Crypto(format!("Failed to parse mnemonic: {:?}", e)));
        }
    };
    
    // Generate seed and extended key
    let seed = mnemonic.to_seed(password);
    println!("TEST UTILS: Seed generated successfully");
    
    // Create an extended private key
    let xpriv = match bitcoin::bip32::ExtendedPrivKey::new_master(bitcoin::Network::Bitcoin, &seed) {
        Ok(key) => {
            println!("TEST UTILS: Master key created successfully");
            key
        },
        Err(e) => {
            println!("TEST UTILS: Master key creation failed: {:?}", e);
            return Err(WalletError::Crypto(format!("Extended key derivation failed: {:?}", e)));
        }
    };
    
    // Convert to BDK's ExtendedKey type
    let key = ExtendedKey::from(xpriv);
    println!("TEST UTILS: Key successfully created");
    
    println!("TEST UTILS: TEST DECRYPT SUCCESS");
    Ok((key, mnemonic))
}
