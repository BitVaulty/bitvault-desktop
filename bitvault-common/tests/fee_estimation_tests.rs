use std::sync::Once;
use bdk::bitcoin::Network;
use bitvault_common::types::FeePriority;
use bitvault_common::network_status::{CongestionLevel, NetworkStatusProvider, MockNetworkStatusProvider};
use bitvault_common::fee_estimation::{
    adjust_fee_for_congestion, calculate_total_fee, FeeRecommendations,
    estimate_fee, create_recommendations, create_recommendations_from_provider,
    defaults::{get_default_fee_rate, min_reasonable_fee_rate, max_reasonable_fee_rate}
};
use rust_decimal_macros::dec;

// Static initialization for test module
static INIT_LOGGER: Once = Once::new();

fn setup() {
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });
}

#[test]
fn test_adjust_fee_for_congestion() {
    setup();
    
    let base_fee = dec!(2.0);
    
    // Test all congestion levels
    let low_fee = adjust_fee_for_congestion(base_fee, CongestionLevel::Low);
    let moderate_fee = adjust_fee_for_congestion(base_fee, CongestionLevel::Moderate);
    let high_fee = adjust_fee_for_congestion(base_fee, CongestionLevel::High);
    let severe_fee = adjust_fee_for_congestion(base_fee, CongestionLevel::Severe);
    
    // Verify expected adjustment factors
    assert_eq!(low_fee, dec!(2.0));
    assert_eq!(moderate_fee, dec!(2.4));
    assert_eq!(high_fee, dec!(3.0));
    assert_eq!(severe_fee, dec!(4.0));
    
    // Verify relative fee relationships
    assert!(severe_fee > high_fee);
    assert!(high_fee > moderate_fee);
    assert!(moderate_fee > low_fee);
}

#[test]
fn test_calculate_total_fee() {
    setup();
    
    // Test with whole numbers
    assert_eq!(calculate_total_fee(dec!(2.0), 250), 500);
    
    // Test with fractional rate
    assert_eq!(calculate_total_fee(dec!(1.5), 250), 375);
    
    // Test rounding up for fractional results
    assert_eq!(calculate_total_fee(dec!(1.0), 150), 150);
    assert_eq!(calculate_total_fee(dec!(1.1), 150), 165);
    
    // Test boundary conditions
    assert_eq!(calculate_total_fee(dec!(0.0), 100), 0, "Zero fee rate should result in zero fee");
    assert_eq!(calculate_total_fee(dec!(1.0), 0), 0, "Zero size should result in zero fee");
    
    // Test with very large values
    let large_size = 100_000; // 100 KB transaction (unrealistically large)
    let large_fee = calculate_total_fee(dec!(10.0), large_size);
    assert_eq!(large_fee, 1_000_000, "Large transaction should have correct fee");
    
    // Test with very small fractional fee rate
    let tiny_fee_rate = dec!(0.1);
    let tiny_fee = calculate_total_fee(tiny_fee_rate, 100);
    assert_eq!(tiny_fee, 10, "Small fee rate should calculate correctly");
}

#[test]
fn test_fee_recommendations() {
    setup();
    
    // Create fee recommendations
    let mut recommendations = FeeRecommendations::new(Network::Bitcoin);
    
    // Set up sample fee rates
    let high_fee = dec!(5.0);
    let medium_fee = dec!(3.0);
    let low_fee = dec!(1.0);
    
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
    
    // Test block target getters
    let target_1_fee = recommendations.get_fee_for_target(1);
    assert_eq!(target_1_fee, high_fee);
    
    let target_3_fee = recommendations.get_fee_for_target(3);
    assert_eq!(target_3_fee, medium_fee);
    
    let target_6_fee = recommendations.get_fee_for_target(6);
    assert_eq!(target_6_fee, low_fee);
    
    // Test interpolation for targets not directly stored
    let target_2_fee = recommendations.get_fee_for_target(2);
    assert!(target_2_fee >= medium_fee && target_2_fee <= high_fee,
           "Target 2 fee should be between medium and high");
    
    let target_10_fee = recommendations.get_fee_for_target(10);
    assert!(target_10_fee <= low_fee,
           "Target 10 fee should be below or equal to low fee");
}

