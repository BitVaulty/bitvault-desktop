use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::events::UtxoEventBus;
use bitvault_common::utxo_selection::strategies::minimize_change::MinimizeChangeStrategy;
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use std::time::{Duration, Instant};

#[test]
fn test_timeout_fix_basic() {
    // Create a simple UTXO
    let utxo = Utxo::new(
        OutPoint::new(
            Txid::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            0,
        ),
        Amount::from_sat(100_000),
        1,
        false,
    );
    
    // Create a test UTXO set
    let utxos = vec![utxo];
    
    // Create a selector with our timeout fix
    let selector = UtxoSelector::with_minimize_change_strategy(
        MinimizeChangeStrategy::with_timeout(100)
    );
    
    // Target amount smaller than the UTXO
    let target = Amount::from_sat(50_000);
    
    // Should select the UTXO and calculate fee correctly
    match selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            assert_eq!(selected.len(), 1, "Should select one UTXO");
            
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            assert_eq!(total_selected, 100_000, "Should select correct amount");
            
            assert!(fee_amount.to_sat() > 0, "Fee should be positive");
            assert!(change_amount.to_sat() > 0, "Should have change");
            
            let expected_change = 100_000 - 50_000 - fee_amount.to_sat();
            assert_eq!(change_amount.to_sat(), expected_change, "Change should be correctly calculated");
            
            println!("Test passed: selected={}, fee={}, change={}", 
                     total_selected, fee_amount.to_sat(), change_amount.to_sat());
        },
        _ => panic!("Selection failed when it should succeed"),
    }
}

#[test]
fn test_medium_utxo_performance() {
    println!("Starting medium UTXO performance test");
    
    // Generate some test UTXOs - similar to the performance test
    let mut utxos = Vec::with_capacity(100);
    
    for i in 0..100 {
        // Create a deterministic but unique txid
        let txid_hex = format!(
            "{:064x}", 
            0x1000000000000000u64 + (i as u64)
        );
        
        let txid = Txid::from_str(&txid_hex).unwrap();
        
        // Create UTXOs with varying amounts
        let amount = match i % 5 {
            0 => 5_000,       // Small UTXOs
            1 => 10_000,      // Medium UTXOs
            2 => 50_000,      // Larger UTXOs
            3 => 100_000,     // Large UTXOs
            _ => 1_000_000,   // Very large UTXOs
        };
        
        // Add some variety to confirmations
        let confirmations = (i % 10) as u32;
        
        // Make some UTXOs change outputs
        let is_change = i % 3 == 0;
        
        utxos.push(Utxo::new(
            OutPoint::new(txid, 0),
            Amount::from_sat(amount),
            confirmations,
            is_change
        ));
    }
    
    // Target amount that requires multiple UTXOs
    let target = Amount::from_sat(200_000);
    
    // Create selector with timeout
    let selector = UtxoSelector::with_minimize_change_strategy(
        MinimizeChangeStrategy::with_timeout(300) // 300ms should be enough
    );
    
    println!("Running MinimizeChange selection with 100 UTXOs");
    
    let start = Instant::now();
    
    // Run the selection
    match selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            let duration = start.elapsed();
            println!("Selection completed in {:?}", duration);
            
            // Verify the selection makes sense
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("Selected {} UTXOs with total amount of {} sats", selected.len(), total_selected);
            println!("Fee: {} sats", fee_amount.to_sat());
            println!("Change: {} sats", change_amount.to_sat());
            
            // Basic assertions
            assert!(!selected.is_empty(), "Should select at least one UTXO");
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(), "Selected amount should cover target + fee");
            assert!(duration < Duration::from_secs(5), "Selection should complete in reasonable time");
            
            println!("Test passed: selection returned successfully in reasonable time");
        },
        SelectionResult::InsufficientFunds { available, required } => {
            panic!("Selection failed with insufficient funds: available={}, required={}", 
                  available.to_sat(), required.to_sat());
        }
    }
} 