use bdk::FeeRate;
use bitcoin::Amount;
use bitvault_common::math;
use bitvault_common::types::DUST_THRESHOLD;

#[test]
fn test_is_dust_amount() {
    // Test dust threshold
    assert!(math::is_dust_amount(DUST_THRESHOLD - 1));
    assert!(!math::is_dust_amount(DUST_THRESHOLD));
    assert!(!math::is_dust_amount(DUST_THRESHOLD + 1));

    // Test edge cases
    assert!(math::is_dust_amount(0));
    assert!(!math::is_dust_amount(100_000));
}

#[test]
fn test_min_economical_change() {
    // For a very low fee rate
    let low_fee_rate = FeeRate::from_sat_per_vb(1.0);
    let min_change_low = math::min_economical_change(low_fee_rate, 34); // P2WPKH output
    assert!(min_change_low > DUST_THRESHOLD); // Should be greater than dust

    // For a high fee rate
    let high_fee_rate = FeeRate::from_sat_per_vb(20.0);
    let min_change_high = math::min_economical_change(high_fee_rate, 34);
    assert!(min_change_high > min_change_low); // Higher fee rate means higher min change

    // Test with different output sizes
    let p2pkh_size = 34;
    let p2sh_size = 32;
    let min_change_p2pkh = math::min_economical_change(high_fee_rate, p2pkh_size);
    let min_change_p2sh = math::min_economical_change(high_fee_rate, p2sh_size);
    assert!(min_change_p2pkh > min_change_p2sh); // Larger output size should need more min change
}

#[test]
fn test_bitcoin_amount_usage() {
    // Demonstrate Bitcoin amount conversion
    let btc_amount = 1.5;
    let satoshi_amount = 150_000_000;

    // BTC to satoshis
    let sats = Amount::from_btc(btc_amount).unwrap().to_sat();
    assert_eq!(sats, satoshi_amount);

    // Satoshis to BTC
    let btc = Amount::from_sat(satoshi_amount).to_btc();
    assert_eq!(btc, btc_amount);

    // Formatting
    let amount = Amount::from_sat(satoshi_amount);
    assert_eq!(format!("{}", amount), "1.5 BTC");
}

#[test]
fn test_transaction_fee_calculation() {
    // Test simple fee calculation cases
    
    // For a 250 byte transaction at 1 sat/byte
    let tx_size = 250;
    let fee_rate = 1.0;
    let fee = math::calculate_fee(tx_size, fee_rate);
    assert_eq!(fee, Amount::from_sat(250));
    
    // Higher fee rate
    let fee_rate = 5.0;
    let fee = math::calculate_fee(tx_size, fee_rate);
    assert_eq!(fee, Amount::from_sat(1250));
    
    // Test with floating point fee rate
    let fee_rate = 2.5;
    let fee = math::calculate_fee(tx_size, fee_rate);
    assert_eq!(fee, Amount::from_sat(625));
    
    // Test small transaction
    let tx_size = 10;
    let fee_rate = 1.0;
    let fee = math::calculate_fee(tx_size, fee_rate);
    assert_eq!(fee, Amount::from_sat(10));
}

#[test]
fn test_transaction_size_estimation() {
    // Test basic transaction size estimation
    
    // Simple 1-input, 1-output transaction
    let size_1_1 = math::estimate_tx_size(1, 1);
    // Should be roughly 10 (overhead) + 68 (input) + 33 (output) = 111 bytes
    assert_eq!(size_1_1, 111);
    
    // 2-input, 1-output transaction
    let size_2_1 = math::estimate_tx_size(2, 1);
    // Should add one more input size (68 bytes)
    assert_eq!(size_2_1, 179);
    
    // Verify the size increases correctly with additional inputs/outputs
    // 3 inputs: 3 * 68 = 204, 2 outputs: 2 * 33 = 66, overhead: 10
    // Total: 204 + 66 + 10 = 280
    assert_eq!(math::estimate_tx_size(3, 2), 280);
    
    // Verify that the detailed size estimation works correctly
    let input_types = ["p2wpkh", "p2pkh"]; 
    let output_types = ["p2wpkh"];
    let detailed_size = math::estimate_tx_size_detailed(&input_types, &output_types);
    
    // Should be 10 (overhead) + 68 (p2wpkh input) + 148 (p2pkh input) + 31 (p2wpkh output) = 257
    assert_eq!(detailed_size, 257);
    
    // Test that different script types return different sizes
    assert!(math::get_input_size("p2pkh") > math::get_input_size("p2wpkh"));
    assert!(math::get_output_size("p2wsh") > math::get_output_size("p2wpkh"));
}
