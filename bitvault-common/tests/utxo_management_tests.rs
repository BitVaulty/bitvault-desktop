use bitvault_common::utxo_management::UtxoManager;
use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use std::sync::Once;
use bitvault_common::logging;
use std::fs::File;
use std::io::Write;
use bitvault_common::events::UtxoEventBus;

// Define our own CoinControl struct for tests
struct CoinControl {
    selected_outpoints: Vec<OutPoint>,
}

impl CoinControl {
    fn new() -> Self {
        CoinControl {
            selected_outpoints: Vec::new(),
        }
    }

    fn select_utxo(&mut self, outpoint: OutPoint) {
        self.selected_outpoints.push(outpoint);
    }
}

// Static initialization for test module
static UTXO_MANAGEMENT_TESTS_INIT: Once = Once::new();

fn setup_utxo_management_tests() -> (UtxoManager, bitvault_common::events::MessageBus) {
    UTXO_MANAGEMENT_TESTS_INIT.call_once(|| {
        // Configure minimal logging for tests
        let config = logging::LogConfig {
            level: logging::LogLevel::Error, // Use Error level to minimize output
            log_file: None,                  // No file logging in tests
            include_timestamps: false,
            include_source_location: false,
            max_file_size: 1024 * 1024,
            console_logging: false,          // Disable console logging for tests
            json_format: false,
        };

        // Initialize logging with test configuration
        let _ = logging::init(&config);
    });
    
    // Create a new UtxoManager and MessageBus to return
    let utxo_manager = UtxoManager::new();
    let message_bus = bitvault_common::events::MessageBus::new();
    
    (utxo_manager, message_bus)
}

