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
use bitcoin::Amount;

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

/// Calculates the fee for a transaction based on size and fee rate
///
/// # Arguments
/// * `tx_size` - The size of the transaction in bytes
/// * `fee_rate` - The fee rate in satoshis per vbyte
///
/// # Returns
/// The calculated fee as a bitcoin Amount
pub fn calculate_fee(tx_size: usize, fee_rate: f32) -> Amount {
    let fee_sats = (tx_size as f32 * fee_rate).ceil() as u64;
    Amount::from_sat(fee_sats)
}

/// Estimates the size of a standard transaction based on the number of inputs and outputs
///
/// This provides a simplistic estimation of transaction size, not accounting for
/// different script types. For more accurate size estimation, use BDK's transaction
/// building and then check the actual size.
///
/// # Arguments
/// * `inputs` - Number of inputs
/// * `outputs` - Number of outputs
///
/// # Returns
/// The estimated transaction size in bytes
pub fn estimate_tx_size(inputs: usize, outputs: usize) -> usize {
    // These values are approximate for P2WPKH 
    // For more complex script types, the actual size will differ
    const TX_OVERHEAD: usize = 10; // Fixed transaction overhead
    const INPUT_SIZE: usize = 68;  // Approx size of a P2WPKH input
    const OUTPUT_SIZE: usize = 33; // Approx size of a P2WPKH output

    TX_OVERHEAD + (inputs * INPUT_SIZE) + (outputs * OUTPUT_SIZE)
}

/// Gets approximate input size for different script types
///
/// # Arguments
/// * `script_type` - The type of script as a string: "p2pkh", "p2wpkh", "p2sh", "p2wsh", etc.
///
/// # Returns
/// The typical size in bytes for an input of the given script type
pub fn get_input_size(script_type: &str) -> usize {
    match script_type.to_lowercase().as_str() {
        "p2pkh" => 148,   // Legacy P2PKH
        "p2wpkh" => 68,   // Native SegWit
        "p2sh-p2wpkh" => 91, // Nested SegWit
        "p2wsh" => 104,   // Native SegWit multisig (depends on script size)
        "p2tr" => 58,     // Taproot single-sig
        // Default to P2WPKH as a fallback
        _ => 68,
    }
}

/// Gets approximate output size for different script types
///
/// # Arguments
/// * `script_type` - The type of script as a string: "p2pkh", "p2wpkh", "p2sh", "p2wsh", etc.
///
/// # Returns
/// The typical size in bytes for an output of the given script type
pub fn get_output_size(script_type: &str) -> usize {
    match script_type.to_lowercase().as_str() {
        "p2pkh" => 34,    // Legacy P2PKH
        "p2wpkh" => 31,   // Native SegWit
        "p2sh" => 32,     // P2SH
        "p2wsh" => 43,    // Native SegWit multisig
        "p2tr" => 43,     // Taproot
        // Default to P2WPKH as a fallback
        _ => 31,
    }
}

/// More accurate transaction size estimation based on specific input and output types
///
/// # Arguments
/// * `input_types` - Vector of script type strings for inputs
/// * `output_types` - Vector of script type strings for outputs
///
/// # Returns
/// The estimated transaction size in bytes
pub fn estimate_tx_size_detailed(input_types: &[&str], output_types: &[&str]) -> usize {
    const TX_OVERHEAD: usize = 10; // Fixed transaction overhead
    
    // Sum the sizes of all inputs
    let inputs_size: usize = input_types.iter()
        .map(|script_type| get_input_size(script_type))
        .sum();
    
    // Sum the sizes of all outputs
    let outputs_size: usize = output_types.iter()
        .map(|script_type| get_output_size(script_type))
        .sum();
    
    TX_OVERHEAD + inputs_size + outputs_size
}
