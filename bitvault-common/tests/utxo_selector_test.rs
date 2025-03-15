use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::events::UtxoEventBus;
use bitvault_common::events::MessageBus;
use bitcoin::{Amount, OutPoint, Txid, Network};
use std::str::FromStr;
use std::fs;

#[test]
fn test_utxo_selector_with_file_output() {
    let mut output = String::new();
    
    output.push_str("Starting utxo_selector_with_file_output test\n");
    
    // Create a test UTXO using the new constructor
    let utxo = Utxo::new(
        OutPoint::new(
            Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            0
        ),
        Amount::from_sat(100_000),
        10, // confirmations
        false, // is_change
    );
    
    output.push_str(&format!("Created UTXO: {:?}\n", utxo));
    
    // Create a selector and select UTXOs
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(50_000);
    
    output.push_str(&format!("Selecting UTXOs for target: {}\n", target));
    let result = selector.select_utxos(&[utxo.clone()], target, SelectionStrategy::MinimizeFee, None, None);
    
    output.push_str(&format!("Selection result: {:?}\n", result));
    
    // Check the result
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            output.push_str(&format!("Success! Selected UTXOs: {}, Fee: {}, Change: {}\n", 
                     selected.len(), fee_amount, change_amount));
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0], utxo);
        },
        SelectionResult::InsufficientFunds { available, required } => {
            output.push_str(&format!("Insufficient funds: available={}, required={}\n", 
                   available, required));
            panic!("Expected success but got insufficient funds");
        },
    }
    
    output.push_str("Test passed!\n");
    
    // Write output to a file in the current directory
    fs::write("utxo_selector_test_output.txt", output).expect("Failed to write output file");
}

#[test]
fn test_utxo_selector_simple() {
    eprintln!("Starting simple UtxoSelector test");
    
    // Create a test UTXO using the new constructor
    let utxo = Utxo::new(
        OutPoint::new(
            Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            0
        ),
        Amount::from_sat(100_000),
        10, // confirmations
        false, // is_change
    );
    
    // Create a selector and select UTXOs
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(50_000);
    
    eprintln!("Selecting UTXOs for target: {}", target);
    let result = selector.select_utxos(&[utxo.clone()], target, SelectionStrategy::MinimizeFee, None, None);
    
    eprintln!("Selection result: {:?}", result);
    
    // Check the result
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            eprintln!("Success! Selected UTXOs: {}, Fee: {}, Change: {}", 
                     selected.len(), fee_amount, change_amount);
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0], utxo);
        },
        SelectionResult::InsufficientFunds { available, required } => {
            panic!("Expected success but got insufficient funds: available={}, required={}", 
                   available, required);
        },
    }
    
    eprintln!("Simple UtxoSelector test passed!");
}

#[test]
fn test_minimize_fee_strategy() {
    eprintln!("Starting minimize_fee_strategy test");
    
    // Create multiple test UTXOs
    let utxo1 = Utxo::new(
        OutPoint::new(
            Txid::from_str("1111111111111111111111111111111111111111111111111111111111111111").unwrap(),
            0
        ),
        Amount::from_sat(30_000),
        5, // confirmations
        false, // is_change
    );
    
    let utxo2 = Utxo::new(
        OutPoint::new(
            Txid::from_str("2222222222222222222222222222222222222222222222222222222222222222").unwrap(),
            0
        ),
        Amount::from_sat(40_000),
        10, // confirmations
        false, // is_change
    );
    
    let utxo3 = Utxo::new(
        OutPoint::new(
            Txid::from_str("3333333333333333333333333333333333333333333333333333333333333333").unwrap(),
            0
        ),
        Amount::from_sat(20_000),
        15, // confirmations
        true, // is_change
    );
    
    eprintln!("Created UTXOs: {:?}, {:?}, {:?}", utxo1, utxo2, utxo3);
    
    // Create a message bus for testing
    let message_bus = MessageBus::new();
    
    // Create a selector
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(50_000);
    
    eprintln!("Selecting UTXOs for target: {}", target);
    let result = selector.select_utxos(
        &[utxo1.clone(), utxo2.clone(), utxo3.clone()], 
        target, 
        SelectionStrategy::MinimizeFee, 
        Some(&message_bus),
        None, // No domain-specific bus for this test
    );
    
    eprintln!("Selection result: {:?}", result);
    
    // Check the result - minimize_fee should prefer larger UTXOs to minimize inputs
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            eprintln!("Success! Selected UTXOs: {}, Fee: {}, Change: {}", 
                     selected.len(), fee_amount, change_amount);
            
            // Should select utxo2 (40_000) and either utxo1 (30_000) or utxo3 (20_000)
            assert!(selected.len() <= 2, "Should select at most 2 UTXOs");
            
            // The largest UTXO should be included
            assert!(selected.contains(&utxo2), "Should include the largest UTXO");
            
            // Total selected amount should be >= target
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            assert!(total_selected >= target.to_sat(), "Total selected should cover target");
            
            eprintln!("minimize_fee_strategy test passed!");
        },
        SelectionResult::InsufficientFunds { available, required } => {
            panic!("Expected success but got insufficient funds: available={}, required={}", 
                   available, required);
        },
    }
} 