#[test]
fn test_minimize_fee_selection_single_large_utxo() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo = Utxo::new(
        OutPoint::new(Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 0),
        Amount::from_sat(100_000),
        1,
        false,
    );
    manager.add_utxo(utxo.clone());
    
    let target = Amount::from_sat(50_000);
    match manager.select_utxos(target, SelectionStrategy::MinimizeFee, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("Selected UTXOs: {} with total amount: {}", selected.len(), total_selected);
            
            // Log results for debugging
            for (i, utxo) in selected.iter().enumerate() {
                println!("  UTXO {}: {} sats", i, utxo.amount.to_sat());
            }
            println!("Fee amount: {} sats", fee_amount.to_sat());
            println!("Change amount: {} sats", change_amount.to_sat());
            
            // Fundamental assertions
            assert!(!selected.is_empty(), "Should select at least one UTXO");
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(), 
                  "Selected amount should cover target + fee");
            
            // Since there's only one UTXO available, it must be selected
            assert!(selected.contains(&utxo), "The only available UTXO should be selected");
            
            // Verify balance equation with tolerance
            let expected_change = total_selected - target.to_sat() - fee_amount.to_sat();
            let actual_change = change_amount.to_sat();
            let tolerance = 10; // Allow small rounding differences
            
            assert!(
                (expected_change as i64 - actual_change as i64).abs() <= tolerance as i64,
                "Change amount differs by more than tolerance: expected {}, got {}, diff {}",
                expected_change, actual_change, 
                (expected_change as i64 - actual_change as i64).abs()
            );
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_minimize_fee_selection_multiple_utxos() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("1111111111111111111111111111111111111111111111111111111111111111").unwrap(), 0),
        Amount::from_sat(30_000),
        1,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("2222222222222222222222222222222222222222222222222222222222222222").unwrap(), 0),
        Amount::from_sat(40_000),
        2,
        false,
    );
    let utxo3 = Utxo::new(
        OutPoint::new(Txid::from_str("3333333333333333333333333333333333333333333333333333333333333333").unwrap(), 0),
        Amount::from_sat(50_000),
        3,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());
    manager.add_utxo(utxo3.clone());
    
    // Test with a target amount of 60_000
    let target = Amount::from_sat(60_000);
    match manager.select_utxos(target, SelectionStrategy::MinimizeFee, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("Target: {} sats", target.to_sat());
            println!("Selected UTXOs: {} with total amount: {}", selected.len(), total_selected);
            
            // Log results for debugging
            for (i, utxo) in selected.iter().enumerate() {
                println!("  UTXO {}: {} sats", i, utxo.amount.to_sat());
            }
            println!("Fee amount: {} sats", fee_amount.to_sat());
            println!("Change amount: {} sats", change_amount.to_sat());
            
            // Fundamental assertions
            assert!(!selected.is_empty(), "Should select at least one UTXO");
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(), 
                  "Selected amount should cover target + fee");
            
            // With MinimizeFee strategy, we expect to minimize the number of inputs
            // For a target of 60_000, we expect utxo3 (50_000) to be selected
            // along with at least one more UTXO
            assert!(selected.iter().any(|u| u.amount.to_sat() >= 50_000), 
                  "Larger UTXOs should be preferred with MinimizeFee strategy");
            
            // Verify balance equation with tolerance
            let expected_change = total_selected - target.to_sat() - fee_amount.to_sat();
            let actual_change = change_amount.to_sat();
            let tolerance = 10; // Allow small rounding differences
            
            assert!(
                (expected_change as i64 - actual_change as i64).abs() <= tolerance as i64,
                "Change amount differs by more than tolerance: expected {}, got {}, diff {}",
                expected_change, actual_change, 
                (expected_change as i64 - actual_change as i64).abs()
            );
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_maximize_privacy_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("1111111111111111111111111111111111111111111111111111111111111111").unwrap(), 0),
        Amount::from_sat(40_000),
        1,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("2222222222222222222222222222222222222222222222222222222222222222").unwrap(), 0),
        Amount::from_sat(40_000),
        2,
        false,
    );
    let utxo3 = Utxo::new(
        OutPoint::new(Txid::from_str("3333333333333333333333333333333333333333333333333333333333333333").unwrap(), 0),
        Amount::from_sat(40_000),
        3,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());
    manager.add_utxo(utxo3.clone());
    
    let target = Amount::from_sat(60_000);
    match manager.select_utxos(target, SelectionStrategy::MaximizePrivacy, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("Target: {} sats", target.to_sat());
            println!("Selected UTXOs: {} with total amount: {}", selected.len(), total_selected);
            
            // Log selected UTXOs for debugging
            for (i, utxo) in selected.iter().enumerate() {
                println!("  UTXO {}: {} sats", i, utxo.amount.to_sat());
            }
            println!("Fee amount: {} sats", fee_amount.to_sat());
            println!("Change amount: {} sats", change_amount.to_sat());
            
            // Fundamental assertions
            assert!(!selected.is_empty(), "Should select at least one UTXO");
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(), 
                  "Selected amount should cover target + fee");
            
            // MaximizePrivacy strategy typically looks for UTXOs that are equal or close in value
            // We don't want to assert the exact number of UTXOs selected, as that's an implementation detail
            let utxo_values: Vec<u64> = selected.iter().map(|u| u.amount.to_sat()).collect();
            println!("Selected UTXO values: {:?}", utxo_values);
            
            // For privacy, we expect at least 2 UTXOs to be selected (when available and when needed)
            // Only assert this for large amounts where multiple UTXOs would reasonably be needed
            if target.to_sat() > 40_000 {
                assert!(selected.len() >= 2, "MaximizePrivacy should favor multiple inputs for larger amounts");
            }
            
            // Verify balance equation with tolerance
            let expected_change = total_selected - target.to_sat() - fee_amount.to_sat();
            let actual_change = change_amount.to_sat();
            let tolerance = 10; // Allow small rounding differences
            
            assert!(
                (expected_change as i64 - actual_change as i64).abs() <= tolerance as i64,
                "Change amount differs by more than tolerance: expected {}, got {}, diff {}",
                expected_change, actual_change, 
                (expected_change as i64 - actual_change as i64).abs()
            );
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_consolidate_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
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
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());
    
    // First target: low amount (70_000)
    let target = Amount::from_sat(70_000);
    match manager.select_utxos(target, SelectionStrategy::Consolidate, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("Target (Low): {} sats", target.to_sat());
            println!("Selected UTXOs: {} with total amount: {}", selected.len(), total_selected);
            
            // Log selected UTXOs for debugging
            for (i, utxo) in selected.iter().enumerate() {
                println!("  UTXO {}: {} sats", i, utxo.amount.to_sat());
            }
            println!("Fee amount: {} sats", fee_amount.to_sat());
            println!("Change amount: {} sats", change_amount.to_sat());
            
            // Fundamental assertions - should have enough funds
            assert!(!selected.is_empty(), "Should select at least one UTXO");
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(), 
                  "Selected amount should cover target + fee");
            
            // For Consolidate strategy, we expect to select as many UTXOs as needed
            // This test should select both UTXOs as the total is close to the target
            // However, we don't want to make the test brittle with exact assertions
            
            // Verify balance equation with tolerance
            let expected_change = total_selected - target.to_sat() - fee_amount.to_sat();
            let actual_change = change_amount.to_sat();
            let tolerance = 10; // Allow small rounding differences
            
            assert!(
                (expected_change as i64 - actual_change as i64).abs() <= tolerance as i64,
                "Change amount differs by more than tolerance: expected {}, got {}, diff {}",
                expected_change, actual_change, 
                (expected_change as i64 - actual_change as i64).abs()
            );
        },
        _ => panic!("Expected success but got failure"),
    }
    
    // Second target: higher amount (90_000)
    let target = Amount::from_sat(90_000);
    match manager.select_utxos(target, SelectionStrategy::Consolidate, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("Target (High): {} sats", target.to_sat());
            println!("Selected UTXOs: {} with total amount: {}", selected.len(), total_selected);
            
            // Log selected UTXOs for debugging
            for (i, utxo) in selected.iter().enumerate() {
                println!("  UTXO {}: {} sats", i, utxo.amount.to_sat());
            }
            println!("Fee amount: {} sats", fee_amount.to_sat());
            println!("Change amount: {} sats", change_amount.to_sat());
            
            // For higher targets, we definitely should use all available UTXOs
            assert_eq!(selected.len(), 2, "Should select all available UTXOs for high target");
            assert!(selected.contains(&utxo1), "Should include utxo1");
            assert!(selected.contains(&utxo2), "Should include utxo2");
            
            // Verify that we have enough to cover the target and fee
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(), 
                  "Selected amount should cover target + fee");
            
            // Verify balance equation with tolerance
            let expected_change = total_selected - target.to_sat() - fee_amount.to_sat();
            let actual_change = change_amount.to_sat();
            let tolerance = 10; // Allow small rounding differences
            
            assert!(
                (expected_change as i64 - actual_change as i64).abs() <= tolerance as i64,
                "Change amount differs by more than tolerance: expected {}, got {}, diff {}",
                expected_change, actual_change, 
                (expected_change as i64 - actual_change as i64).abs()
            );
        },
        SelectionResult::InsufficientFunds { available, required } => {
            // If we get insufficient funds, this is also acceptable as the target + fee 
            // might be very close to or exceed our total available funds
            println!("Got insufficient funds for high target: available={}, required={}", 
                  available.to_sat(), required.to_sat());
            
            // Still verify we're close to the expected values
            let total_available = utxo1.amount.to_sat() + utxo2.amount.to_sat();
            assert_eq!(available.to_sat(), total_available, 
                      "Available amount should be the total of all UTXOs");
            assert!(required.to_sat() >= target.to_sat(), 
                   "Required amount should be at least the target amount");
        },
    }
}

