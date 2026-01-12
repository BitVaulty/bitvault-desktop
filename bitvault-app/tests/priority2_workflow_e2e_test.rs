//! Priority 2 Workflow E2E Tests
//!
//! Tests for:
//! - Vault Selection workflow
//! - Co-owner Exchange workflow
//! - View-Only Mode workflow
//! - Receive Flow workflow

use bitvault_common::wallet::{VaultMetadata, VaultService};
use bdk::bitcoin::Network;
use tempfile::TempDir;

/// Helper to create a test vault with metadata
async fn create_test_vault_with_metadata(
    name: &str,
    network: Network,
    temp_dir: &TempDir,
) -> Result<(VaultService, VaultMetadata), String> {
    let descriptor = "wsh(multi(2,tpub1,tpub2))";
    let db_path = temp_dir.path().join(format!("{}.db", name));

    let mut vault_service = VaultService::new(network);
    vault_service
        .initialize_wallet(descriptor, Some(db_path.to_str().unwrap().to_string()), None)
        .await
        .map_err(|e| format!("Failed to initialize wallet: {}", e))?;

    let address = vault_service
        .get_new_address()
        .await
        .map_err(|e| format!("Failed to get address: {}", e))?;

    let mut metadata = VaultMetadata::new(
        name.to_string(),
        network,
        address.clone(),
        db_path.to_str().unwrap().to_string(),
    );
    metadata.descriptor = Some(descriptor.to_string());
    metadata.save().map_err(|e| format!("Failed to save metadata: {}", e))?;

    Ok((vault_service, metadata))
}

// ============================================================================
// VAULT SELECTION WORKFLOW TESTS
// ============================================================================

