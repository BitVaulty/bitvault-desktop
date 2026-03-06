//! Tests for SQLiteTransactionStorage
//! Equivalent to Swift's SQLiteTransactionStorageTests

use bitvault_app::services::tx_storage::{
    tx_status, RecentAddress, SQLiteTransactionStorage, SyncRecord, TxDetails,
};
use bitvault_common::types::Price;

fn create_test_tx(tx_id: &str, address: &str, status: &str) -> TxDetails {
    TxDetails {
        tx_id: tx_id.to_string(),
        amount_sent: 1.0,
        amount_received: 0.0,
        fee_in_sat: Some(1000),
        prices: Price::default(),
        block_height: Some(800000),
        timestamp: Some(1700000000),
        status: status.to_string(),
        destination_address: Some("addr1".to_string()),
        source_address: Some("addr2".to_string()),
        address: address.to_string(),
        locktime: 0,
        message: None,
        transaction_execution_timestamp: 0,
    }
}

// Helper to create a mutable test tx for timestamp modification
fn create_mut_test_tx(tx_id: &str, address: &str, status: &str) -> TxDetails {
    create_test_tx(tx_id, address, status)
}

#[test]
fn test_insert_and_get_tx() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let tx = create_test_tx("test_tx_1", "test1", tx_status::CONFIRMED);

    storage.insert(&tx).unwrap();
    let all = storage.get_all_txs("test1").unwrap();

    assert_eq!(all.len(), 1);
    assert_eq!(all[0].tx_id, tx.tx_id);
    assert_eq!(all[0].amount_received, tx.amount_received);
    assert_eq!(all[0].amount_sent, tx.amount_sent);
    assert_eq!(all[0].block_height, tx.block_height);
    assert_eq!(all[0].destination_address, tx.destination_address);
    assert_eq!(all[0].source_address, tx.source_address);
    assert_eq!(all[0].fee_in_sat, tx.fee_in_sat);
    assert_eq!(all[0].locktime, tx.locktime);
    assert_eq!(all[0].status, tx.status);
    assert_eq!(all[0].timestamp, tx.timestamp);
    assert_eq!(all[0].message, tx.message);

    // Cleanup
    storage.delete_all_txs_for_address("test1").unwrap();
}

#[test]
fn test_delete_all_txs() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let tx = create_test_tx("test_tx_1", "test1", tx_status::CONFIRMED);

    storage.insert(&tx).unwrap();
    storage.delete_all_txs_for_address("test1").unwrap();
    let all = storage.get_all_txs("test1").unwrap();

    assert!(all.is_empty());
}

#[test]
fn test_sync_record() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let addr = "test1";
    let sync = SyncRecord {
        tx_id: "testid".to_string(),
        timestamp: 123,
    };

    storage
        .save_sync_record(&sync.tx_id, sync.timestamp, addr)
        .unwrap();
    let loaded = storage.get_sync_record(addr).unwrap();

    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.tx_id, sync.tx_id);
    assert_eq!(loaded.timestamp, sync.timestamp);

    storage.delete_sync_record(addr).unwrap();
    let deleted = storage.get_sync_record(addr).unwrap();
    assert!(deleted.is_none());
}

#[test]
fn test_insert_multiple_txs() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let tx1 = create_test_tx("test_tx_1", "test1", tx_status::CONFIRMED);
    let tx2 = create_test_tx("test_tx_2", "test2", tx_status::CONFIRMED);

    storage.insert(&tx1).unwrap();
    let all_after_first = storage.get_all_txs("test1").unwrap();
    assert_eq!(all_after_first.len(), 1);

    storage.insert(&tx2).unwrap();
    let all_after_second = storage.get_all_txs("test2").unwrap();
    assert_eq!(all_after_second.len(), 1); // Different addresses

    // Cleanup
    storage.delete_all_txs_for_address("test1").unwrap();
    storage.delete_all_txs_for_address("test2").unwrap();
}

#[test]
fn test_add_and_get_recent_addresses() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let vault_id = format!(
        "vault1_test_add_get_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let first_address = "tb1qdestinationaddress1111111111111111111111111";
    let second_address = "tb1qdestinationaddress2222222222222222222222222";

    // Clean up first
    let _ = storage.delete_all_addresses(&vault_id);

    // Add first address
    storage
        .add_recent_address(first_address, &vault_id)
        .unwrap();

    // Wait to ensure different timestamp (storage uses seconds, so 1 second minimum)
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Add second address (should have later timestamp)
    storage
        .add_recent_address(second_address, &vault_id)
        .unwrap();

    let recent_addresses = storage.get_recent_addresses(&vault_id).unwrap();
    assert_eq!(recent_addresses.len(), 2, "Should have 2 addresses");

    // Verify ordering - newest (highest timestamp) should be first
    // The second address was added later, so it should have a higher timestamp
    assert!(
        recent_addresses[0].timestamp >= recent_addresses[1].timestamp,
        "Addresses should be ordered by timestamp descending. First: {}, Second: {}",
        recent_addresses[0].timestamp,
        recent_addresses[1].timestamp
    );

    // Since we waited 1 second, the second address should definitely be newer
    assert_eq!(
        recent_addresses[0].recipient_address, second_address,
        "Newest address (added second) should be first. Got: {}",
        recent_addresses[0].recipient_address
    );
    assert_eq!(
        recent_addresses[1].recipient_address, first_address,
        "Older address (added first) should be second. Got: {}",
        recent_addresses[1].recipient_address
    );

    // Cleanup
    storage.delete_all_addresses(&vault_id).unwrap();
}

