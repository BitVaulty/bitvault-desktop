use bitvault_common::utxo_selection::{Utxo, UtxoSelector, SelectionStrategy, SelectionResult};
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use log::{info, error};

#[test]
fn test_maximize_privacy_with_shared_address() {
    // Initialize logging
    env_logger::builder().is_test(true).init();

    info!("Starting test_maximize_privacy_with_shared_address");
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

    // Simulate a transaction creation with MaximizePrivacy strategy
    // Use the standard selector with an explicit fee rate
    let selector = UtxoSelector::with_fee_rate(1.0); // 1 sat/vByte
    let target = Amount::from_sat(30_000);

    info!("Selecting UTXOs for target: {} satoshis with MaximizePrivacy strategy", target.to_sat());

    let selection_result = selector.select_utxos(&utxos, target, SelectionStrategy::MaximizePrivacy);
    info!("Result of UTXO selection: {:?}", selection_result);

    match selection_result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            info!("Selected UTXOs: {:?}", selected);
            info!("Transaction fee: {} satoshis", fee_amount.to_sat());
            info!("Change amount: {} satoshis", change_amount.to_sat());

            // Verify selected UTXOs
            assert!(!selected.is_empty(), "No UTXOs selected");

            // Verify transaction fee
            assert!(fee_amount.to_sat() > 0, "Fee should be greater than zero");

            // Verify we have enough funds
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(),
                "Selected amount should cover target plus fee");
        },
        _ => {
            error!("Expected success but got failure");
            panic!("Expected success but got failure");
        },
    }
} 