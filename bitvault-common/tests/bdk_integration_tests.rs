//! Integration tests specifically focused on Bitcoin Development Kit (BDK) integration
//! This verifies that our type system correctly interacts with BDK types
//! and that our abstractions are properly aligned.

use bdk::FeeRate;
use bitcoin::{Address, Amount, Network};
use bitvault_common::{math::*, types::*};
use std::str::FromStr;

#[test]
fn test_address_info_with_bdk() {
    // Create a bitcoin address and wrap it in our AddressInfo
    let address_str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    let unchecked_address = Address::from_str(address_str).unwrap();
    // Convert to NetworkChecked
    let address = unchecked_address.require_network(Network::Bitcoin).unwrap();

    let address_info = AddressInfo::new(address.clone(), false);

    // Verify properties
    assert_eq!(address_info.address, address);
    assert_eq!(address_info.is_owned(), false);
    assert_eq!(address_info.label(), None);

    // Test display/string representation
    assert_eq!(address_info.to_string(), address.to_string());
}

#[test]
fn test_math_integration_with_bdk() {
    // Test dust threshold check
    let dust_amount = Amount::from_sat(DUST_THRESHOLD - 1);
    let valid_amount = Amount::from_sat(DUST_THRESHOLD + 100);

    assert!(is_dust_amount(dust_amount.to_sat()));
    assert!(!is_dust_amount(valid_amount.to_sat()));

    // Test fee calculations
    let fee_rate = FeeRate::from_sat_per_vb(5.0);
    let min_change = min_economical_change(fee_rate, 34); // P2WPKH output size

    // Min change should be greater than dust
    assert!(min_change > DUST_THRESHOLD);
}

#[test]
fn test_fee_estimates_serialization() {
    // Create fee estimates
    let fee_estimates = FeeEstimates {
        low: FeeRate::from_sat_per_vb(1.0),
        medium: FeeRate::from_sat_per_vb(5.0),
        high: FeeRate::from_sat_per_vb(10.0),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&fee_estimates).unwrap();

    // Deserialize back
    let deserialized: FeeEstimates = serde_json::from_str(&json).unwrap();

    // Check fee rates match
    assert_eq!(
        deserialized.get_fee_rate(FeePriority::Low).as_sat_per_vb(),
        fee_estimates.get_fee_rate(FeePriority::Low).as_sat_per_vb()
    );
    assert_eq!(
        deserialized
            .get_fee_rate(FeePriority::Medium)
            .as_sat_per_vb(),
        fee_estimates
            .get_fee_rate(FeePriority::Medium)
            .as_sat_per_vb()
    );
    assert_eq!(
        deserialized.get_fee_rate(FeePriority::High).as_sat_per_vb(),
        fee_estimates
            .get_fee_rate(FeePriority::High)
            .as_sat_per_vb()
    );
}

#[test]
fn test_wallet_settings_serialization() {
    // Create wallet settings with custom fee rate
    let settings = WalletSettings {
        network: Network::Bitcoin,
        use_tor: true,
        default_fee_level: FeePriority::Medium,
        custom_fee_rate: Some(FeeRate::from_sat_per_vb(7.5)),
        display_as_bitcoin: true,
        preferred_fiat: "USD".to_string(),
        show_fiat_amounts: true,
        custom_settings: std::collections::HashMap::new(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&settings).unwrap();

    // Deserialize back
    let deserialized: WalletSettings = serde_json::from_str(&json).unwrap();

    // Check values match
    assert_eq!(deserialized.network, settings.network);
    assert_eq!(deserialized.use_tor, settings.use_tor);
    assert_eq!(deserialized.default_fee_level, settings.default_fee_level);
    assert_eq!(
        deserialized.custom_fee_rate.unwrap().as_sat_per_vb(),
        settings.custom_fee_rate.unwrap().as_sat_per_vb()
    );
}