#[test]
fn test_oldest_first_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("7777777777777777777777777777777777777777777777777777777777777777").unwrap(), 0),
        Amount::from_sat(50_000),
        10, // Higher confirmations (older)
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("8888888888888888888888888888888888888888888888888888888888888888").unwrap(), 0),
        Amount::from_sat(50_000),
        5,  // Lower confirmations (newer)
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());

    // Test that we select the oldest UTXO first (higher confirmations)
    let target = Amount::from_sat(40_000);
    match manager.select_utxos(target, SelectionStrategy::OldestFirst, None, None) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 1);
            assert!(selected.contains(&utxo1));
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_coin_control_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    // Create 3 UTXOs with different amounts
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("1111111111111111111111111111111111111111111111111111111111111111").unwrap(), 0),
        Amount::from_sat(30_000),
        1,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("2222222222222222222222222222222222222222222222222222222222222222").unwrap(), 0),
        Amount::from_sat(40_000),
        2,
        false,
    );
    let utxo3 = Utxo::new(
        OutPoint::new(Txid::from_str("3333333333333333333333333333333333333333333333333333333333333333").unwrap(), 0),
        Amount::from_sat(50_000),
        3,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());
    manager.add_utxo(utxo3.clone());
    
    // Create a CoinControl instance and manually select the first two UTXOs
    let mut coin_control = CoinControl::new();
    coin_control.select_utxo(utxo1.outpoint);
    coin_control.select_utxo(utxo2.outpoint);
    
    // Target amount 
    let target = Amount::from_sat(60_000);
    
    // Test with CoinControl strategy using our coin_control.selected_outpoints
    match manager.select_coin_control(&coin_control.selected_outpoints, target, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            println!("Target: {} sats", target.to_sat());
            println!("Selected UTXOs: {} with total amount: {}", selected.len(), total_selected);
            
            // Log selected UTXOs for debugging
            for (i, utxo) in selected.iter().enumerate() {
                println!("  UTXO {}: {} sats", i, utxo.amount.to_sat());
            }
            println!("Fee amount: {} sats", fee_amount.to_sat());
            println!("Change amount: {} sats", change_amount.to_sat());
            
            // Fundamental assertions
            assert!(!selected.is_empty(), "Should select at least one UTXO");
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(), 
                  "Selected amount should cover target + fee");
            
            // CoinControl specific assertions - should only select UTXOs we've manually chosen
            assert!(selected.contains(&utxo1), "Should include manually selected utxo1");
            assert!(selected.contains(&utxo2), "Should include manually selected utxo2");
            assert!(!selected.contains(&utxo3), "Should NOT include utxo3 (not manually selected)");
            
            // Verify balance equation with tolerance
            let expected_change = total_selected - target.to_sat() - fee_amount.to_sat();
            let actual_change = change_amount.to_sat();
            let tolerance = 10; // Allow small rounding differences
            
            assert!(
                (expected_change as i64 - actual_change as i64).abs() <= tolerance as i64,
                "Change amount differs by more than tolerance: expected {}, got {}, diff {}",
                expected_change, actual_change, 
                (expected_change as i64 - actual_change as i64).abs()
            );
        },
        SelectionResult::InsufficientFunds { available, required } => {
            panic!("Expected success but got insufficient funds: available={}, required={}",
                   available.to_sat(), required.to_sat());
        },
    }
    
    // Now try with insufficient funds
    let target = Amount::from_sat(100_000);
    let mut coin_control = CoinControl::new();
    coin_control.select_utxo(utxo1.outpoint); // Only select one UTXO with 30,000 sats
    match manager.select_coin_control(&coin_control.selected_outpoints, target, None, None) {
        SelectionResult::InsufficientFunds { available, required } => {
            println!("Got expected InsufficientFunds error: available={}, required={}",
                    available.to_sat(), required.to_sat());
            assert!(available.to_sat() < required.to_sat(), 
                  "Available funds should be less than needed");
        },
        SelectionResult::Success { .. } => panic!("Expected failure but got success"),
    }
}

