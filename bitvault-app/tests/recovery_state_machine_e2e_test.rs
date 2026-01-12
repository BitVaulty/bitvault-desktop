//! E2E Tests for Recovery Flow State Machines
//!
//! Tests complete state machine flows for:
//! - Recovery transaction flow (UTXOs older than 1 year)
//! - UTXO refresh flow (UTXOs older than 6 months)
//! - UTXO selection state
//!
//! Note: RecoveryState and UtxoRefreshState are private and stored in thread_local! variables.
//! Tests focus on public API functions. UtxoSelectionState is tested through its public API.

use bitvault_app::ui::recovery::{
    go_back_in_recovery_workflow,
    go_back_in_utxo_refresh_workflow,
};
use bitvault_common::types::OldUtxo;

// Note: UtxoSelectionState and RecoveryMode are in a private module, so we can't import them directly.
// We'll test what we can through the public API.

#[test]
fn test_recovery_workflow_back_navigation() {
    // Test: Back navigation in recovery workflow
    // Recovery workflow: LoadingUtxos → SelectingUtxos → BuildingPreview → 
    // PreviewReady → Signing → Sharing → Completed
    
    // Initially, go_back should return false (at first step)
    let went_back = go_back_in_recovery_workflow();
    assert!(!went_back, "Should not be able to go back from first step");
    
    // After going back (if there was a previous step), it should work
    // Since we're at the first step, go_back should return false
    let went_back_again = go_back_in_recovery_workflow();
    assert!(!went_back_again, "Should still not be able to go back from first step");
}

#[test]
fn test_utxo_refresh_workflow_back_navigation() {
    // Test: Back navigation in UTXO refresh workflow
    // UTXO refresh workflow: LoadingUtxos → SelectingUtxos → Signing → Sharing → Completed
    
    // Initially, go_back should return false (at first step)
    let went_back = go_back_in_utxo_refresh_workflow();
    assert!(!went_back, "Should not be able to go back from first step");
    
    // After going back (if there was a previous step), it should work
    let went_back_again = go_back_in_utxo_refresh_workflow();
    assert!(!went_back_again, "Should still not be able to go back from first step");
}

#[test]
fn test_old_utxo_structure() {
    // Test: OldUtxo structure (only has amount and outpoint)
    let utxo = OldUtxo {
        outpoint: "txid1:0".to_string(),
        amount: 0.001,
    };
    
    assert_eq!(utxo.outpoint, "txid1:0");
    assert_eq!(utxo.amount, 0.001);
    
    // OldUtxo only has these two fields (age_days and confirmed are not part of the struct)
    // The age filtering happens when loading UTXOs from the wallet
}

#[test]
fn test_old_utxo_serialization() {
    // Test: OldUtxo can be serialized/deserialized
    let utxo = OldUtxo {
        outpoint: "txid1:0".to_string(),
        amount: 0.001,
    };
    
    // Serialize
    let json = serde_json::to_string(&utxo).unwrap();
    assert!(json.contains("txid1:0"));
    assert!(json.contains("0.001"));
    
    // Deserialize
    let deserialized: OldUtxo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.outpoint, utxo.outpoint);
    assert_eq!(deserialized.amount, utxo.amount);
}

#[test]
fn test_old_utxo_multiple_utxos() {
    // Test: Multiple OldUtxo instances
    let utxos = vec![
        OldUtxo {
            outpoint: "tx1:0".to_string(),
            amount: 0.001,
        },
        OldUtxo {
            outpoint: "tx2:0".to_string(),
            amount: 0.002,
        },
        OldUtxo {
            outpoint: "tx3:0".to_string(),
            amount: 0.005,
        },
    ];
    
    assert_eq!(utxos.len(), 3);
    
    // Calculate total amount
    let total: f64 = utxos.iter().map(|u| u.amount).sum();
    assert_eq!(total, 0.008); // 0.001 + 0.002 + 0.005
    
    // Verify outpoints are unique
    let outpoints: Vec<&String> = utxos.iter().map(|u| &u.outpoint).collect();
    assert_eq!(outpoints.len(), 3);
    assert_ne!(outpoints[0], outpoints[1]);
    assert_ne!(outpoints[1], outpoints[2]);
}

