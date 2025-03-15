use bitvault_common::utxo_management::UtxoManager;
use bitvault_common::utxo_selection::types::{Utxo, SelectionResult, SelectionStrategy};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::events::UtxoEventBus;
use bitcoin::{Amount, OutPoint, Txid, Network};
use std::str::FromStr;
use rust_decimal::Decimal;
use std::io::{self, Write};

// Add a test module initializer that runs before all tests
#[cfg(test)]
#[ctor::ctor]
fn init() {
    // Initialize env_logger to see log messages
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {} - {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
    
    eprintln!("TEST INITIALIZATION COMPLETE - YOU SHOULD SEE THIS MESSAGE");
}

// Directly compare UtxoManager and UtxoSelector with identical inputs
#[test]
fn test_compare_manager_and_selector() {
    // Create identical test data for both approaches
    let utxo: Utxo = Utxo::new(
        OutPoint::new(Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 0),
        Amount::from_sat(100_000),
        10,
        false,
    );
    let target = Amount::from_sat(80_000);
    
    // Approach 1: Direct UtxoSelector
    let selector = UtxoSelector::new();
    let selector_result: SelectionResult = selector.select_utxos(&[utxo.clone()], target, SelectionStrategy::MinimizeFee, None, None);
    
    // Approach 2: Via UtxoManager
    let mut manager = UtxoManager::new();
    manager.add_utxo(utxo.clone());
    let manager_result: SelectionResult = manager.select_utxos(target, SelectionStrategy::MinimizeFee, None, None);
    
    // Compare results by using basic checks rather than direct equality
    match (selector_result, manager_result) {
        (SelectionResult::Success { selected: s1, .. }, SelectionResult::Success { selected: s2, .. }) => {
            // Both succeeded
            assert_eq!(s1.len(), s2.len(), "Selected UTXO count mismatch: selector: {}, manager: {}", s1.len(), s2.len());
            if s1.len() == 1 && s2.len() == 1 {
                assert_eq!(s1[0].outpoint, s2[0].outpoint, "Selected UTXOs have different outpoints");
                assert_eq!(s1[0].amount, s2[0].amount, "Selected UTXOs have different amounts");
            }
        },
        (SelectionResult::InsufficientFunds { .. }, SelectionResult::InsufficientFunds { .. }) => {
            // Both failed due to insufficient funds - that's consistent
            panic!("Both methods reported insufficient funds, but this should have worked!");
        },
        (SelectionResult::Success { .. }, SelectionResult::InsufficientFunds { available, required }) => {
            // Selector succeeded but Manager failed
            panic!("Direct selector succeeded but manager reported insufficient funds: available={}, required={}", available, required);
        },
        (SelectionResult::InsufficientFunds { available, required }, SelectionResult::Success { .. }) => {
            // Manager succeeded but Selector failed
            panic!("Direct selector reported insufficient funds but manager succeeded: available={}, required={}", available, required);
        },
    }
}

// Test to verify UTXO cloning/passing behavior
#[test]
fn test_utxo_passing_behavior() {
    // Create a test UTXO
    let original_utxo: Utxo = Utxo::new(
        OutPoint::new(Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 0),
        Amount::from_sat(100_000),
        10,
        false,
    );
    
    // Add to manager and retrieve
    let mut manager = UtxoManager::new();
    manager.add_utxo(original_utxo.clone());
    
    // Test direct equality of UTXOs
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(50_000); // Small enough to definitely succeed
    
    // Get result from selector with a single UTXO in a vector
    let direct_result: SelectionResult = selector.select_utxos(&[original_utxo.clone()], target, SelectionStrategy::MinimizeFee, None, None);
    
    // Make very basic assertions
    match direct_result {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 1, "Should only select one UTXO");
            let selected_utxo = &selected[0];
            assert_eq!(selected_utxo.outpoint, original_utxo.outpoint, "Outpoint mismatch");
            assert_eq!(selected_utxo.amount, original_utxo.amount, "Amount mismatch");
            assert_eq!(selected_utxo.confirmations, original_utxo.confirmations, "Confirmations mismatch");
            assert_eq!(selected_utxo.is_change, original_utxo.is_change, "is_change mismatch");
            assert_eq!(selected_utxo.is_frozen, original_utxo.is_frozen, "is_frozen mismatch");
        },
        _ => panic!("Direct selection should have succeeded"),
    }
}