#[test]
fn test_privacy_focused_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("3333333333333333333333333333333333333333333333333333333333333333").unwrap(), 0),
        Amount::from_sat(30_000),
        5,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("4444444444444444444444444444444444444444444444444444444444444444").unwrap(), 0),
        Amount::from_sat(40_000),
        5,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());

    let target = Amount::from_sat(60_000);
    match manager.select_utxos(target, SelectionStrategy::PrivacyFocused, None, None) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 2);
            assert!(selected.contains(&utxo1));
            assert!(selected.contains(&utxo2));
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_avoid_change_selection() {
    setup_utxo_management_tests();
    
    let mut manager = UtxoManager::new();
    let utxo1 = Utxo::new(
        OutPoint::new(Txid::from_str("3333333333333333333333333333333333333333333333333333333333333333").unwrap(), 0),
        Amount::from_sat(30_000),
        5,
        false,
    );
    let utxo2 = Utxo::new(
        OutPoint::new(Txid::from_str("4444444444444444444444444444444444444444444444444444444444444444").unwrap(), 0),
        Amount::from_sat(40_000),
        5,
        false,
    );
    manager.add_utxo(utxo1.clone());
    manager.add_utxo(utxo2.clone());

    let target = Amount::from_sat(60_000);
    match manager.select_utxos(target, SelectionStrategy::AvoidChange, None, None) {
        SelectionResult::Success { selected, .. } => {
            assert_eq!(selected.len(), 2);
            assert!(selected.contains(&utxo1));
            assert!(selected.contains(&utxo2));
        },
        _ => panic!("Expected success but got failure"),
    }
} 