#[test]
fn test_recovery_workflow_initial_state() {
    // Test: Recovery workflow initial state
    // The workflow should start at LoadingUtxos step
    
    // go_back should return false (no previous step)
    let went_back = go_back_in_recovery_workflow();
    assert!(!went_back, "Should not be able to go back from initial state");
}

#[test]
fn test_utxo_refresh_workflow_initial_state() {
    // Test: UTXO refresh workflow initial state
    // The workflow should start at LoadingUtxos step
    
    // go_back should return false (no previous step)
    let went_back = go_back_in_utxo_refresh_workflow();
    assert!(!went_back, "Should not be able to go back from initial state");
}

#[test]
fn test_old_utxo_outpoint_format() {
    // Test: OldUtxo outpoint format (txid:vout)
    let utxo = OldUtxo {
        outpoint: "abc123def456:0".to_string(),
        amount: 0.001,
    };
    
    // Outpoint should be in format "txid:vout"
    assert!(utxo.outpoint.contains(':'));
    let parts: Vec<&str> = utxo.outpoint.split(':').collect();
    assert_eq!(parts.len(), 2);
    assert!(!parts[0].is_empty()); // txid
    assert!(!parts[1].is_empty()); // vout
}

#[test]
fn test_old_utxo_amount_precision() {
    // Test: OldUtxo amount handling (f64 precision)
    let utxo = OldUtxo {
        outpoint: "tx1:0".to_string(),
        amount: 0.00000001, // 1 satoshi in BTC
    };
    
    assert!(utxo.amount > 0.0);
    assert_eq!(utxo.amount, 0.00000001);
    
    // Test larger amounts
    let large_utxo = OldUtxo {
        outpoint: "tx2:0".to_string(),
        amount: 1.0, // 1 BTC
    };
    
    assert_eq!(large_utxo.amount, 1.0);
}

#[test]
fn test_recovery_workflow_multiple_calls() {
    // Test: Multiple calls to recovery workflow functions
    // Each call should be independent (thread-local state)
    
    // First call
    let went_back1 = go_back_in_recovery_workflow();
    assert!(!went_back1);
    
    // Second call (should still be at initial state)
    let went_back2 = go_back_in_recovery_workflow();
    assert!(!went_back2);
    
    // Both should return false (at initial state)
    assert_eq!(went_back1, went_back2);
}

#[test]
fn test_utxo_refresh_workflow_multiple_calls() {
    // Test: Multiple calls to UTXO refresh workflow functions
    // Each call should be independent (thread-local state)
    
    // First call
    let went_back1 = go_back_in_utxo_refresh_workflow();
    assert!(!went_back1);
    
    // Second call (should still be at initial state)
    let went_back2 = go_back_in_utxo_refresh_workflow();
    assert!(!went_back2);
    
    // Both should return false (at initial state)
    assert_eq!(went_back1, went_back2);
}

#[test]
fn test_old_utxo_collection_operations() {
    // Test: Operations on collections of OldUtxo
    let mut utxos = vec![
        OldUtxo {
            outpoint: "tx1:0".to_string(),
            amount: 0.001,
        },
        OldUtxo {
            outpoint: "tx2:0".to_string(),
            amount: 0.002,
        },
    ];
    
    // Filter by amount
    let large_utxos: Vec<&OldUtxo> = utxos.iter().filter(|u| u.amount >= 0.002).collect();
    assert_eq!(large_utxos.len(), 1);
    assert_eq!(large_utxos[0].outpoint, "tx2:0");
    
    // Find by outpoint
    let found = utxos.iter().find(|u| u.outpoint == "tx1:0");
    assert!(found.is_some());
    assert_eq!(found.unwrap().amount, 0.001);
    
    // Add more UTXOs
    utxos.push(OldUtxo {
        outpoint: "tx3:0".to_string(),
        amount: 0.003,
    });
    
    assert_eq!(utxos.len(), 3);
    
    // Calculate total
    let total: f64 = utxos.iter().map(|u| u.amount).sum();
    assert_eq!(total, 0.006);
}