// Test with very simple UTXO and target values
#[test]
fn test_simple_minimization() {
    // Set up a very basic test case
    eprintln!("Starting test_simple_minimization");
    
    // Create a UTXO with a specific value using the correct structure
    let utxo: Utxo = Utxo {
        outpoint: OutPoint::new(
            Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 
            0
        ),
        amount: Amount::from_sat(100_000),
        confirmations: 1,
        is_change: false,
        is_frozen: false,
        address: Some("addr1".to_string()),
        derivation_path: None,
        label: None,
        network: Network::Testnet,
    };
    
    eprintln!("Created test UTXO: {:?}", utxo);
    
    let utxos = vec![utxo.clone()];
    let target_amount = Amount::from_sat(50_000);
    
    eprintln!("Target amount: {}", target_amount);
    
    // Test UtxoSelector directly
    let selector = UtxoSelector::new();
    eprintln!("Created UtxoSelector");
    
    let selector_result: SelectionResult = selector.select_utxos(&utxos, target_amount, SelectionStrategy::MinimizeFee, None, None);
    eprintln!("UtxoSelector result: {:?}", selector_result);
    
    // Test UtxoManager
    let mut manager = UtxoManager::new();
    eprintln!("Created UtxoManager");
    
    manager.add_utxo(utxo.clone());
    eprintln!("Added UTXO to manager");
    
    let manager_result: SelectionResult = manager.select_utxos(target_amount, SelectionStrategy::MinimizeFee, None, None);
    eprintln!("UtxoManager result: {:?}", manager_result);
    
    // Compare results
    match (selector_result, manager_result) {
        (SelectionResult::Success { selected: selector_selected, .. }, 
         SelectionResult::Success { selected: manager_selected, .. }) => {
            eprintln!("Both selections succeeded");
            eprintln!("Selector selected {} UTXOs", selector_selected.len());
            eprintln!("Manager selected {} UTXOs", manager_selected.len());
            
            assert_eq!(selector_selected.len(), manager_selected.len(), 
                "Both should select the same number of UTXOs");
        },
        (SelectionResult::InsufficientFunds { available: selector_available, required: selector_required }, 
         SelectionResult::InsufficientFunds { available: manager_available, required: manager_required }) => {
            eprintln!("Both selections failed due to insufficient funds");
            eprintln!("Selector: available {}, required {}", selector_available, selector_required);
            eprintln!("Manager: available {}, required {}", manager_available, manager_required);
            
            assert_eq!(selector_available, manager_available, "Available amounts should match");
            assert_eq!(selector_required, manager_required, "Required amounts should match");
        },
        (SelectionResult::Success { selected, .. }, 
         SelectionResult::InsufficientFunds { available, required }) => {
            eprintln!("Selector succeeded but Manager failed");
            eprintln!("Selector selected {} UTXOs", selected.len());
            eprintln!("Manager: available {}, required {}", available, required);
            
            panic!("Inconsistent results: Selector succeeded but Manager failed");
        },
        (SelectionResult::InsufficientFunds { available, required }, 
         SelectionResult::Success { selected, .. }) => {
            eprintln!("Selector failed but Manager succeeded");
            eprintln!("Selector: available {}, required {}", available, required);
            eprintln!("Manager selected {} UTXOs", selected.len());
            
            panic!("Inconsistent results: Selector failed but Manager succeeded");
        },
    }
    
    eprintln!("test_simple_minimization completed");
}

