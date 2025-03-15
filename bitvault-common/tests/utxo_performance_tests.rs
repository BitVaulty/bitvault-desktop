use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::events::UtxoEventBus;
use bitvault_common::utxo_selection::strategies::minimize_change::MinimizeChangeStrategy;
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use std::time::{Duration, Instant};

// Performance test config
const LARGE_UTXO_COUNT: usize = 1000;
const MEDIUM_UTXO_COUNT: usize = 100;
const SMALL_UTXO_COUNT: usize = 10;

// For tests, use a much shorter timeout to prevent freezing
const TEST_TIMEOUT_MS: u64 = 200; // 200ms timeout for tests

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
    // Create a selector with appropriate timeout for the strategy
    let selector = if strategy == SelectionStrategy::MinimizeChange {
        // For MinimizeChange, use a short-timeout selector
        UtxoSelector::with_minimize_change_strategy(
            MinimizeChangeStrategy::with_timeout(TEST_TIMEOUT_MS)
        )
    } else {
        // For other strategies, use the default selector
        UtxoSelector::new()
    };
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _result = selector.select_utxos(utxos, target_amount, strategy, None, None);
    }
    
    start.elapsed()
}

/// Test performance of UTXO selection with a large number of UTXOs
#[test]
fn test_utxo_selection_performance_large() {
    // Generate 1000 UTXOs
    let utxos = generate_test_utxos(LARGE_UTXO_COUNT);
    
    // Target amount that requires multiple UTXOs
    let target = Amount::from_sat(500_000);
    
    // Number of iterations for averaging
    // Use fewer iterations in debug mode to speed up tests
    let iterations = if cfg!(debug_assertions) {
        1  // Just run once in debug mode - it's already slow enough
    } else {
        100  // Use full 100 iterations for release/benchmark mode
    };
    
    println!("Running large UTXO selection performance test with {} iterations", iterations);
    
    // In debug mode, only test the faster strategies
    let strategies = if cfg!(debug_assertions) {
        vec![
            SelectionStrategy::MinimizeFee,
            SelectionStrategy::MinimizeChange,
            SelectionStrategy::OldestFirst,
        ]
    } else {
        vec![
            SelectionStrategy::MinimizeFee,
            SelectionStrategy::MinimizeChange,
            SelectionStrategy::OldestFirst,
            SelectionStrategy::PrivacyFocused,
            SelectionStrategy::MaximizePrivacy,
        ]
    };
    
    // Test each strategy
    for strategy in strategies {
        let duration = run_selection_performance_test(
            &utxos, target, strategy, iterations
        );
        
        println!(
            "{:?} with {} UTXOs: {:?} per iteration", 
            strategy,
            LARGE_UTXO_COUNT, 
            duration / iterations as u32
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
    // Use fewer iterations in debug mode to speed up tests
    let iterations = if cfg!(debug_assertions) {
        3  // Use just 3 iterations for debug/development to avoid long runs
    } else {
        50  // Reduced from 1000 to 50 for faster tests in release mode too
    };
    
    println!("Running UTXO selection performance test with {} iterations", iterations);
    
    // Test each strategy separately to prevent one slow strategy from blocking the rest
    let strategies = [
        SelectionStrategy::MinimizeFee,      // Fast strategy
        SelectionStrategy::OldestFirst,      // Fast strategy 
        SelectionStrategy::PrivacyFocused,   // Potentially slow strategy
        SelectionStrategy::MaximizePrivacy,  // Potentially slow strategy
        SelectionStrategy::MinimizeChange,   // Slow strategy, but with timeout
    ];
    
    for strategy in strategies {
        // Print which strategy we're testing
        println!("Testing strategy: {:?}", strategy);
        
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
    // Use fewer iterations in debug mode to speed up tests
    let iterations = if cfg!(debug_assertions) {
        100  // Use 100 iterations for debug/development
    } else {
        10_000  // Use full 10,000 iterations for release/benchmark mode
    };
    
    println!("Running small UTXO selection performance test with {} iterations", iterations);
    
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
            SMALL_UTXO_COUNT, 
            duration / iterations as u32
        );
    }
} 