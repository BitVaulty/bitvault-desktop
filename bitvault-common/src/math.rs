//! Mathematical utility functions for Bitcoin calculations
//!
//! This module provides minimal utilities for Bitcoin calculations that
//! are not directly provided by BDK. For amount conversions and calculations,
//! use `bitcoin::Amount` directly.
//!
//! This approach follows BDK best practices by using `bitcoin::Amount`
//! for amount handling and conversions, ensuring consistent behavior
//! with the broader Bitcoin ecosystem.
//!
//! Example:
//! ```
//! use bitcoin::Amount;
//!
//! // Convert BTC to satoshis
//! let sats = Amount::from_btc(1.5).unwrap().to_sat();
//!
//! // Convert satoshis to BTC
//! let btc = Amount::from_sat(150_000_000).to_btc();
//! ```

use crate::types::DUST_THRESHOLD;
use bdk::FeeRate;

/// Determines if an amount is considered "dust" (too small to be economically viable)
/// Bitcoin has a standard minimum output value to prevent spam transactions.
///
/// # Arguments
/// * `amount_sats` - The amount in satoshis to check
///
/// # Returns
/// `true` if the amount is considered dust, `false` otherwise
pub fn is_dust_amount(amount_sats: u64) -> bool {
    amount_sats < DUST_THRESHOLD
}

/// Calculates the minimum change amount that would be economical to create
/// based on the fee rate and output size.
///
/// # Arguments
/// * `fee_rate` - The fee rate to use for calculation
/// * `output_size` - The size of the output in bytes (typically 32-34 bytes)
///
/// # Returns
/// The minimum amount in satoshis that would be economical as change
pub fn min_economical_change(fee_rate: FeeRate, output_size: usize) -> u64 {
    // Calculate the fee cost of adding this output
    let output_fee = (output_size as f32 * fee_rate.as_sat_per_vb()).ceil() as u64;

    // The minimum economical amount is the output fee plus the dust threshold
    // This ensures that the output is worth more than it costs to spend it
    DUST_THRESHOLD + output_fee
}
