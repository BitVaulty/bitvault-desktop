use bitvault_common::utxo_selection::{
    Utxo, SelectionResult
};
use bitvault_common::utxo_selection::advanced;
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use rust_decimal_macros::dec;
use std::sync::Once;
use env_logger;

// Initialize once for UTXO advanced tests
static INIT_LOGGER: Once = Once::new();

fn setup() {
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });
}

// Helper function to create simple test UTXOs
fn create_test_utxos() -> Vec<Utxo> {
    vec![
        // Small UTXO
        Utxo::new(
            OutPoint::new(
                Txid::from_str("7967a5185e907a25225574544c31f7b059c1a191d65b53dcc1554d339c4f9efc").unwrap(),
                0,
            ),
            Amount::from_sat(10_000),
            1,
            false,
        ),
        // Medium UTXO
        Utxo::new(
            OutPoint::new(
                Txid::from_str("9dcbf5a86b4e70be97fc5c953ad4111dfe0a94ea6768286e5efd6c35fd9ec9d1").unwrap(),
                1,
            ),
            Amount::from_sat(50_000),
            2,
            false,
        ),
        // Large UTXO
        Utxo::new(
            OutPoint::new(
                Txid::from_str("3d7c1421a4732a250ee59ce08b2ae34b5de8d3242e266a81a3d09887b8ca2e7c").unwrap(),
                0,
            ),
            Amount::from_sat(100_000),
            3,
            false,
        ),
    ]
}

#[test]
fn test_branch_and_bound_basic() {
    setup();
    
    let utxos = create_test_utxos();
    let fee_rate = dec!(1.0); // 1 sat/vByte
    
    // Test with a simple target
    let target = Amount::from_sat(30_000);
    
    match advanced::branch_and_bound(&utxos, target, fee_rate) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected amount
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            // Basic assertions that must hold true
            assert!(total_selected >= target.to_sat(), 
                   "Total selected must be at least the target amount");
            
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(),
                   "Total selected must cover target plus fee");
            
            // Calculate expected change
            let expected_change = total_selected.saturating_sub(target.to_sat() + fee_amount.to_sat());
            assert_eq!(change_amount.to_sat(), expected_change,
                      "Change amount should be correct");
            
            println!("Branch and bound test passed with {} UTXOs selected", selected.len());
            println!("Selected amount: {}, Target: {}, Fee: {}, Change: {}", 
                    total_selected, target.to_sat(), fee_amount.to_sat(), change_amount.to_sat());
        },
        _ => panic!("Expected success but got failure"),
    }
}

#[test]
fn test_minimize_waste_basic() {
    setup();
    
    let utxos = create_test_utxos();
    let fee_rate = dec!(1.0); // 1 sat/vByte
    
    // Test with a simple target
    let target = Amount::from_sat(30_000);
    
    match advanced::minimize_waste(&utxos, target, fee_rate) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected amount
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            // Basic assertions that must hold true
            assert!(total_selected >= target.to_sat(), 
                   "Total selected must be at least the target amount");
            
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(),
                   "Total selected must cover target plus fee");
            
            // Calculate expected change
            let expected_change = total_selected.saturating_sub(target.to_sat() + fee_amount.to_sat());
            assert_eq!(change_amount.to_sat(), expected_change,
                      "Change amount should be correct");
            
            println!("Minimize waste test passed with {} UTXOs selected", selected.len());
            println!("Selected amount: {}, Target: {}, Fee: {}, Change: {}", 
                    total_selected, target.to_sat(), fee_amount.to_sat(), change_amount.to_sat());
        },
        _ => panic!("Expected success but got failure"),
    }
} 