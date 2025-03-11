//! Property-based tests for the math module
//!
//! These tests use quickcheck to verify mathematical properties
//! of the Bitcoin-related calculations in the math module.

use bitcoin::Amount;
use bitvault_common::math;
use bitvault_common::types::*;
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;

// Helper to generate valid Bitcoin amounts
#[derive(Clone, Debug)]
struct ValidBitcoinAmount(f64);

impl Arbitrary for ValidBitcoinAmount {
    fn arbitrary(g: &mut Gen) -> Self {
        // Generate a value between 0 and 21 million BTC
        let btc = f64::arbitrary(g) % 21_000_000.0;
        // Ensure it's positive and finite
        let btc = if btc < 0.0 { -btc } else { btc };
        let btc = if !btc.is_finite() { 1.0 } else { btc };
        
        ValidBitcoinAmount(btc)
    }
}

// Helper to generate valid satoshi amounts
#[derive(Clone, Debug)]
struct ValidSatoshiAmount(u64);

impl Arbitrary for ValidSatoshiAmount {
    fn arbitrary(g: &mut Gen) -> Self {
        // Generate a value between 0 and MAX_BITCOIN_SUPPLY
        let sats = u64::arbitrary(g) % MAX_BITCOIN_SUPPLY;
        ValidSatoshiAmount(sats)
    }
}

#[quickcheck]
fn btc_to_sats_and_back_is_identity(amount: ValidBitcoinAmount) -> TestResult {
    let btc = amount.0;
    
    // Skip very small values where rounding might cause issues
    if btc < 0.00000001 {
        return TestResult::discard();
    }
    
    // Use bitcoin::Amount for conversion
    match Amount::from_btc(btc) {
        Ok(amount) => {
            let sats = amount.to_sat();
            let btc_again = Amount::from_sat(sats).to_btc();
            
            // Due to floating point precision, we check if the values are close enough
            let diff = (btc - btc_again).abs();
            TestResult::from_bool(diff < 0.00000001)
        },
        Err(_) => TestResult::discard(), // Invalid input
    }
}

#[quickcheck]
fn sats_to_btc_to_sats_is_identity(amount: ValidSatoshiAmount) -> bool {
    let sats = amount.0;
    
    // Use bitcoin::Amount for conversion
    let btc = Amount::from_sat(sats).to_btc();
    let sats_again = match Amount::from_btc(btc) {
        Ok(amount) => amount.to_sat(),
        Err(_) => return false, // This should never happen for valid inputs
    };
    
    sats == sats_again
}

#[quickcheck]
fn dust_threshold_is_consistent(amount: u64) -> bool {
    // Check if our dust threshold detection is consistent
    if amount < DUST_THRESHOLD {
        math::is_dust_amount(amount)
    } else {
        !math::is_dust_amount(amount)
    }
}

#[quickcheck]
fn min_economical_change_exceeds_dust(fee_rate_sat_per_vb: u8, output_size: u8) -> bool {
    // Ensure reasonable test values
    let fee_rate = bdk::FeeRate::from_sat_per_vb(fee_rate_sat_per_vb as f32);
    let output_size = output_size as usize + 30; // Ensure output size is reasonable (at least 30 bytes)
    
    // Min economical change should always exceed dust threshold
    let min_change = math::min_economical_change(fee_rate, output_size);
    min_change >= DUST_THRESHOLD
} 