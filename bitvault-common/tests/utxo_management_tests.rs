use bitvault_common::utxo_management::UtxoManager;
use bitvault_common::utxo_selection::{Utxo, SelectionStrategy, SelectionResult};
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use std::sync::Once;
use bitvault_common::logging;

// Static initialization for test module
static UTXO_MANAGEMENT_TESTS_INIT: Once = Once::new();

fn setup_utxo_management_tests() {
    UTXO_MANAGEMENT_TESTS_INIT.call_once(|| {
        // Configure minimal logging for tests
        let config = logging::LogConfig {
            level: logging::LogLevel::Error, // Use Error level to minimize output
            log_file: None,                  // No file logging in tests
            include_timestamps: false,
            include_source_location: false,
            max_file_size: 1024 * 1024,
            console_logging: false,          // Disable console logging for tests
            json_format: false,
        };

        // Initialize logging with test configuration
        let _ = logging::init(&config);
    });
}

#[test]
fn test_minimize_fee_selection_single_large_utxo() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let large_utxo = Utxo::new(
        OutPoint::new(Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 0),
        Amount::from_sat(100_000),
        10,
        false,
    );
    manager.add_utxo(large_utxo.clone());

    let target = Amount::from_sat(80_000);
    match manager.select_utxos(target, SelectionStrategy::MinimizeFee) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0], large_utxo);
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_minimize_fee_selection_multiple_utxos() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("1111111111111111111111111111111111111111111111111111111111111111").unwrap(), 0),
        Amount::from_sat(50_000),
        5,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("2222222222222222222222222222222222222222222222222222222222222222").unwrap(), 0),
        Amount::from_sat(60_000),
        5,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());

    let target = Amount::from_sat(100_000);
    match manager.select_utxos(target, SelectionStrategy::MinimizeFee) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 2);
            assert!(selected.contains(&utxo1));
            assert!(selected.contains(&utxo2));
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_maximize_privacy_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("3333333333333333333333333333333333333333333333333333333333333333").unwrap(), 0),
        Amount::from_sat(30_000),
        5,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("4444444444444444444444444444444444444444444444444444444444444444").unwrap(), 0),
        Amount::from_sat(40_000),
        5,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());

    let target = Amount::from_sat(60_000);
    match manager.select_utxos(target, SelectionStrategy::MaximizePrivacy) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 2);
            assert!(selected.contains(&utxo1));
            assert!(selected.contains(&utxo2));
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_consolidate_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("5555555555555555555555555555555555555555555555555555555555555555").unwrap(), 0),
        Amount::from_sat(20_000),
        5,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("6666666666666666666666666666666666666666666666666666666666666666").unwrap(), 0),
        Amount::from_sat(80_000),
        5,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());

    // Test that for a target of 70_000, only the larger UTXO is selected
    let target = Amount::from_sat(70_000);
    match manager.select_utxos(target, SelectionStrategy::Consolidate) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 1);
            assert!(selected.contains(&utxo2));
        },
        _ => panic!("Expected success but got failure"),
    }
    
    // Test with a higher target that requires both UTXOs
    let target = Amount::from_sat(90_000);
    match manager.select_utxos(target, SelectionStrategy::Consolidate) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 2);
            assert!(selected.contains(&utxo1));
            assert!(selected.contains(&utxo2));
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_oldest_first_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("7777777777777777777777777777777777777777777777777777777777777777").unwrap(), 0),
        Amount::from_sat(50_000),
        10, // Higher confirmations (older)
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("8888888888888888888888888888888888888888888888888888888888888888").unwrap(), 0),
        Amount::from_sat(50_000),
        5,  // Lower confirmations (newer)
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());

    // Test that we select the oldest UTXO first (higher confirmations)
    let target = Amount::from_sat(40_000);
    match manager.select_utxos(target, SelectionStrategy::OldestFirst) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 1);
            assert!(selected.contains(&utxo1));
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_coin_control_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("9999999999999999999999999999999999999999999999999999999999999999").unwrap(), 0),
        Amount::from_sat(30_000),
        5,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(), 0),
        Amount::from_sat(40_000),
        2,
        false,
    );
    let utxo3 = Utxo::new(
        OutPoint::new(Txid::from_str("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(), 0),
        Amount::from_sat(50_000),
        8,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());
    manager.add_utxo(utxo3.clone());

    // Test direct selection using outpoints
    let selected_outpoints = vec![utxo1.outpoint, utxo3.outpoint];
    let target = Amount::from_sat(60_000);
    
    match manager.select_coin_control(&selected_outpoints, target) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Verify we get exactly the UTXOs we selected
            assert_eq!(selected.len(), 2);
            assert!(selected.contains(&utxo1));
            assert!(selected.contains(&utxo3));
            
            // Verify fee and change calculations
            let total_selected = Amount::from_sat(30_000 + 50_000);
            
            // Simplified fee calculation for verification
            let input_cost = 2 * 68; // 2 inputs
            let output_cost = 2 * 34; // target + change
            let base_cost = 10;
            let fee_rate = 2;
            let expected_fee = (input_cost + output_cost + base_cost) * fee_rate;
            
            assert_eq!(fee_amount.to_sat(), expected_fee);
            assert_eq!(change_amount.to_sat(), total_selected.to_sat() - target.to_sat() - fee_amount.to_sat());
        },
        _ => panic!("Expected success but got failure"),
    }
    
    // Test insufficient funds
    let selected_outpoints = vec![utxo2.outpoint]; // Only 40,000 sats
    let target = Amount::from_sat(50_000);
    
    match manager.select_coin_control(&selected_outpoints, target) {
        SelectionResult::InsufficientFunds { available, required } => {
            assert_eq!(available.to_sat(), 40_000);
            assert_eq!(required.to_sat(), 50_000);
        },
        _ => panic!("Expected insufficient funds but got success"),
    }
    
    // Test selecting UTXOs with the CoinControl strategy directly (should fail without preselection)
    let target = Amount::from_sat(60_000);
    match manager.select_utxos(target, SelectionStrategy::CoinControl) {
        SelectionResult::InsufficientFunds { .. } => {
            // Expected behavior - CoinControl strategy requires pre-selected UTXOs
        },
        _ => panic!("Expected failure but got success"),
    }
} 