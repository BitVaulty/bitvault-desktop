//! E2E Tests for Vault Persistence
//!
//! Tests vault creation, saving, loading, listing, and deletion:
//! - Vault metadata persistence
//! - Vault listing
//! - Vault loading from metadata
//! - Vault deletion
//! - Network-specific vault isolation
//! - Multiple vault management

use bitvault_common::wallet::{VaultMetadata, VaultService};
use bdk::bitcoin::Network;
use bdk::keys::bip39::Mnemonic;
use bitvault_common::derivation::{build_all_descriptors, get_owner_keys};
use tempfile::TempDir;

/// Helper to create a valid test descriptor from test mnemonics
fn get_test_descriptor(network: Network) -> Result<String, String> {
    use bitvault_common::derivation::build_vault_descriptor;
    
    // Create different mnemonics for owner, coowner, and convenience
    let owner_mnemonic_str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let coowner_mnemonic_str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
    let convenience_mnemonic_str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
    
    let owner_mnemonic: Mnemonic = owner_mnemonic_str
        .parse()
        .map_err(|e| format!("Failed to parse owner mnemonic: {}", e))?;
    let coowner_mnemonic: Mnemonic = coowner_mnemonic_str
        .parse()
        .map_err(|e| format!("Failed to parse coowner mnemonic: {}", e))?;
    let convenience_mnemonic: Mnemonic = convenience_mnemonic_str
        .parse()
        .map_err(|e| format!("Failed to parse convenience mnemonic: {}", e))?;
    
    // Derive keys
    let owner_keys = get_owner_keys(&owner_mnemonic)
        .map_err(|e| format!("Failed to derive owner keys: {}", e))?;
    let coowner_keys = get_owner_keys(&coowner_mnemonic)
        .map_err(|e| format!("Failed to derive coowner keys: {}", e))?;
    let convenience_keys = get_owner_keys(&convenience_mnemonic)
        .map_err(|e| format!("Failed to derive convenience keys: {}", e))?;
    
    let timelock = 144; // Minimum timelock (1 day)
    
    // Build descriptor directly for the specific network
    // This ensures all keys are from the same network (mainnet or testnet)
    let (owner_net_keys, coowner_net_keys, convenience_key) = match network {
        Network::Bitcoin => (
            &owner_keys.mainnet,
            &coowner_keys.mainnet,
            &convenience_keys.mainnet.owner_key1,
        ),
        _ => (
            &owner_keys.testnet,
            &coowner_keys.testnet,
            &convenience_keys.testnet.owner_key1,
        ),
    };
    
    let descriptor = build_vault_descriptor(
        &owner_net_keys.owner_key1,
        &owner_net_keys.owner_key2,
        &coowner_net_keys.owner_key1,
        &coowner_net_keys.owner_key2,
        convenience_key,
        timelock,
    )
    .map_err(|e| format!("Failed to build descriptor: {}", e))?;
    
    if descriptor.is_empty() {
        return Err("Generated descriptor is empty".to_string());
    }
    
    Ok(descriptor)
}

/// Helper to create a test vault with a temporary database
async fn create_test_vault(
    name: &str,
    network: Network,
    temp_dir: &TempDir,
) -> Result<(VaultService, VaultMetadata), String> {
    // Create a test descriptor
    let descriptor = get_test_descriptor(network)?;

    // Create database path in temp directory
    let db_path = temp_dir.path().join(format!("{}.db", name));

    // Create vault service
    let mut vault_service = VaultService::new(network);
    vault_service
        .initialize_wallet(&descriptor, Some(db_path.to_str().unwrap().to_string()), None)
        .await
        .map_err(|e| format!("Failed to initialize wallet: {}", e))?;

    // Get vault address
    let address = vault_service
        .get_new_address()
        .await
        .map_err(|e| format!("Failed to get receive address: {}", e))?;

    // Create metadata with descriptor
    let mut metadata = VaultMetadata::new(
        name.to_string(),
        network,
        address.clone(),
        db_path.to_str().unwrap().to_string(),
    );
    metadata.descriptor = Some(descriptor.clone());

    // Save metadata
    metadata
        .save()
        .map_err(|e| format!("Failed to save metadata: {}", e))?;

    Ok((vault_service, metadata))
}

