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
