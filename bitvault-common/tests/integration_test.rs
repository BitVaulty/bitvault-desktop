use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult, UtxoSet};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::events::UtxoEventBus;
use bitcoin::{Amount, OutPoint, Txid, Network};
use std::str::FromStr;
use log::{info, error};
use std::sync::Once;

// Initialize logging once
static INIT: Once = Once::new();

fn init_logging() {
    INIT.call_once(|| {
        env_logger::builder().is_test(true).init();
    });
}

#[test]
fn test_utxo_transaction_integration() {
    init_logging();

    // Setup mock UTXOs
    let utxos = vec![
        Utxo::new(
            OutPoint::new(
                Txid::from_str("7967a5185e907a25225574544c31f7b059c1a191d65b53dcc1554d339c4f9efc").unwrap(),
                0,
            ),
            Amount::from_sat(10_000),
            0,
            false,
        ),
        Utxo::new(
            OutPoint::new(
                Txid::from_str("9dcbf5a86b4e70be97fc5c953ad4111dfe0a94ea6768286e5efd6c35fd9ec9d1").unwrap(),
                1,
            ),
            Amount::from_sat(50_000),
            2,
            true,
        ),
    ];

    info!("UTXOs setup: {:?}", utxos);

    // Create a UTXO set
    let mut utxo_set = UtxoSet::new(Vec::new(), Network::Bitcoin);
    for utxo in &utxos {
        utxo_set.add(utxo.clone());
    }

    // Simulate a transaction creation
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(30_000);

    info!("Selecting UTXOs for target: {} satoshis", target.to_sat());

    let result = selector.select_utxos(&utxos[..], target, SelectionStrategy::MinimizeFee, None, None);

    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            info!("Selected UTXOs: {:?}", selected);
            info!("Transaction fee: {} satoshis", fee_amount.to_sat());
            info!("Change amount: {} satoshis", change_amount.to_sat());

            // Verify selected UTXOs
            assert!(!selected.is_empty(), "No UTXOs selected");

            // Verify transaction fee
            assert!(fee_amount.to_sat() > 0, "Fee should be greater than zero");

            // Verify change amount
            // assert!(change_amount.to_sat() >= 0, "Change should be non-negative");
        },
        _ => {
            error!("Expected success but got failure");
            panic!("Expected success but got failure");
        },
    }
}

#[test]
fn test_utxo_selection_maximize_privacy_integration() {
    init_logging();

    info!("Starting test_utxo_selection_maximize_privacy_integration");
    // Setup mock UTXOs
    let mut utxos = vec![
        Utxo::new(
            OutPoint::new(
                Txid::from_str("7967a5185e907a25225574544c31f7b059c1a191d65b53dcc1554d339c4f9efc").unwrap(),
                0,
            ),
            Amount::from_sat(10_000),
            0,
            false,
        ),
        Utxo::new(
            OutPoint::new(
                Txid::from_str("9dcbf5a86b4e70be97fc5c953ad4111dfe0a94ea6768286e5efd6c35fd9ec9d1").unwrap(),
                1,
            ),
            Amount::from_sat(50_000),
            2,
            true,
        ),
    ];

    // Add addresses to UTXOs for privacy testing
    let shared_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string();
    utxos[0].address = Some(shared_address.clone());
    utxos[1].address = Some(shared_address);

    info!("UTXOs setup for MaximizePrivacy: {:?}", utxos);

    // Create a UTXO set
    let mut utxo_set = UtxoSet::new(Vec::new(), Network::Bitcoin);
    for utxo in &utxos {
        utxo_set.add(utxo.clone());
    }

    // Simulate a transaction creation with MaximizePrivacy strategy
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(30_000);

    info!("Selecting UTXOs for target: {} satoshis with MaximizePrivacy strategy", target.to_sat());

    let result = selector.select_utxos(&utxos[..], target, SelectionStrategy::MaximizePrivacy, None, None);
    info!("Result of UTXO selection: {:?}", result);

    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            info!("Selected UTXOs: {:?}", selected);
            info!("Transaction fee: {} satoshis", fee_amount.to_sat());
            info!("Change amount: {} satoshis", change_amount.to_sat());

            // Verify selected UTXOs
            assert!(!selected.is_empty(), "No UTXOs selected");

            // Verify transaction fee
            assert!(fee_amount.to_sat() > 0, "Fee should be greater than zero");

            // Remove unnecessary comparison
            // assert!(change_amount.to_sat() >= 0, "Change should be non-negative");
        },
        _ => {
            error!("Expected success but got failure");
            panic!("Expected success but got failure");
        },
    }
}

#[test]
fn test_debug_test_runner() {
    println!("Debug test runner is working");
    assert!(true, "This test should always pass");
}