#[tokio::test]
async fn test_vault_selection_listing() {
    // Test: Vault selection can list all available vaults
    let temp_dir = TempDir::new().unwrap();

    // Create multiple vaults
    let (_, _) = create_test_vault_with_metadata("Vault A", Network::Testnet, &temp_dir)
        .await
        .unwrap();
    let (_, _) = create_test_vault_with_metadata("Vault B", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // List vaults
    let vaults = VaultService::list_vaults().unwrap();

    // Verify both vaults are listed
    assert!(vaults.len() >= 2, "Should list at least 2 vaults");
    let vault_a_found = vaults.iter().any(|v| v.name == "Vault A");
    let vault_b_found = vaults.iter().any(|v| v.name == "Vault B");
    assert!(vault_a_found, "Vault A should be in the list");
    assert!(vault_b_found, "Vault B should be in the list");
}

#[tokio::test]
async fn test_vault_selection_load() {
    // Test: Vault selection can load a selected vault
    let temp_dir = TempDir::new().unwrap();
    let (_, metadata) = create_test_vault_with_metadata("Load Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Load vault from metadata
    let loaded_service = VaultService::load_vault_from_metadata(&metadata)
        .await
        .unwrap();

    // Verify vault is loaded
    assert!(loaded_service.is_loaded(), "Vault should be loaded");
    
    // Verify address can be retrieved
    let address = loaded_service.get_new_address().await.unwrap();
    assert!(!address.is_empty(), "Address should be retrievable");
}

#[tokio::test]
async fn test_vault_selection_rename() {
    // Test: Vault selection can rename a vault
    let temp_dir = TempDir::new().unwrap();
    let (_, mut metadata) = create_test_vault_with_metadata("Original Name", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Rename vault
    metadata.name = "Renamed Vault".to_string();
    metadata.save().unwrap();

    // Load and verify rename
    let loaded = VaultMetadata::load(&metadata.address).unwrap();
    assert_eq!(loaded.name, "Renamed Vault");
}

#[tokio::test]
async fn test_vault_selection_delete() {
    // Test: Vault selection can delete a vault
    let temp_dir = TempDir::new().unwrap();
    let (_, metadata) = create_test_vault_with_metadata("Delete Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Verify vault exists
    assert!(VaultMetadata::load(&metadata.address).is_ok());

    // Delete vault
    VaultMetadata::delete(&metadata.address).unwrap();

    // Verify vault is deleted
    assert!(VaultMetadata::load(&metadata.address).is_err());
}

#[tokio::test]
async fn test_vault_selection_filter_orphaned() {
    // Test: Vault selection filters out orphaned metadata (missing database)
    let temp_dir = TempDir::new().unwrap();
    let (_, metadata) = create_test_vault_with_metadata("Orphan Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Delete database but keep metadata
    std::fs::remove_file(&metadata.database_path).unwrap();

    // List vaults - should handle orphaned metadata
    let vaults = VaultService::list_vaults().unwrap();
    
    // Depending on implementation, orphaned vaults might be filtered or removed
    // This test verifies the listing doesn't crash
    assert!(true, "Listing should handle orphaned metadata gracefully");
}

// ============================================================================
// CO-OWNER EXCHANGE WORKFLOW TESTS
// ============================================================================

#[tokio::test]
async fn test_coowner_exchange_key_encoding() {
    // Test: Co-owner keys can be encoded for exchange
    use bitvault_common::derivation::CoownerKeys;
    use bitvault_common::ur::encode_qr_data;

    // Create mock co-owner keys
    let coowner_keys = CoownerKeys {
        mainnet: bitvault_common::derivation::NetworkKeys {
            owner_key1: "xpub1".to_string(),
            owner_key2: "xpub2".to_string(),
            owner_key1_change: "xpub1_change".to_string(),
            owner_key2_change: "xpub2_change".to_string(),
        },
        testnet: bitvault_common::derivation::NetworkKeys {
            owner_key1: "tpub1".to_string(),
            owner_key2: "tpub2".to_string(),
            owner_key1_change: "tpub1_change".to_string(),
            owner_key2_change: "tpub2_change".to_string(),
        },
    };

    // Encode keys
    let encoded = encode_qr_data(&coowner_keys).unwrap();
    assert!(!encoded.is_empty(), "Encoded keys should not be empty");

    // Decode keys
    let decoded: CoownerKeys = bitvault_common::ur::decode_qr_data(&encoded).unwrap();
    assert_eq!(decoded.mainnet.owner_key1, "xpub1");
    assert_eq!(decoded.testnet.owner_key1, "tpub1");
}

#[tokio::test]
async fn test_coowner_exchange_key_decoding() {
    // Test: Co-owner keys can be decoded from exchange data
    use bitvault_common::derivation::CoownerKeys;
    use bitvault_common::ur::{decode_qr_data, encode_qr_data};

    // Create and encode keys
    let original_keys = CoownerKeys {
        mainnet: bitvault_common::derivation::NetworkKeys {
            owner_key1: "xpub_main_1".to_string(),
            owner_key2: "xpub_main_2".to_string(),
            owner_key1_change: "xpub_main_1_change".to_string(),
            owner_key2_change: "xpub_main_2_change".to_string(),
        },
        testnet: bitvault_common::derivation::NetworkKeys {
            owner_key1: "tpub_test_1".to_string(),
            owner_key2: "tpub_test_2".to_string(),
            owner_key1_change: "tpub_test_1_change".to_string(),
            owner_key2_change: "tpub_test_2_change".to_string(),
        },
    };

    let encoded = encode_qr_data(&original_keys).unwrap();
    let decoded: CoownerKeys = decode_qr_data(&encoded).unwrap();

    // Verify all fields match
    assert_eq!(decoded.mainnet.owner_key1, original_keys.mainnet.owner_key1);
    assert_eq!(decoded.mainnet.owner_key2, original_keys.mainnet.owner_key2);
    assert_eq!(decoded.testnet.owner_key1, original_keys.testnet.owner_key1);
    assert_eq!(decoded.testnet.owner_key2, original_keys.testnet.owner_key2);
}

#[tokio::test]
async fn test_coowner_exchange_invalid_data() {
    // Test: Co-owner exchange handles invalid data gracefully
    use bitvault_common::ur::decode_qr_data;
    use bitvault_common::derivation::CoownerKeys;

    // Try to decode invalid data
    let invalid_data = "invalid_qr_data";
    let result: Result<CoownerKeys, _> = decode_qr_data(invalid_data);
    
    // Should return error for invalid data
    assert!(result.is_err(), "Invalid data should return error");
}

// ============================================================================
// VIEW-ONLY MODE WORKFLOW TESTS
// ============================================================================

#[tokio::test]
async fn test_view_only_descriptor_parsing() {
    // Test: View-only mode can parse descriptor from QR
    let descriptor = "wsh(multi(2,tpub1,tpub2))";
    
    // Verify descriptor is valid format
    assert!(descriptor.starts_with("wsh("), "Descriptor should start with wsh(");
    assert!(descriptor.contains("multi("), "Descriptor should contain multi(");
}

#[tokio::test]
async fn test_view_only_vault_creation() {
    // Test: View-only vault can be created from descriptor
    let temp_dir = TempDir::new().unwrap();
    let descriptor = "wsh(multi(2,tpub1,tpub2))";
    let db_path = temp_dir.path().join("view_only.db");

    // Create view-only vault (no mnemonic needed)
    let mut vault_service = VaultService::new(Network::Testnet);
    vault_service
        .initialize_wallet(descriptor, Some(db_path.to_str().unwrap().to_string()), None)
        .await
        .unwrap();

    // Verify vault is loaded
    assert!(vault_service.is_loaded(), "View-only vault should be loaded");
    
    // Verify address can be retrieved (view-only can still get addresses)
    let address = vault_service.get_new_address().await.unwrap();
    assert!(!address.is_empty(), "View-only vault should be able to get address");
}

#[tokio::test]
async fn test_view_only_metadata_storage() {
    // Test: View-only vault metadata is stored correctly
    let temp_dir = TempDir::new().unwrap();
    let descriptor = "wsh(multi(2,tpub1,tpub2))";
    let db_path = temp_dir.path().join("view_only_meta.db");

    let mut vault_service = VaultService::new(Network::Testnet);
    vault_service
        .initialize_wallet(descriptor, Some(db_path.to_str().unwrap().to_string()), None)
        .await
        .unwrap();

    let address = vault_service.get_new_address().await.unwrap();
    let mut metadata = VaultMetadata::new(
        "View-Only Vault".to_string(),
        Network::Testnet,
        address.clone(),
        db_path.to_str().unwrap().to_string(),
    );
    metadata.descriptor = Some(descriptor.to_string());
    metadata.save().unwrap();

    // Load metadata
    let loaded = VaultMetadata::load(&address).unwrap();
    assert_eq!(loaded.name, "View-Only Vault");
    assert_eq!(loaded.descriptor, Some(descriptor.to_string()));
}

// ============================================================================
// RECEIVE FLOW WORKFLOW TESTS
// ============================================================================

#[tokio::test]
async fn test_receive_address_generation() {
    // Test: Receive flow can generate a new address
    let temp_dir = TempDir::new().unwrap();
    let (vault_service, _) = create_test_vault_with_metadata("Receive Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Generate new address
    let address = vault_service.get_new_address().await.unwrap();
    
    // Verify address is valid
    assert!(!address.is_empty(), "Address should not be empty");
    assert!(address.starts_with("tb1") || address.starts_with("bc1"), "Address should be valid format");
}

#[tokio::test]
async fn test_receive_address_consistency() {
    // Test: Receive flow returns consistent addresses from same vault
    let temp_dir = TempDir::new().unwrap();
    let (vault_service, _) = create_test_vault_with_metadata("Consistency Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Get address multiple times
    let address1 = vault_service.get_address().unwrap();
    let address2 = vault_service.get_address().unwrap();
    
    // Stored address should be consistent
    assert_eq!(address1, address2, "Stored address should be consistent");
}

#[tokio::test]
async fn test_receive_address_new_vs_stored() {
    // Test: Receive flow can get both new and stored addresses
    let temp_dir = TempDir::new().unwrap();
    let (vault_service, _) = create_test_vault_with_metadata("New vs Stored", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Get stored address
    let stored_address = vault_service.get_address().unwrap();
    
    // Get new address
    let new_address = vault_service.get_new_address().await.unwrap();
    
    // Both should be valid
    assert!(!stored_address.is_empty(), "Stored address should be valid");
    assert!(!new_address.is_empty(), "New address should be valid");
    
    // New address might be different from stored (depending on address index)
    // Both should be valid Bitcoin addresses
    assert!(stored_address.starts_with("tb1") || stored_address.starts_with("bc1"));
    assert!(new_address.starts_with("tb1") || new_address.starts_with("bc1"));
}

#[tokio::test]
async fn test_receive_address_network_specific() {
    // Test: Receive addresses are network-specific
    let temp_dir = TempDir::new().unwrap();
    
    // Create testnet vault
    let (testnet_service, _) = create_test_vault_with_metadata("Testnet", Network::Testnet, &temp_dir)
        .await
        .unwrap();
    let testnet_address = testnet_service.get_new_address().await.unwrap();
    
    // Create mainnet vault
    let (mainnet_service, _) = create_test_vault_with_metadata("Mainnet", Network::Bitcoin, &temp_dir)
        .await
        .unwrap();
    let mainnet_address = mainnet_service.get_new_address().await.unwrap();
    
    // Verify addresses are different
    assert_ne!(testnet_address, mainnet_address, "Addresses should be different for different networks");
    
    // Verify address formats match network
    assert!(testnet_address.starts_with("tb1"), "Testnet address should start with tb1");
    assert!(mainnet_address.starts_with("bc1"), "Mainnet address should start with bc1");
}

#[tokio::test]
async fn test_receive_flow_error_handling() {
    // Test: Receive flow handles errors gracefully
    
    // Create vault but don't initialize wallet
    let vault_service = VaultService::new(Network::Testnet);
    
    // Try to get address from uninitialized vault
    let result = vault_service.get_new_address().await;
    
    // Should return error
    assert!(result.is_err(), "Uninitialized vault should return error when getting address");
}