#[test]
fn test_custom_fee_priority() {
    setup();
    
    // Test custom priority handling
    let recommendations = FeeRecommendations::new(Network::Bitcoin);
    
    // Create a custom priority with specific rate
    let custom_rate = 7.5;
    let custom_priority = FeePriority::Custom(custom_rate);
    
    // Get fee for custom priority
    let custom_fee = recommendations.get_fee_for_priority(custom_priority);
    
    // Verify it matches the custom rate
    assert_eq!(custom_fee, dec!(7.5));
    
    // Test with extreme values - should be capped
    let too_low_priority = FeePriority::Custom(0.0001);
    let too_low_fee = recommendations.get_fee_for_priority(too_low_priority);
    let min_fee = min_reasonable_fee_rate(Network::Bitcoin);
    assert_eq!(too_low_fee, min_fee, "Too low fee should be capped at minimum");
    
    let too_high_priority = FeePriority::Custom(5000.0);
    let too_high_fee = recommendations.get_fee_for_priority(too_high_priority);
    let max_fee = max_reasonable_fee_rate(Network::Bitcoin);
    assert_eq!(too_high_fee, max_fee, "Too high fee should be capped at maximum");
}

#[test]
fn test_network_specific_defaults() {
    setup();
    
    // Test mainnet defaults
    let mainnet_high = get_default_fee_rate(Network::Bitcoin, FeePriority::High);
    let mainnet_med = get_default_fee_rate(Network::Bitcoin, FeePriority::Medium);
    let mainnet_low = get_default_fee_rate(Network::Bitcoin, FeePriority::Low);
    
    assert!(mainnet_high > mainnet_med, "High priority should be higher than medium");
    assert!(mainnet_med > mainnet_low, "Medium priority should be higher than low");
    
    // Test testnet defaults
    let testnet_high = get_default_fee_rate(Network::Testnet, FeePriority::High);
    let testnet_med = get_default_fee_rate(Network::Testnet, FeePriority::Medium);
    let testnet_low = get_default_fee_rate(Network::Testnet, FeePriority::Low);
    
    assert!(testnet_high > testnet_med, "High priority should be higher than medium");
    assert!(testnet_med > testnet_low, "Medium priority should be higher than low");
    
    // Typically mainnet has higher fees than testnet
    assert!(mainnet_high >= testnet_high, "Mainnet high should be >= testnet high");
    
    // Test regtest defaults (should be lower)
    let regtest_high = get_default_fee_rate(Network::Regtest, FeePriority::High);
    assert!(regtest_high < mainnet_high, "Regtest high should be lower than mainnet high");
}

#[test]
fn test_min_max_reasonable_fees() {
    setup();
    
    // Test min/max bounds for different networks
    let mainnet_min = min_reasonable_fee_rate(Network::Bitcoin);
    let mainnet_max = max_reasonable_fee_rate(Network::Bitcoin);
    
    assert!(mainnet_min > dec!(0.0), "Minimum fee should be greater than zero");
    assert!(mainnet_max > mainnet_min, "Maximum fee should be greater than minimum");
    
    // Test that min/max values are different across networks
    let testnet_min = min_reasonable_fee_rate(Network::Testnet);
    let regtest_min = min_reasonable_fee_rate(Network::Regtest);
    
    assert!(regtest_min <= testnet_min, "Regtest min should be <= testnet min");
    assert!(testnet_min <= mainnet_min, "Testnet min should be <= mainnet min");
}

#[test]
fn test_create_recommendations() {
    setup();
    
    // Test creating recommendations with specified congestion level
    let recommendations = create_recommendations(
        Network::Bitcoin,
        CongestionLevel::High
    );
    
    // Verify the network and congestion level
    assert_eq!(recommendations.network, Network::Bitcoin);
    assert_eq!(recommendations.congestion, CongestionLevel::High);
    
    // Check that priority map is populated
    assert!(recommendations.by_priority.contains_key(&FeePriority::High));
    assert!(recommendations.by_priority.contains_key(&FeePriority::Medium));
    assert!(recommendations.by_priority.contains_key(&FeePriority::Low));
    
    // High congestion should have higher fees than normal
    let high_fee = recommendations.get_fee_for_priority(FeePriority::High);
    let normal_recommendations = create_recommendations(
        Network::Bitcoin,
        CongestionLevel::Low
    );
    let normal_high_fee = normal_recommendations.get_fee_for_priority(FeePriority::High);
    
    assert!(high_fee > normal_high_fee, "High congestion fees should be higher than normal");
}

