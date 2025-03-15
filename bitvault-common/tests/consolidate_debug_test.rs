use bitvault_common::utxo_management::UtxoManager;
use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;

#[test]
fn test_consolidate_strategy_debug() {
    println!("Starting consolidate_strategy_debug test");
    
    // Create a new UtxoManager
    let mut manager = UtxoManager::new();
    
    // Create two UTXOs
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("1111111111111111111111111111111111111111111111111111111111111111").unwrap(), 0),
        Amount::from_sat(40_000),
        1,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("2222222222222222222222222222222222222222222222222222222222222222").unwrap(), 0),
        Amount::from_sat(50_000),
        2,
        false,
    );
    
    println!("Created UTXOs: {} sats and {} sats", utxo1.amount.to_sat(), utxo2.amount.to_sat());
    
    // Add the UTXOs to the manager
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());
    
    println!("Added UTXOs to manager");
    
    // Test with a target amount close to the total available
    let target = Amount::from_sat(90_000);
    println!("Testing with high target amount: {} sats (close to total)", target.to_sat());
    
    match manager.select_utxos(target, SelectionStrategy::Consolidate, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            println!("Consolidate test passed successfully!");
            println!("Selected UTXOs: {}", selected.len());
            println!("Total selected: {}", selected.iter().map(|u| u.amount.to_sat()).sum::<u64>());
            println!("Fee amount: {}", fee_amount.to_sat());
            println!("Change amount: {}", change_amount.to_sat());
            
            // Verify the fundamental balance equation
            let total_selected = selected.iter().map(|u| u.amount.to_sat()).sum::<u64>();
            assert_eq!(total_selected, target.to_sat() + fee_amount.to_sat() + change_amount.to_sat());
        },
        SelectionResult::InsufficientFunds { available, required } => {
            // Should be a valid outcome if the fee estimation puts us over the total
            println!("Insufficient funds: {} available, {} required", available.to_sat(), required.to_sat());
        },
    }
    
    // Test with a target amount significantly below the total available
    let target = Amount::from_sat(70_000);
    println!("\nTesting with lower target amount: {} sats", target.to_sat());
    
    match manager.select_utxos(target, SelectionStrategy::Consolidate, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            println!("Consolidate test lower target passed successfully!");
            println!("Selected UTXOs: {}", selected.len());
            println!("Total selected: {}", selected.iter().map(|u| u.amount.to_sat()).sum::<u64>());
            println!("Fee amount: {}", fee_amount.to_sat());
            println!("Change amount: {}", change_amount.to_sat());
            
            // Verify the fundamental balance equation
            let total_selected = selected.iter().map(|u| u.amount.to_sat()).sum::<u64>();
            assert_eq!(total_selected, target.to_sat() + fee_amount.to_sat() + change_amount.to_sat());
        },
        SelectionResult::InsufficientFunds { available, required } => {
            panic!("Expected success but got insufficient funds: {} available, {} required", 
                available.to_sat(), required.to_sat());
        },
    }
    
    println!("Test completed successfully");
} 