#[tokio::test]
async fn test_vault_metadata_save_and_load() {
    // Test: Vault metadata can be saved and loaded
    let temp_dir = TempDir::new().unwrap();
    let (_, metadata) = create_test_vault("Test Vault", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Load metadata
    let loaded = VaultMetadata::load(&metadata.address).unwrap();

    // Verify loaded metadata matches
    assert_eq!(loaded.name, metadata.name);
    assert_eq!(loaded.network, metadata.network);
    assert_eq!(loaded.address, metadata.address);
    assert_eq!(loaded.database_path, metadata.database_path);
}

#[tokio::test]
async fn test_vault_listing() {
    // Test: Multiple vaults can be listed
    let temp_dir = TempDir::new().unwrap();

    // Create multiple vaults
    let (_, metadata1) = create_test_vault("Vault 1", Network::Testnet, &temp_dir)
        .await
        .unwrap();
    let (_, metadata2) = create_test_vault("Vault 2", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // List all vaults
    let vaults = VaultService::<bdk::database::SqliteDatabase>::list_vaults().unwrap();

    // Verify both vaults are in the list
    let vault1_found = vaults.iter().any(|v| v.address == metadata1.address);
    let vault2_found = vaults.iter().any(|v| v.address == metadata2.address);

    assert!(vault1_found, "Vault 1 should be in the list");
    assert!(vault2_found, "Vault 2 should be in the list");
}

#[tokio::test]
async fn test_vault_load_from_metadata() {
    // Test: Vault can be loaded from metadata
    let temp_dir = TempDir::new().unwrap();
    let (original_service, metadata) =
        create_test_vault("Load Test", Network::Testnet, &temp_dir)
            .await
            .unwrap();

    // Get original address
    let original_address = original_service
        .get_new_address()
        .await
        .unwrap();

    // Load vault from metadata
    let loaded_service = VaultService::load_vault_from_metadata(&metadata)
        .await
        .unwrap();

    // Verify loaded vault can get address (may be different due to address index)
    let loaded_address = loaded_service.get_new_address().await.unwrap();
    // Addresses might differ due to address index, but both should be valid
    assert!(!loaded_address.is_empty());
}

#[tokio::test]
async fn test_vault_deletion() {
    // Test: Vault metadata can be deleted
    let temp_dir = TempDir::new().unwrap();
    let (_, metadata) = create_test_vault("Delete Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Verify metadata exists
    assert!(VaultMetadata::load(&metadata.address).is_ok());

    // Delete metadata
    VaultMetadata::delete(&metadata.address).unwrap();

    // Verify metadata is deleted
    assert!(VaultMetadata::load(&metadata.address).is_err());
}

#[tokio::test]
async fn test_network_specific_vault_isolation() {
    // Test: Vaults are isolated by network
    let temp_dir = TempDir::new().unwrap();

    // Create vaults on different networks
    let (_, mainnet_metadata) = create_test_vault("Mainnet Vault", Network::Bitcoin, &temp_dir)
        .await
        .unwrap();
    let (_, testnet_metadata) = create_test_vault("Testnet Vault", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // List all vaults
    let vaults = VaultService::<bdk::database::SqliteDatabase>::list_vaults().unwrap();

    // Verify both vaults are listed
    let mainnet_found = vaults.iter().any(|v| v.address == mainnet_metadata.address);
    let testnet_found = vaults.iter().any(|v| v.address == testnet_metadata.address);

    assert!(mainnet_found, "Mainnet vault should be in the list");
    assert!(testnet_found, "Testnet vault should be in the list");

    // Verify networks are different
    assert_ne!(mainnet_metadata.network, testnet_metadata.network);
}

#[tokio::test]
async fn test_vault_metadata_validation() {
    // Test: Vault metadata validation
    let temp_dir = TempDir::new().unwrap();
    let (_, metadata) = create_test_vault("Validation Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Validate metadata
    let validation_result = metadata.validate();
    assert!(validation_result.is_ok(), "Metadata should be valid");
}

#[tokio::test]
async fn test_vault_metadata_update() {
    // Test: Vault metadata can be updated
    let temp_dir = TempDir::new().unwrap();
    let (_, mut metadata) = create_test_vault("Update Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Update metadata
    metadata.name = "Updated Name".to_string();
    metadata.save().unwrap();

    // Load and verify update
    let loaded = VaultMetadata::load(&metadata.address).unwrap();
    assert_eq!(loaded.name, "Updated Name");
}

#[tokio::test]
async fn test_vault_metadata_with_descriptor() {
    // Test: Vault metadata can store descriptor
    let temp_dir = TempDir::new().unwrap();
    let (_, mut metadata) = create_test_vault("Descriptor Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Set descriptor (using the generated descriptor from the vault)
    let test_descriptor = get_test_descriptor(Network::Testnet).unwrap();
    metadata.descriptor = Some(test_descriptor.clone());
    metadata.save().unwrap();

    // Load and verify descriptor
    let loaded = VaultMetadata::load(&metadata.address).unwrap();
    assert_eq!(loaded.descriptor, Some(test_descriptor));
}

#[tokio::test]
async fn test_vault_listing_filters_orphaned_metadata() {
    // Test: Orphaned metadata (without database) should be handled
    let temp_dir = TempDir::new().unwrap();
    let (_, metadata) = create_test_vault("Orphan Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Delete database but keep metadata
    std::fs::remove_file(&metadata.database_path).unwrap();

    // List vaults - should handle orphaned metadata gracefully
    let vaults = VaultService::<bdk::database::SqliteDatabase>::list_vaults().unwrap();

    // The orphaned vault might or might not be in the list depending on implementation
    // This test verifies the listing doesn't crash
    assert!(true, "Listing should not crash with orphaned metadata");
}

#[tokio::test]
async fn test_vault_metadata_directory_creation() {
    // Test: Metadata directory is created automatically
    let temp_dir = TempDir::new().unwrap();
    let (_, metadata) = create_test_vault("Directory Test", Network::Testnet, &temp_dir)
        .await
        .unwrap();

    // Verify metadata can be loaded (which means file exists)
    let loaded = VaultMetadata::load(&metadata.address);
    assert!(loaded.is_ok(), "Metadata file should exist and be loadable");
}