#[test]
fn test_estimate_fee_safety() {
    setup();
    
    // Create a mock provider to test with
    let mut provider = MockNetworkStatusProvider::new(Network::Bitcoin);
    provider = provider.with_congestion(CongestionLevel::Low);
    
    // Test with different priority levels
    let high_priority = FeePriority::High;
    let medium_priority = FeePriority::Medium;
    let low_priority = FeePriority::Low;
    
    // Test high priority fee estimation
    let high_fee_result = estimate_fee(high_priority, &provider);
    assert!(high_fee_result.is_ok(), "High priority fee estimation should succeed");
    let high_fee = high_fee_result.unwrap();
    assert!(high_fee > dec!(0.0), "High priority fee should be positive");
    
    // Test medium priority fee estimation
    let medium_fee_result = estimate_fee(medium_priority, &provider);
    assert!(medium_fee_result.is_ok(), "Medium priority fee estimation should succeed");
    let medium_fee = medium_fee_result.unwrap();
    assert!(medium_fee > dec!(0.0), "Medium priority fee should be positive");
    
    // Test low priority fee estimation
    let low_fee_result = estimate_fee(low_priority, &provider);
    assert!(low_fee_result.is_ok(), "Low priority fee estimation should succeed");
    let low_fee = low_fee_result.unwrap();
    assert!(low_fee > dec!(0.0), "Low priority fee should be positive");
    
    // Test custom fee rate
    let custom_rate = 2.5;
    let custom_priority = FeePriority::Custom(custom_rate);
    let custom_fee_result = estimate_fee(custom_priority, &provider);
    assert!(custom_fee_result.is_ok(), "Custom priority fee estimation should succeed");
    let custom_fee = custom_fee_result.unwrap();
    assert_eq!(custom_fee, dec!(2.5), "Custom fee should match the specified rate");
    
    // Test with extreme custom values
    let too_low_priority = FeePriority::Custom(0.0001);
    let too_low_result = estimate_fee(too_low_priority, &provider);
    assert!(too_low_result.is_ok(), "Too low fee estimation should still succeed");
    
    let too_high_priority = FeePriority::Custom(5000.0);
    let too_high_result = estimate_fee(too_high_priority, &provider);
    assert!(too_high_result.is_ok(), "Too high fee estimation should still succeed");
    
    // Verify fee relationships
    assert!(high_fee >= medium_fee, "High priority fee should be >= medium priority fee");
    assert!(medium_fee >= low_fee, "Medium priority fee should be >= low priority fee");
}

#[test]
fn test_create_recommendations_from_provider() {
    setup();
    
    // Create a mock provider with known congestion
    let mut provider = MockNetworkStatusProvider::new(Network::Bitcoin);
    provider = provider.with_congestion(CongestionLevel::Severe);
    
    // Get network status to extract fee estimates
    let network_status = provider.get_network_status().expect("Failed to get network status");
    
    // Create recommendations from the provider's data
    let recommendations = create_recommendations_from_provider(
        Network::Bitcoin,
        network_status.fee_estimates,
        CongestionLevel::Severe
    );
    
    // Verify the recommendations match the provider's congestion level
    assert_eq!(recommendations.congestion, CongestionLevel::Severe);
    
    // Fees should be higher with severe congestion
    let high_fee = recommendations.get_fee_for_priority(FeePriority::High);
    let normal_recommendations = create_recommendations(
        Network::Bitcoin,
        CongestionLevel::Low
    );
    let normal_high_fee = normal_recommendations.get_fee_for_priority(FeePriority::High);
    
    assert!(high_fee > normal_high_fee, "Severe congestion fees should be higher than normal");
} 