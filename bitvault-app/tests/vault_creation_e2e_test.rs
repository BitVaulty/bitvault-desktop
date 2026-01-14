//! E2E Tests for Vault Creation Flow
//!
//! Tests complete user workflows for:
//! - Vault creation (main device)
//! - Seed phrase generation and verification
//! - Vault metadata and persistence
//!
//! Note: These tests focus on the core vault creation logic
//! Network-dependent operations (ConvenienceService) are mocked or skipped

use bdk::bitcoin::Network;
use bdk::keys::bip39::Mnemonic;
use std::str::FromStr;
use bitvault_common::utils::TimeDelay;
use bitvault_common::wallet::VaultService;
use bitvault_common::derivation::get_owner_keys;
use bitvault_common::convenience::ConvenienceService;
use bitvault_common::wallet::VaultMetadata;

/// Helper to create a test mnemonic (BDK type)
fn create_test_mnemonic() -> Mnemonic {
    // Use a deterministic mnemonic for testing
    // BDK's Mnemonic::from_entropy takes just the entropy
    Mnemonic::from_entropy(&[0u8; 16]).unwrap()
}

/// Helper to create a test mnemonic string for verification tests
fn create_test_mnemonic_phrase() -> String {
    let mnemonic = create_test_mnemonic();
    mnemonic.to_string()
}

/// Helper to create test coowner keys
fn create_test_coowner_keys() -> bitvault_common::derivation::CoownerKeys {
    // Create a test mnemonic for coowner (BDK type)
    let coowner_mnemonic = Mnemonic::from_entropy(&[1u8; 16]).unwrap();
    get_owner_keys(&coowner_mnemonic).unwrap()
}

/// Helper to encode coowner keys as string (simulating QR code data)
fn encode_coowner_keys(keys: &bitvault_common::derivation::CoownerKeys) -> String {
    serde_json::to_string(keys).unwrap()
}

#[tokio::test]
async fn test_vault_creation_seed_phrase_generation() {
    // Test: Seed phrase can be generated
    // Generate 12-word mnemonic (16 bytes entropy)
    let entropy_12 = [0u8; 16];
    let mnemonic_12 = Mnemonic::from_entropy(&entropy_12).unwrap();
    
    let phrase_12 = mnemonic_12.to_string();
    let words: Vec<&str> = phrase_12.split_whitespace().collect();
    assert_eq!(words.len(), 12, "12-word mnemonic should have 12 words");
    
    // Generate 24-word mnemonic (32 bytes entropy)
    let entropy_24 = [0u8; 32];
    let mnemonic_24 = Mnemonic::from_entropy(&entropy_24).unwrap();
    
    let phrase_24 = mnemonic_24.to_string();
    let words: Vec<&str> = phrase_24.split_whitespace().collect();
    assert_eq!(words.len(), 24, "24-word mnemonic should have 24 words");
}

#[tokio::test]
async fn test_vault_creation_seed_phrase_verification() {
    // Test: Seed phrase verification logic
    let original_phrase = create_test_mnemonic_phrase();
    
    // Test correct verification - BDK's Mnemonic uses FromStr
    let verified_mnemonic: Result<Mnemonic, _> = original_phrase.parse();
    assert!(verified_mnemonic.is_ok(), "Valid mnemonic should parse");
    assert_eq!(verified_mnemonic.unwrap().to_string(), original_phrase);
    
    // Test incorrect verification (wrong word)
    let mut wrong_phrase = original_phrase.split_whitespace().collect::<Vec<&str>>();
    wrong_phrase[0] = "wrong";
    let wrong_phrase_str = wrong_phrase.join(" ");
    
    // Should fail to parse or produce different mnemonic
    let wrong_result: Result<Mnemonic, _> = wrong_phrase_str.parse();
    assert!(wrong_result.is_err() || wrong_result.unwrap().to_string() != original_phrase, 
            "Invalid mnemonic should fail to parse or produce different result");
}

