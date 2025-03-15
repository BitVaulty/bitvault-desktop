use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitcoin::{Amount, OutPoint, Txid, Network};
use std::str::FromStr;
use std::sync::Once;
use std::fs;
use std::env;
use std::path::PathBuf;
mod test_helpers;
use test_helpers::{assert_amounts_approximately_equal, assert_balance_equation, create_test_logger};
use bitvault_common::events::UtxoEventBus;

// Static initialization for test module
static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        // Create test_results directory if it doesn't exist
        let mut project_dir = env::current_dir().expect("Failed to get current directory");
        project_dir.push("test_results");
        fs::create_dir_all(&project_dir).expect("Failed to create test_results directory");
    });
}

fn create_test_utxos() -> Vec<Utxo> {
    vec![
        Utxo::new(
            OutPoint::new(Txid::from_str("7967a5185e907a25225574544c31f7b059c1a191d65b53dcc1554d339c4f9efc").unwrap(), 0),
            Amount::from_sat(10_000),
            1,
            false,
        ),
        Utxo::new(
            OutPoint::new(Txid::from_str("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap(), 0),
            Amount::from_sat(20_000),
            2,
            false,
        ),
        Utxo::new(
            OutPoint::new(Txid::from_str("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").unwrap(), 0),
            Amount::from_sat(100_000),
            5,
            false,
        ),
        Utxo::new(
            OutPoint::new(Txid::from_str("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap(), 0),
            Amount::from_sat(50_000),
            10,
            false,
        ),
    ]
}

#[test]
fn test_basic_selection() {
    setup();
    let mut logger = create_test_logger("test_basic_selection");
    
    let utxos = create_test_utxos();
    let target = Amount::from_sat(50_000);
    
    logger.log_amount("Target amount", target);
    
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeFee, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Log results
            logger.log_amount("Fee amount", fee_amount);
            logger.log_amount("Change amount", change_amount);
            
            // Basic assertions
            assert!(!selected.is_empty());
            assert!(fee_amount.to_sat() > 0);
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            let total_selected_amount = Amount::from_sat(total_selected);
            logger.log_amount("Total selected", total_selected_amount);
            
            // Verify we have enough to cover target + fee
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat());
            
            // Verify balance equation
            assert_balance_equation(
                total_selected_amount,
                target,
                fee_amount,
                change_amount,
                10, // 10 sats tolerance
            );
            
            logger.log_success();
        },
        _ => {
            logger.log_failure("Expected success but got failure");
            panic!("Expected success but got failure");
        },
    }
}

#[test]
fn test_coin_control() {
    setup();
    let mut logger = create_test_logger("test_coin_control");
    
    let utxos = create_test_utxos();
    let selected_utxos = vec![utxos[2].clone()]; // Select the 100,000 sat UTXO
    let target = Amount::from_sat(50_000);
    
    logger.log_amount("Target amount", target);
    logger.log_amount("Selected UTXO amount", selected_utxos[0].amount);
    
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_coin_control(&selected_utxos, target, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Log results
            logger.log_amount("Fee amount", fee_amount);
            logger.log_amount("Change amount", change_amount);
            
            // Basic assertions
            assert_eq!(selected.len(), 1);
            assert!(fee_amount.to_sat() > 0);
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            let total_selected_amount = Amount::from_sat(total_selected);
            logger.log_amount("Total selected", total_selected_amount);
            
            // Verify we have enough to cover target + fee
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat());
            
            // Verify balance equation
            assert_balance_equation(
                total_selected_amount,
                target,
                fee_amount,
                change_amount,
                10, // 10 sats tolerance
            );
            
            logger.log_success();
        },
        _ => {
            logger.log_failure("Expected success but got failure");
            panic!("Expected success but got failure");
        },
    }
}

#[test]
fn test_minimize_change() {
    setup();
    
    let utxos = create_test_utxos();
    let target = Amount::from_sat(25_000);
    
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Basic assertions
            assert!(!selected.is_empty());
            assert!(fee_amount.to_sat() > 0);
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            // Verify we have enough to cover target + fee
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat());
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_oldest_first() {
    setup();
    
    let utxos = create_test_utxos();
    let target = Amount::from_sat(25_000);
    
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::OldestFirst, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Basic assertions
            assert!(!selected.is_empty());
            assert!(fee_amount.to_sat() > 0);
            
            // Verify we selected the oldest UTXO first
            assert!(selected.iter().any(|u| u.confirmations == 10));
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_privacy_focused() {
    setup();
    
    let utxos = create_test_utxos();
    let target = Amount::from_sat(25_000);
    
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::PrivacyFocused, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Basic assertions
            assert!(!selected.is_empty());
            assert!(fee_amount.to_sat() > 0);
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            // Verify we have enough to cover target + fee
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat());
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_maximize_privacy() {
    setup();
    
    let utxos = create_test_utxos();
    let target = Amount::from_sat(25_000);
    
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::MaximizePrivacy, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Basic assertions
            assert!(!selected.is_empty());
            assert!(fee_amount.to_sat() > 0);
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            // Verify we have enough to cover target + fee
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat());
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_consolidate() {
    setup();
    
    let utxos = create_test_utxos();
    let target = Amount::from_sat(25_000);
    
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::Consolidate, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Basic assertions
            assert!(!selected.is_empty());
            assert!(fee_amount.to_sat() > 0);
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            // Verify we have enough to cover target + fee
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat());
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_avoid_change() {
    setup();
    let mut logger = create_test_logger("test_avoid_change");
    
    let utxos = create_test_utxos();
    let target = Amount::from_sat(25_000);
    
    logger.log_amount("Target amount", target);
    
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::AvoidChange, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Log results
            logger.log_amount("Fee amount", fee_amount);
            logger.log_amount("Change amount", change_amount);
            
            // Basic assertions
            assert!(!selected.is_empty());
            assert!(fee_amount.to_sat() > 0);
            
            // Calculate total selected
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            let total_selected_amount = Amount::from_sat(total_selected);
            logger.log_amount("Total selected", total_selected_amount);
            
            // Verify we have enough to cover target + fee
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat());
            
            // Verify balance equation
            assert_balance_equation(
                total_selected_amount,
                target,
                fee_amount,
                change_amount,
                10, // 10 sats tolerance
            );
            
            logger.log_success();
        },
        _ => {
            logger.log_failure("Expected success but got failure");
            panic!("Expected success but got failure");
        },
    }
} 