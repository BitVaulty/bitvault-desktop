use bitvault_common::utxo_selection::{
    Utxo, UtxoSelector, SelectionStrategy,
};
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use std::time::{Duration, Instant};

// Performance test config
const LARGE_UTXO_COUNT: usize = 1000;
const MEDIUM_UTXO_COUNT: usize = 100;
const SMALL_UTXO_COUNT: usize = 10;

/// Generate a large set of test UTXOs for performance testing
fn generate_test_utxos(count: usize) -> Vec<Utxo> {
    let mut utxos = Vec::with_capacity(count);
    
    for i in 0..count {
        // Create a deterministic but unique txid
        let txid_hex = format!(
            "{:064x}", 
            0x1000000000000000u64 + (i as u64)
        );
        
        let txid = Txid::from_str(&txid_hex).unwrap();
        
        // Create UTXOs with varying amounts
        let amount = match i % 5 {
            0 => 5_000,                  // Small UTXOs
            1 => 10_000,                 // Medium UTXOs
            2 => 50_000,                 // Larger UTXOs
            3 => 100_000,                // Large UTXOs
            _ => 1_000_000,              // Very large UTXOs
        };
        
        // Add some variety to confirmations
        let confirmations = (i % 10) as u32;
        
        // Make some UTXOs change outputs
        let is_change = i % 3 == 0;
        
        // Create UTXO
        let mut utxo = Utxo::new(
            OutPoint::new(txid, 0),
            Amount::from_sat(amount),
            confirmations,
            is_change
        );
        
        // Add address to some UTXOs
        if i % 2 == 0 {
            utxo.address = Some(format!("bc1q{:038x}", i));
        }
        
        utxos.push(utxo);
    }
    
    utxos
}

/// Run a performance test for UTXO selection with the given parameters
fn run_selection_performance_test(
    utxos: &[Utxo], 
    target_amount: Amount,
    strategy: SelectionStrategy,
    iterations: usize
) -> Duration {
    let selector = UtxoSelector::new();
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _result = selector.select_utxos(utxos, target_amount, strategy);
    }
    
    start.elapsed()
}

/// Test performance of UTXO selection with a large number of UTXOs
#[test]
fn test_utxo_selection_performance_large() {
    // Only run in release mode to get meaningful numbers
    if !cfg!(debug_assertions) {
        // Generate 1000 UTXOs
        let utxos = generate_test_utxos(LARGE_UTXO_COUNT);
        
        // Target amount that requires multiple UTXOs
        let target = Amount::from_sat(500_000);
        
        // Number of iterations for averaging
        let iterations = 100;
        
        // Test MinimizeFee strategy
        let duration_minimize_fee = run_selection_performance_test(
            &utxos, target, SelectionStrategy::MinimizeFee, iterations
        );
        
        println!(
            "MinimizeFee with {} UTXOs: {:?} per iteration", 
            LARGE_UTXO_COUNT, 
            duration_minimize_fee / iterations as u32
        );
        
        // Test MinimizeChange strategy (instead of MinimizeTxSize)
        let duration_minimize_change = run_selection_performance_test(
            &utxos, target, SelectionStrategy::MinimizeChange, iterations
        );
        
        println!(
            "MinimizeChange with {} UTXOs: {:?} per iteration", 
            LARGE_UTXO_COUNT, 
            duration_minimize_change / iterations as u32
        );
        
        // Test MaximizePrivacy strategy (typically slower)
        let privacy_iterations = 10; // Fewer iterations as this is more complex
        let duration_privacy = run_selection_performance_test(
            &utxos, target, SelectionStrategy::MaximizePrivacy, privacy_iterations
        );
        
        println!(
            "MaximizePrivacy with {} UTXOs: {:?} per iteration", 
            LARGE_UTXO_COUNT, 
            duration_privacy / privacy_iterations as u32
        );
        
        // Make sure the branch-and-bound algorithm doesn't take too long
        // This is a very basic performance assertion - adjust as needed
        let threshold = Duration::from_millis(50);
        assert!(
            (duration_minimize_fee / iterations as u32) < threshold,
            "MinimizeFee selection should be fast even with many UTXOs"
        );
    }
}

/// Test performance of UTXO selection with a medium number of UTXOs
#[test]
fn test_utxo_selection_performance_medium() {
    // Generate 100 UTXOs
    let utxos = generate_test_utxos(MEDIUM_UTXO_COUNT);
    
    // Target amount that requires multiple UTXOs
    let target = Amount::from_sat(200_000);
    
    // Number of iterations for averaging
    let iterations = 1000;
    
    // Test each strategy
    for strategy in [
        SelectionStrategy::MinimizeFee,
        SelectionStrategy::MinimizeChange,
        SelectionStrategy::OldestFirst,
        SelectionStrategy::PrivacyFocused,
        SelectionStrategy::MaximizePrivacy,
    ] {
        let duration = run_selection_performance_test(
            &utxos, target, strategy, iterations
        );
        
        println!(
            "{:?} with {} UTXOs: {:?} per iteration", 
            strategy,
            MEDIUM_UTXO_COUNT, 
            duration / iterations as u32
        );
    }
}

/// Test that small UTXO sets are processed very quickly
#[test]
fn test_utxo_selection_performance_small() {
    // Generate 10 UTXOs (common case for most wallets)
    let utxos = generate_test_utxos(SMALL_UTXO_COUNT);
    
    // Target amount that might require 1-2 UTXOs but is low enough to leave room for fees
    // Using 30_000 instead of 50_000 to ensure we leave enough for the static 1000 sat fee
    let target = Amount::from_sat(30_000);
    
    // Number of iterations for averaging
    let iterations = 10_000;
    
    // Print the available UTXOs for debugging
    println!("Available UTXOs:");
    for (i, utxo) in utxos.iter().enumerate() {
        println!("  UTXO {}: {} sats", i, utxo.amount.to_sat());
    }
    
    // Run performance test for all strategies
    for strategy in [
        SelectionStrategy::MinimizeFee,
        SelectionStrategy::MinimizeChange,
        SelectionStrategy::OldestFirst,
        SelectionStrategy::PrivacyFocused,
        SelectionStrategy::MaximizePrivacy,
    ] {
        let duration = run_selection_performance_test(
            &utxos, target, strategy, iterations
        );
        
        println!(
            "{:?} with {} UTXOs: {:?} per iteration", 
            strategy,
            SMALL_UTXO_COUNT, 
            duration / iterations as u32
        );
        
        // Small UTXO sets should be processed in microseconds
        let threshold = Duration::from_micros(500);
        assert!(
            (duration / iterations as u32) < threshold,
            "Selection with small UTXO sets should be very fast"
        );
    }
} 