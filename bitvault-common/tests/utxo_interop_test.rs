use bitcoin::{Amount, OutPoint, Txid, Network};
use std::str::FromStr;
use bitvault_common::utxo_management::UtxoManager;
use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::events::UtxoEventBus;

#[test]
fn test_utxo_interoperability() {
    // Create a simple UTXO
    let utxo = Utxo::new(
        OutPoint::new(
            Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            0
        ),
        Amount::from_sat(100_000),
        10, // confirmations
        false // not change
    );
    
    // Test with UtxoManager
    let mut manager = UtxoManager::new();
    manager.add_utxo(utxo);
    
    // Select UTXOs
    let target = Amount::from_sat(50_000);
    let result = manager.select_utxos(target, SelectionStrategy::MinimizeFee, None, None);
    
    // Verify result
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            assert_eq!(selected.len(), 1, "Should select 1 UTXO");
            assert!(fee_amount.to_sat() > 0, "Fee should be positive");
            assert!(change_amount.to_sat() > 0, "Change should be positive");
        },
        SelectionResult::InsufficientFunds { available, required } => {
            panic!("Expected Success but got InsufficientFunds: available={}, required={}", 
                  available, required);
        }
    }
} 