use bitvault_common::network_status::{
    CongestionLevel, MempoolStatus, MockNetworkStatusProvider, NetworkStatusProvider, TransactionConfirmationStatus,
};
use bitcoin::Network;

#[test]
fn test_network_status_retrieval() {
    let provider = MockNetworkStatusProvider::default();
    let network_status = provider.get_network_status().unwrap();
    
    assert_eq!(network_status.network, Network::Bitcoin);
    assert!(network_status.current_height > 0);
    assert!(network_status.connected);
}

#[test]
fn test_different_congestion_levels() {
    // Test low congestion - construct a new provider for each test
    let low_provider = MockNetworkStatusProvider::default()
        .with_congestion(CongestionLevel::Low);
    let low_status = low_provider.get_network_status().unwrap();
    assert_eq!(low_status.congestion, CongestionLevel::Low, "Expected Low congestion level");
    
    let low_mempool = low_provider.get_mempool_status().unwrap();
    // The mempool congestion is determined by fullness percentage
    // With our updated .with_congestion method, this should now be Low
    assert_eq!(low_mempool.determine_congestion_level(), CongestionLevel::Low, 
               "Mempool should report Low congestion");
    
    // Test high congestion using a completely new instance
    let high_provider = MockNetworkStatusProvider::default()
        .with_congestion(CongestionLevel::High);
    let high_status = high_provider.get_network_status().unwrap();
    assert_eq!(high_status.congestion, CongestionLevel::High, "Expected High congestion level");
    
    let high_mempool = high_provider.get_mempool_status().unwrap();
    assert_eq!(high_mempool.determine_congestion_level(), CongestionLevel::High,
               "Mempool should report High congestion");
    
    // Test severe congestion using another completely new instance
    let severe_provider = MockNetworkStatusProvider::default()
        .with_congestion(CongestionLevel::Severe);
    let severe_status = severe_provider.get_network_status().unwrap();
    assert_eq!(severe_status.congestion, CongestionLevel::Severe, "Expected Severe congestion level");
    
    let severe_mempool = severe_provider.get_mempool_status().unwrap();
    assert_eq!(severe_mempool.determine_congestion_level(), CongestionLevel::Severe,
               "Mempool should report Severe congestion");
    
    // Compare fee estimates between low and high congestion
    let low_fee = low_provider.get_recommended_fee_rate(1).unwrap();
    let high_fee = high_provider.get_recommended_fee_rate(1).unwrap();
    assert!(high_fee > low_fee, "High congestion fees should be higher than low congestion fees");
}

#[test]
fn test_transaction_confirmation_status() {
    let provider = MockNetworkStatusProvider::default();
    
    // Test unconfirmed transaction
    let unconfirmed = provider.get_tx_confirmation_status(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ).unwrap();
    assert_eq!(unconfirmed.confirmations, 0);
    assert!(!unconfirmed.is_confirmed());
    assert!(!unconfirmed.is_target_reached());
    
    // Test confirmed transaction - use a txid that will have confirmations
    // Last character '1' should give 1 confirmation based on the mock implementation
    let confirmed = provider.get_tx_confirmation_status(
        "1111111111111111111111111111111111111111111111111111111111111111"
    ).unwrap();
    
    // Verify it has at least 1 confirmation
    assert!(confirmed.confirmations > 0, "Transaction should have at least 1 confirmation");
    assert!(confirmed.is_confirmed(), "Transaction should be confirmed");
    
    // Test confirmation progress
    let half_confirmed = TransactionConfirmationStatus {
        txid: "test".to_string(),
        confirmations: 3,
        target_blocks: 6,
        fee_rate: 5.0,
        first_seen: 0,
        eta_seconds: None,
        is_rbf: false,
        parent_txids: Vec::new(),
    };
    
    assert_eq!(half_confirmed.confirmation_progress(), 50.0);
}

#[test]
fn test_mempool_congestion_determination() {
    let mut mempool = MempoolStatus::new();
    
    // Test various fullness levels with values clearly in each range
    mempool.fullness_percentage = 5.0;  // Well below the 15% threshold
    assert_eq!(mempool.determine_congestion_level(), CongestionLevel::Low);
    
    mempool.fullness_percentage = 25.0; // Between 15% and 40%
    assert_eq!(mempool.determine_congestion_level(), CongestionLevel::Moderate);
    
    mempool.fullness_percentage = 50.0; // Between 40% and 70%
    assert_eq!(mempool.determine_congestion_level(), CongestionLevel::High);
    
    mempool.fullness_percentage = 85.0; // Above 70%
    assert_eq!(mempool.determine_congestion_level(), CongestionLevel::Severe);
}

#[test]
fn test_likely_to_confirm() {
    let mut mempool = MempoolStatus::new();
    mempool.min_fee_rate = 5.0;
    
    // Test with various fee rates - using values clearly above thresholds
    assert!(mempool.likely_to_confirm(15.0, 1), "Fee rate 15.0 should be enough for next block (min*2=10.0)");
    assert!(mempool.likely_to_confirm(9.0, 2), "Fee rate 9.0 should be enough for 2-3 blocks (min*1.5=7.5)");
    assert!(mempool.likely_to_confirm(7.0, 4), "Fee rate 7.0 should be enough for 4-6 blocks (min*1.2=6.0)");
    assert!(!mempool.likely_to_confirm(9.0, 1), "Fee rate 9.0 should NOT be enough for next block (min*2=10.0)");
    
    // Edge cases
    assert!(mempool.likely_to_confirm(10.0, 1), "Fee rate exactly at threshold should confirm");
    assert!(mempool.likely_to_confirm(7.5, 3), "Fee rate exactly at threshold should confirm");
    assert!(mempool.likely_to_confirm(6.0, 6), "Fee rate exactly at threshold should confirm");
}

#[test]
fn test_fee_by_congestion() {
    // Test all congestion levels with separate, isolated provider instances
    let mut fee_rates = Vec::new();
    
    // Test Low congestion
    let low_provider = MockNetworkStatusProvider::default()
        .with_congestion(CongestionLevel::Low);
    let low_status = low_provider.get_network_status().unwrap();
    fee_rates.push(low_status.get_fee_by_congestion());
    
    // Test Moderate congestion
    let moderate_provider = MockNetworkStatusProvider::default()
        .with_congestion(CongestionLevel::Moderate);
    let moderate_status = moderate_provider.get_network_status().unwrap();
    fee_rates.push(moderate_status.get_fee_by_congestion());
    
    // Test High congestion
    let high_provider = MockNetworkStatusProvider::default()
        .with_congestion(CongestionLevel::High);
    let high_status = high_provider.get_network_status().unwrap();
    fee_rates.push(high_status.get_fee_by_congestion());
    
    // Test Severe congestion
    let severe_provider = MockNetworkStatusProvider::default()
        .with_congestion(CongestionLevel::Severe);
    let severe_status = severe_provider.get_network_status().unwrap();
    fee_rates.push(severe_status.get_fee_by_congestion());
    
    // Check that fee rates are positive and increase with congestion
    assert!(fee_rates.iter().all(|&fee| fee > 0.0), "All fee rates should be positive");
    
    // Fee rates should be non-decreasing with increasing congestion
    for i in 1..fee_rates.len() {
        assert!(fee_rates[i] >= fee_rates[i-1], 
            "Fee rates should increase with congestion: {:?}", fee_rates);
    }
} 