use bitvault_common::utxo_selection::types::{Utxo, SelectionResult, SelectionStrategy};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::events::UtxoEventBus;
use bitvault_common::utxo_selection::strategies::minimize_change::MinimizeChangeStrategy;
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

/// Create a test selector with timeout for tests
fn create_test_selector() -> UtxoSelector {
    // For tests, we want a consistent timeout
    UtxoSelector::with_minimize_change_strategy(
        MinimizeChangeStrategy::with_timeout(500) // 500ms timeout for test stability
    )
}

#[test]
fn test_branch_and_bound_basic() {
    setup();
    
    let utxos = create_test_utxos();
    let _fee_rate = dec!(1.0); // 1 sat/vByte
    
    // Test with a simple target
    let target = Amount::from_sat(30_000);
    
    // Create a selector with accuracy prioritization
    let selector = create_test_selector();
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected amount
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            // Basic assertions that must hold true
            assert!(total_selected >= target.to_sat(),
                "Total selected should cover target");
            
            // Verify fee calculation
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(),
                "Total selected should cover target + fee");
            
            // Verify change calculation
            let expected_change = total_selected.saturating_sub(target.to_sat() + fee_amount.to_sat());
            assert_eq!(change_amount.to_sat(), expected_change,
                "Change amount should be correctly calculated");
            
            println!("Branch and Bound selection successful: selected={}, target={}, fee={}, change={}",
                total_selected, target.to_sat(), fee_amount.to_sat(), change_amount.to_sat());
        },
        _ => panic!("Branch and Bound selection failed"),
    }
}

#[test]
fn test_minimize_waste_basic() {
    setup();
    
    let utxos = create_test_utxos();
    let fee_rate = dec!(1.0); // 1 sat/vByte
    
    // Test with a simple target
    let target = Amount::from_sat(30_000);
    
    // Create a selector with the fee rate
    let selector = UtxoSelector::with_fee_rate(1.0);
    
    match selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None) {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Calculate total selected amount
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            // Basic assertions that must hold true
            assert!(total_selected >= target.to_sat(),
                "Total selected should cover target");
            
            // Verify fee calculation
            assert!(total_selected >= target.to_sat() + fee_amount.to_sat(),
                "Total selected should cover target + fee");
            
            // Verify change calculation
            let expected_change = total_selected.saturating_sub(target.to_sat() + fee_amount.to_sat());
            assert_eq!(change_amount.to_sat(), expected_change,
                "Change amount should be correctly calculated");
            
            println!("Minimize Waste selection successful: selected={}, target={}, fee={}, change={}",
                total_selected, target.to_sat(), fee_amount.to_sat(), change_amount.to_sat());
        },
        _ => panic!("Minimize Waste selection failed"),
    }
} 