#[test]
fn test_basic_output() {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    
    writeln!(handle, "DIRECT TEST OUTPUT TO STDERR").unwrap();
    writeln!(handle, "THIS SHOULD DEFINITELY BE VISIBLE").unwrap();
    
    // Drop the handle to flush and unlock stderr
    drop(handle);
    
    // Also try eprintln! for comparison
    eprintln!("EPRINTLN OUTPUT TEST");
    
    // Force a flush of stdout/stderr
    std::io::stdout().flush().unwrap();
    std::io::stderr().flush().unwrap();
    
    // Signal that we got to this point
    assert!(true, "This test should pass");
}

// UTXO Selection Debug Tests
//
// FINDINGS AND RECOMMENDATIONS:
//
// 1. We've observed that tests are running but we can't see any output or create files.
//    This suggests a sandboxed test environment with restricted I/O.
//
// 2. The test_minimize_fee_selection_single_large_utxo test is failing (exit code 101),
//    but we can't see the specific error due to output restrictions.
//
// 3. Basic tests pass, suggesting the issue is specific to UTXO selection logic.
//
// 4. Based on our analysis, the most likely issues are:
//    a. Inconsistency between UtxoManager and UtxoSelector implementations
//    b. Fee calculation issues in the minimize_fee_strategy
//    c. Incorrect handling of the message_bus parameter
//
// 5. Recommended fixes:
//    a. Ensure UtxoManager.select_utxos properly passes parameters to UtxoSelector
//    b. Verify that fee calculations in minimize_fee_strategy are correct
//    c. Check that message_bus is handled consistently across all strategy methods
//    d. Consider adding more robust error handling and reporting

#[test]
fn test_utxo_debug() {
    println!("Starting utxo_debug test");
    
    // Create a test UTXO
    let utxo = Utxo::new(
        OutPoint::new(Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 0),
        Amount::from_sat(100_000),
        10,
        false,
    );
    
    // Create a selector and select UTXOs
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(80_000);
    let result = selector.select_utxos(&[utxo.clone()], target, SelectionStrategy::MinimizeFee, None, None);
    
    // Print the result
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            println!("Selection successful:");
            println!("  Selected UTXOs: {}", selected.len());
            println!("  Fee amount: {} sats", fee_amount.to_sat());
            println!("  Change amount: {} sats", change_amount.to_sat());
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("  Total selected: {} sats", total_selected);
            
            // Verify balance equation
            let left_side = total_selected;
            let right_side = target.to_sat() + fee_amount.to_sat() + change_amount.to_sat();
            println!("  Balance equation: {} = {} + {} + {}", left_side, target.to_sat(), fee_amount.to_sat(), change_amount.to_sat());
            println!("  Difference: {}", if left_side > right_side { left_side - right_side } else { right_side - left_side });
        },
        SelectionResult::InsufficientFunds { .. } => {
            println!("Insufficient funds");
        },
    }
    
    println!("utxo_debug test completed");
}

#[test]
fn test_coin_control_debug() {
    println!("Starting coin_control_debug test");
    
    // Create a test UTXO
    let utxo = Utxo::new(
        OutPoint::new(Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 0),
        Amount::from_sat(100_000),
        10,
        false,
    );
    
    // Create a selector and select UTXOs using coin control
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(80_000);
    let result = selector.select_coin_control(&[utxo.clone()], target, None, None);
    
    // Print the result
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            println!("Coin control selection successful:");
            println!("  Selected UTXOs: {}", selected.len());
            println!("  Fee amount: {} sats", fee_amount.to_sat());
            println!("  Change amount: {} sats", change_amount.to_sat());
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("  Total selected: {} sats", total_selected);
            
            // Verify balance equation
            let left_side = total_selected;
            let right_side = target.to_sat() + fee_amount.to_sat() + change_amount.to_sat();
            println!("  Balance equation: {} = {} + {} + {}", left_side, target.to_sat(), fee_amount.to_sat(), change_amount.to_sat());
            println!("  Difference: {}", if left_side > right_side { left_side - right_side } else { right_side - left_side });
        },
        SelectionResult::InsufficientFunds { .. } => {
            println!("Insufficient funds");
        },
    }
    
    println!("coin_control_debug test completed");
} 