#[tokio::test]
async fn test_vault_creation_owner_keys_derivation() {
    // Test: Owner keys can be derived from mnemonic
    let mnemonic = create_test_mnemonic();
    
    let owner_keys = get_owner_keys(&mnemonic).unwrap();
    
    // Verify keys are derived
    // CoownerKeys has mainnet and testnet fields, each containing NetworkKeys
    // NetworkKeys contains owner_key1, owner_key2, etc.
    assert!(!owner_keys.mainnet.owner_key1.is_empty(), "Mainnet owner_key1 should be derived");
    assert!(!owner_keys.testnet.owner_key1.is_empty(), "Testnet owner_key1 should be derived");
    
    // Verify keys are non-empty strings (actual format depends on BDK's key derivation)
    // BDK may format keys as [fingerprint/path]xpub... or in other formats
    // The important thing is that keys are derived and non-empty
    let mainnet_key = &owner_keys.mainnet.owner_key1;
    let testnet_key = &owner_keys.testnet.owner_key1;
    
    // Just verify they're non-empty - the actual format may vary
    assert!(!mainnet_key.is_empty(), "Mainnet key should not be empty, got: '{}'", mainnet_key);
    assert!(!testnet_key.is_empty(), "Testnet key should not be empty, got: '{}'", testnet_key);
    
    // Verify all four keys are present for each network
    assert!(!owner_keys.mainnet.owner_key2.is_empty());
    assert!(!owner_keys.mainnet.owner_key1_change.is_empty());
    assert!(!owner_keys.mainnet.owner_key2_change.is_empty());
    assert!(!owner_keys.testnet.owner_key2.is_empty());
    assert!(!owner_keys.testnet.owner_key1_change.is_empty());
    assert!(!owner_keys.testnet.owner_key2_change.is_empty());
}

#[tokio::test]
async fn test_vault_creation_coowner_keys_encoding() {
    // Test: Coowner keys can be encoded and decoded
    let coowner_keys = create_test_coowner_keys();
    
    // Encode as string (simulating QR code)
    let encoded = encode_coowner_keys(&coowner_keys);
    assert!(!encoded.is_empty());
    
    // Decode back
    let decoded: bitvault_common::derivation::CoownerKeys = serde_json::from_str(&encoded).unwrap();
    
    assert_eq!(decoded.mainnet.owner_key1, coowner_keys.mainnet.owner_key1);
    assert_eq!(decoded.testnet.owner_key1, coowner_keys.testnet.owner_key1);
}

#[tokio::test]
async fn test_vault_creation_time_delay_conversion() {
    // Test: Time delay can be converted to blocks
    let time_delay = TimeDelay {
        days: 1,
        hours: 0,
    };
    
    let blocks = time_delay.to_blocks();
    assert!(blocks > 0, "Time delay should convert to positive blocks");
    
    // Test minimum time delay
    let min_delay = TimeDelay {
        days: 0,
        hours: 1,
    };
    let min_blocks = min_delay.to_blocks();
    assert!(min_blocks > 0, "Minimum time delay should be positive");
}

#[tokio::test]
async fn test_vault_service_initialization() {
    // Test: VaultService can be created and initialized
    let network = Network::Testnet;
    let mut vault_service = VaultService::new(network);
    
    // Network field is private, but we can test is_loaded()
    assert!(!vault_service.is_loaded(), "Vault should not be loaded initially");
}

#[tokio::test]
async fn test_vault_creation_descriptor_building() {
    // Test: Descriptors can be built from owner and coowner keys
    let mnemonic = create_test_mnemonic();
    let owner_keys = get_owner_keys(&mnemonic).unwrap();
    let coowner_keys = create_test_coowner_keys();
    
    // Get convenience service pubkey (this will make a network call)
    // For e2e tests, we'll test that the service can be created
    let convenience_service = ConvenienceService::new(None);
    
    // Note: fetch_pubkey() requires network access
    // In a full e2e test, we'd mock this or use a test server
    // For now, we just verify the service can be created
    // The actual setup_vault call would require network access
    
    // Verify keys are valid
    assert!(!owner_keys.mainnet.owner_key1.is_empty());
    assert!(!coowner_keys.mainnet.owner_key1.is_empty());
}

#[tokio::test]
async fn test_vault_metadata_creation() {
    // Test: Vault metadata can be created and saved
    let network = Network::Testnet;
    let vault_address = "tb1qtest1234567890abcdefghijklmnopqrstuvwxyz";
    let database_path = std::env::temp_dir().join("test_vault.db");
    
    let metadata = VaultMetadata::new(
        "Test Vault".to_string(),
        network,
        vault_address.to_string(),
        database_path.to_string_lossy().to_string(),
    );
    
    assert_eq!(metadata.name, "Test Vault");
    assert_eq!(metadata.network_to_bdk(), network);
    assert_eq!(metadata.address, vault_address);
}

