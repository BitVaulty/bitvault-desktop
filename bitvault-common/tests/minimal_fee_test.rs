use std::sync::Once;
use bdk::bitcoin::Network;
use bitvault_common::types::FeePriority;
use bitvault_common::fee_estimation::FeeRecommendations;
use rust_decimal_macros::dec;

// Static initialization for test module
static INIT_LOGGER: Once = Once::new();

fn setup() {
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });
}

#[test]
fn test_minimal_fee_recommendations() {
    setup();
    
    // Create fee recommendations
    let mut recommendations = FeeRecommendations::new(Network::Bitcoin);
    
    // Set up sample fee rates
    let high_fee = dec!(5.0);
    let medium_fee = dec!(3.0);
    let low_fee = dec!(1.5);
    
    // Populate maps with values
    recommendations.by_block_target.insert(1, high_fee);
    recommendations.by_block_target.insert(3, medium_fee);
    recommendations.by_block_target.insert(6, low_fee);
    
    // Store values in the priority map
    recommendations.by_priority.insert(FeePriority::High, high_fee);
    recommendations.by_priority.insert(FeePriority::Medium, medium_fee);
    recommendations.by_priority.insert(FeePriority::Low, low_fee);
    
    // Test priority getters
    let high_priority_fee = recommendations.get_fee_for_priority(FeePriority::High);
    assert_eq!(high_priority_fee, high_fee);
    
    let medium_priority_fee = recommendations.get_fee_for_priority(FeePriority::Medium);
    assert_eq!(medium_priority_fee, medium_fee);
    
    let low_priority_fee = recommendations.get_fee_for_priority(FeePriority::Low);
    assert_eq!(low_priority_fee, low_fee);
} 