#[test]
fn test_recent_address_upsert_logic() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let vault_id = "vault1_test_upsert";
    let recipient_address = "tb1qdestinationaddress1111111111111111111111111";

    // Clean up first
    let _ = storage.delete_all_addresses(vault_id);

    storage
        .add_recent_address(recipient_address, vault_id)
        .unwrap();
    let first_result = storage.get_recent_addresses(vault_id).unwrap();
    assert_eq!(
        first_result.len(),
        1,
        "Should have one address after first insert"
    );
    let first_timestamp = first_result[0].timestamp;

    // Use a small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Add the same address again - should update timestamp, not create duplicate
    storage
        .add_recent_address(recipient_address, vault_id)
        .unwrap();
    let second_result = storage.get_recent_addresses(vault_id).unwrap();
    assert_eq!(
        second_result.len(),
        1,
        "There should still be only one entry for this address"
    );
    let second_timestamp = second_result[0].timestamp;

    assert!(
        second_timestamp >= first_timestamp,
        "The timestamp should have been updated to a later time (or same if very fast)"
    );

    // Cleanup
    storage.delete_all_addresses(vault_id).unwrap();
}

#[test]
fn test_delete_all_addresses_for_vault() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let vault1 = "vault1";
    let vault2 = "vault2";

    storage.add_recent_address("addr_v1", vault1).unwrap();
    storage.add_recent_address("addr_v2", vault2).unwrap();

    storage.delete_all_addresses(vault1).unwrap();

    let vault1_addresses = storage.get_recent_addresses(vault1).unwrap();
    let vault2_addresses = storage.get_recent_addresses(vault2).unwrap();

    assert!(
        vault1_addresses.is_empty(),
        "Addresses for vault1 should have been deleted"
    );
    assert_eq!(
        vault2_addresses.len(),
        1,
        "Addresses for vault2 should remain"
    );
    assert_eq!(vault2_addresses[0].recipient_address, "addr_v2");

    // Cleanup
    storage.delete_all_addresses(vault2).unwrap();
}

#[test]
fn test_delete_pending_txs() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let vault_id = "test_vault_for_deletion";

    let confirmed_tx = TxDetails {
        tx_id: "confirmed_tx_1".to_string(),
        amount_sent: 1.0,
        amount_received: 0.0,
        fee_in_sat: Some(1000),
        prices: Price::default(),
        block_height: Some(800000),
        timestamp: Some(1700000000),
        status: tx_status::CONFIRMED.to_string(),
        destination_address: Some("addr1".to_string()),
        source_address: Some("addr2".to_string()),
        address: vault_id.to_string(),
        locktime: 0,
        message: None,
        transaction_execution_timestamp: 0,
    };

    let pending_tx1 = TxDetails {
        tx_id: "pending_tx_1".to_string(),
        amount_sent: 0.5,
        amount_received: 0.0,
        fee_in_sat: Some(500),
        prices: Price::default(),
        block_height: None,
        timestamp: Some(1700000100),
        status: tx_status::PENDING.to_string(),
        destination_address: Some("addr2".to_string()),
        source_address: Some("addr3".to_string()),
        address: vault_id.to_string(),
        locktime: 800144,
        message: Some("Pending 1".to_string()),
        transaction_execution_timestamp: 1700011100,
    };

    let pending_tx2 = TxDetails {
        tx_id: "pending_tx_2".to_string(),
        amount_sent: 0.2,
        amount_received: 0.0,
        fee_in_sat: Some(200),
        prices: Price::default(),
        block_height: None,
        timestamp: Some(1700000200),
        status: tx_status::PENDING.to_string(),
        destination_address: Some("addr3".to_string()),
        source_address: Some("addr4".to_string()),
        address: vault_id.to_string(),
        locktime: 800145,
        message: Some("Pending 2".to_string()),
        transaction_execution_timestamp: 1700011700,
    };

    storage.insert(&confirmed_tx).unwrap();
    storage.insert(&pending_tx1).unwrap();
    storage.insert(&pending_tx2).unwrap();

    let all_before_delete = storage.get_all_txs(vault_id).unwrap();
    assert_eq!(all_before_delete.len(), 3);

    storage.delete_pending_txs(vault_id).unwrap();
    let all_after_delete = storage.get_all_txs(vault_id).unwrap();

    assert_eq!(
        all_after_delete.len(),
        1,
        "Only the confirmed transaction should remain"
    );
    assert_eq!(all_after_delete[0].tx_id, "confirmed_tx_1");
    assert_eq!(all_after_delete[0].status, tx_status::CONFIRMED);

    // Cleanup
    storage.delete_all_txs_for_address(vault_id).unwrap();
}

#[test]
fn test_transaction_ordering() {
    let storage = SQLiteTransactionStorage::new().unwrap();
    let address = "test_ordering";

    // Insert confirmed transaction
    let mut confirmed_tx = create_test_tx("confirmed_1", address, tx_status::CONFIRMED);
    confirmed_tx.timestamp = Some(1700000000);
    storage.insert(&confirmed_tx).unwrap();

    // Insert pending transaction
    let mut pending_tx = create_test_tx("pending_1", address, tx_status::PENDING);
    pending_tx.timestamp = Some(1700000100);
    storage.insert(&pending_tx).unwrap();

    let all = storage.get_all_txs(address).unwrap();

    // Confirmed should come before pending (status ordering)
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].status, tx_status::CONFIRMED);
    assert_eq!(all[1].status, tx_status::PENDING);

    // Cleanup
    storage.delete_all_txs_for_address(address).unwrap();
}