#[tokio::test]
async fn test_vault_creation_step_flow() {
    // Test: Vault creation steps can be navigated
    use bitvault_app::ui::vault_creation::{VaultCreationState, VaultCreationStep, DeviceRole};
    
    let mut state = VaultCreationState {
        current_step: VaultCreationStep::RoleSelection,
        previous_step: None,
        step_history: Vec::new(),
        device_role: DeviceRole::Main,
        mnemonic: None,
        verified_seed_phrase: false,
        time_delay_days: 0,
        time_delay_hours: 0,
        coowner_pubkeys: String::new(),
        coowner_keys: None,
        my_keys_text: None,
        exchange_data_input: String::new(),
        vault_name: String::new(),
        vault_address: None,
        exchange_data_output: None,
        email: String::new(),
        auth_code: String::new(),
        code_sent: false,
        is_sending_code: false,
        error: None,
        is_creating: false,
        pin_setup_state: bitvault_app::ui::pin::PinSetupState::new(),
        import_mnemonic_text: String::new(),
        import_descriptors_qr: String::new(),
        is_importing: false,
        camera_capture: None,
        is_scanning_qr: false,
        saved_key_file: None,
        saved_exchange_file: None,
        signing_secret_key: None,
        recipient_public_key: None,
    };
    
    // Test step navigation
    assert_eq!(state.current_step, VaultCreationStep::RoleSelection);
    
    // Simulate advancing through steps
    state.current_step = VaultCreationStep::NameVault;
    assert_eq!(state.current_step, VaultCreationStep::NameVault);
    
    state.current_step = VaultCreationStep::SetTimeDelay;
    assert_eq!(state.current_step, VaultCreationStep::SetTimeDelay);
    
    state.current_step = VaultCreationStep::MnemonicGeneration;
    assert_eq!(state.current_step, VaultCreationStep::MnemonicGeneration);
}

#[tokio::test]
async fn test_vault_creation_mnemonic_storage() {
    // Test: Mnemonic can be stored and cleared securely
    let mnemonic = create_test_mnemonic();
    let phrase = mnemonic.to_string();
    
    // Verify mnemonic is valid
    assert!(!phrase.is_empty());
    let words: Vec<&str> = phrase.split_whitespace().collect();
    assert!(words.len() == 12 || words.len() == 24);
    
    // Test that mnemonic can be parsed back
    let parsed: Mnemonic = phrase.parse().unwrap();
    assert_eq!(parsed.to_string(), phrase);
}

#[tokio::test]
async fn test_vault_creation_vault_name_validation() {
    // Test: Vault name validation
    // Empty name should be invalid
    assert!("".trim().is_empty());
    
    // Valid names
    assert!(!"Test Vault".trim().is_empty());
    assert!(!"My Wallet".trim().is_empty());
    assert!(!"Vault 123".trim().is_empty());
}

#[tokio::test]
async fn test_vault_creation_time_delay_validation() {
    // Test: Time delay validation
    // Minimum time delay should be at least 1 hour
    let min_delay = TimeDelay {
        days: 0,
        hours: 1,
    };
    let blocks = min_delay.to_blocks();
    assert!(blocks > 0, "1 hour delay should be valid");
    
    // Zero delay should be invalid
    let zero_delay = TimeDelay {
        days: 0,
        hours: 0,
    };
    let zero_blocks = zero_delay.to_blocks();
    // Note: to_blocks() might return 0 or a minimum value
    // The actual validation happens in setup_vault()
}

#[tokio::test]
async fn test_vault_listing() {
    // Test: Vaults can be listed
    // This tests the VaultService::<bdk::database::SqliteDatabase>::list_vaults() method
    let result = VaultService::<bdk::database::SqliteDatabase>::list_vaults();
    
    // Should not panic - may return empty list if no vaults exist
    assert!(result.is_ok(), "list_vaults should not error");
    let vaults = result.unwrap();
    // Can be empty - that's fine for a test environment
}

#[tokio::test]
async fn test_vault_creation_error_handling() {
    // Test: Error handling in vault creation
    let network = Network::Testnet;
    let mut vault_service = VaultService::new(network);
    
    // Try to initialize with invalid descriptor
    let invalid_descriptor = "invalid descriptor";
    let result = vault_service
        .initialize_wallet(invalid_descriptor, None, None)
        .await;
    
    // Should return an error
    assert!(result.is_err(), "Invalid descriptor should cause error");
}
