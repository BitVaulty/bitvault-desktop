use bitvault_common::utxo_management::UtxoManager;
use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;

// This simple test just checks basic UTXO management functionality
#[test]
fn test_utxo_management_basic() {
    println!("Starting basic UTXO management test");
    
    // Create a UTXO manager
    let mut manager = UtxoManager::new();
    
    // Create a test UTXO
    let utxo = Utxo::new(
        OutPoint::new(
            Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            0
        ),
        Amount::from_sat(100_000),
        10, // confirmations
        false, // is_change
    );
    
    // Add the UTXO to the manager
    manager.add_utxo(utxo.clone());
    
    // Verify UTXO selection works
    let target = Amount::from_sat(50_000);
    let result = manager.select_utxos(target, SelectionStrategy::MinimizeFee, None, None);
    
    // Print the result
    println!("UTXO selection result: {:?}", result);
    
    // Check that the selection was successful
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            assert_eq!(selected.len(), 1, "Should select 1 UTXO");
            assert!(selected.contains(&utxo), "Should select our UTXO");
            assert!(fee_amount.to_sat() > 0, "Fee should be positive");
            
            // Calculate expected values
            let total_input = 100_000;
            let target_amount = 50_000;
            let expected_change = total_input - target_amount - fee_amount.to_sat();
            
            assert_eq!(change_amount.to_sat(), expected_change, 
                      "Change should be total_input - target - fee");
            
            println!("Test passed successfully!");
        },
        SelectionResult::InsufficientFunds { available, required } => {
            panic!("Expected Success but got InsufficientFunds: available={}, required={}", 
                  available, required);
        }
    }
} 