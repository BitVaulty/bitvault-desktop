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

/// Computes the effective value of a UTXO after accounting for the fee to spend it
/// 
/// This function calculates how much value a UTXO provides after subtracting the fee required
/// to spend it. Useful for UTXO selection algorithms to determine which UTXOs are
/// economically viable to spend.
///
/// # Arguments
/// * `amount` - The amount of the UTXO
/// * `fee_rate` - The fee rate to use for the calculation
/// * `input_size` - The size of the input in vbytes
///
/// # Returns
/// The effective value of the UTXO in satoshis (may be negative for dust UTXOs)
pub fn effective_value(amount: Amount, fee_rate: FeeRate, input_size: usize) -> i64 {
    // Calculate the fee for spending this input
    let fee = (input_size as f32 * fee_rate.as_sat_per_vb()).ceil() as i64;
    
    // Subtract fee from amount to get effective value
    amount.to_sat() as i64 - fee
}

/// Calculates the waste ratio of a UTXO (fee to spend / amount)
///
/// This metric helps determine which UTXOs are efficient to use. Lower waste ratio
/// is better for efficiency.
///
/// # Arguments
/// * `amount` - The amount of the UTXO
/// * `fee_rate` - The fee rate to use for the calculation
/// * `input_size` - The size of the input in vbytes
///
/// # Returns
/// The waste ratio (fee / amount) as a float
pub fn waste_ratio(amount: Amount, fee_rate: FeeRate, input_size: usize) -> f32 {
    let amount_sats = amount.to_sat() as f32;
    
    // Avoid division by zero
    if amount_sats == 0.0 {
        return f32::INFINITY;
    }
    
    // Calculate fee and waste ratio
    let fee = input_size as f32 * fee_rate.as_sat_per_vb();
    fee / amount_sats
}

/// Calculate the total fee for spending a set of UTXOs
///
/// # Arguments
/// * `input_sizes` - Vector of input sizes in vbytes
/// * `output_sizes` - Vector of output sizes in vbytes
/// * `fee_rate` - The fee rate in satoshis per vbyte
///
/// # Returns
/// The calculated fee in satoshis
pub fn calculate_detailed_fee(input_sizes: &[usize], output_sizes: &[usize], fee_rate: FeeRate) -> u64 {
    const TX_OVERHEAD: usize = 10; // Fixed transaction overhead
    
    // Sum all input and output sizes
    let inputs_total: usize = input_sizes.iter().sum();
    let outputs_total: usize = output_sizes.iter().sum();
    
    // Calculate total vsize
    let total_vsize = TX_OVERHEAD + inputs_total + outputs_total;
    
    // Calculate and return fee
    (total_vsize as f32 * fee_rate.as_sat_per_vb()).ceil() as u64
}

/// Calculates the optimal fee rate based on user priority and current mempool conditions
///
/// # Arguments
/// * `priority` - The user's priority preference (from FeePriority enum)
/// * `mempool_state` - Current mempool status information
///
/// # Returns
/// The optimal fee rate for the given priority and network conditions
pub fn optimal_fee_rate(priority: &crate::types::FeePriority, mempool_state: &crate::network_status::MempoolStatus) -> FeeRate {
    // Base fee rate from the priority
    let base_rate = match priority {
        crate::types::FeePriority::Low => 1.0,
        crate::types::FeePriority::Medium => 3.0,
        crate::types::FeePriority::High => 6.0,
        crate::types::FeePriority::Custom(rate) => *rate,
    };
    
    // Adjust based on mempool congestion
    let congestion_level = mempool_state.determine_congestion_level();
    let congestion_multiplier = match congestion_level {
        crate::network_status::CongestionLevel::Low => 1.0,
        crate::network_status::CongestionLevel::Moderate => 1.25,
        crate::network_status::CongestionLevel::High => 1.5,
        crate::network_status::CongestionLevel::Severe => 2.0,
    };
    
    // Calculate adjusted rate
    let adjusted_rate = base_rate * congestion_multiplier;
    
    // Ensure the fee rate is at least the mempool minimum to avoid stuck transactions
    let final_rate = adjusted_rate.max(mempool_state.min_fee_rate);
    
    FeeRate::from_sat_per_vb(final_rate)
}

/// More precise transaction size estimation considering specific script types and segregated witness data
///
/// This function provides a detailed size calculation that accounts for the different
/// witness and non-witness data proportions in segwit transactions.
///
/// # Arguments
/// * `input_types` - Vector of script types for inputs
/// * `output_types` - Vector of script types for outputs
///
/// # Returns
/// The estimated transaction virtual size in vbytes
pub fn estimate_tx_vsize(input_types: &[crate::types::ScriptType], output_types: &[crate::types::ScriptType]) -> usize {
    // Constants for transaction components
    const TX_OVERHEAD: usize = 10; // Version, locktime, etc.
    const SEGWIT_MARKER: usize = 2; // Marker and flag bytes for segwit txs
    
    // Track if this is a segwit transaction
    let mut has_segwit = false;
    
    // Calculate the input sizes
    let mut input_sizes = Vec::with_capacity(input_types.len());
    for script_type in input_types {
        // Convert script type to string based on the enum variant without moving
        let script_type_str = match script_type {
            crate::types::ScriptType::Pkh => "Legacy (P2PKH)",
            crate::types::ScriptType::Sh => "Wrapped SegWit (P2SH)",
            crate::types::ScriptType::Wpkh => "Native SegWit (P2WPKH)",
            crate::types::ScriptType::Wsh => "Native SegWit Script (P2WSH)",
            crate::types::ScriptType::Tr => "Taproot (P2TR)",
        };
        
        let input_size = get_input_size(script_type_str);
        input_sizes.push(input_size);
        
        // Check if this is a segwit input
        if script_type_str.contains("p2wpkh") || script_type_str.contains("p2wsh") || script_type_str.contains("p2tr") {
            has_segwit = true;
        }
    }
    
    // Calculate the output sizes
    let mut output_sizes = Vec::with_capacity(output_types.len());
    for script_type in output_types {
        // Convert script type to string based on the enum variant without moving
        let script_type_str = match script_type {
            crate::types::ScriptType::Pkh => "Legacy (P2PKH)",
            crate::types::ScriptType::Sh => "Wrapped SegWit (P2SH)",
            crate::types::ScriptType::Wpkh => "Native SegWit (P2WPKH)",
            crate::types::ScriptType::Wsh => "Native SegWit Script (P2WSH)",
            crate::types::ScriptType::Tr => "Taproot (P2TR)",
        };
        
        let output_size = get_output_size(script_type_str);
        output_sizes.push(output_size);
    }
    
    // Calculate total size
    let base_size = TX_OVERHEAD + 
        (if has_segwit { SEGWIT_MARKER } else { 0 }) + 
        input_sizes.iter().sum::<usize>() + 
        output_sizes.iter().sum::<usize>();
    
    base_size
}

/// Calculates the transaction weight according to BIP141
///
/// Weight units are used to compare transactions with and without witness data.
/// A weight unit is 1/4 of a vbyte. Non-witness data counts as 4 weight units per byte,
/// and witness data counts as 1 weight unit per byte.
///
/// # Arguments
/// * `non_witness_size` - The size of non-witness data in bytes
/// * `witness_size` - The size of witness data in bytes
///
/// # Returns
/// The weight of the transaction in weight units
pub fn calculate_tx_weight(non_witness_size: usize, witness_size: usize) -> usize {
    // Per BIP141: weight = base_size * 3 + total_size
    (non_witness_size * 4) + witness_size
}

/// Converts transaction weight to virtual size (vsize)
///
/// # Arguments
/// * `weight` - The weight of the transaction in weight units
///
/// # Returns
/// The virtual size in vbytes (rounded up)
pub fn weight_to_vsize(weight: usize) -> usize {
    // Round up to the nearest vbyte
    (weight + 3) / 4
}

/// Time-weight fee optimization algorithm
///
/// This algorithm finds an optimal fee rate balancing confirmation time 
/// against cost based on user preference and current mempool state.
///
/// # Arguments
/// * `urgency` - Time preference factor (0.0 - 1.0, where 1.0 is most urgent)
/// * `mempool_state` - Current mempool status information
/// * `max_fee_rate` - Maximum acceptable fee rate
///
/// # Returns
/// The optimized fee rate considering the time-cost tradeoff
pub fn optimize_fee_rate(
    urgency: f32, 
    mempool_state: &crate::network_status::MempoolStatus,
    max_fee_rate: FeeRate
) -> FeeRate {
    // Sanitize urgency input
    let urgency = urgency.max(0.0).min(1.0);
    
    // Base parameters
    let min_fee = mempool_state.min_fee_rate;
    let max_fee = max_fee_rate.as_sat_per_vb();
    
    // Get congestion level
    let congestion = mempool_state.determine_congestion_level();
    
    // Calculate congestion factor
    let congestion_factor = match congestion {
        crate::network_status::CongestionLevel::Low => 0.2,
        crate::network_status::CongestionLevel::Moderate => 0.5,
        crate::network_status::CongestionLevel::High => 0.8,
        crate::network_status::CongestionLevel::Severe => 1.0,
    };
    
    // Combine urgency with congestion
    let combined_factor = (urgency + congestion_factor) / 2.0;
    
    // Calculate optimal rate using logarithmic scale to give better granularity at lower fee rates
    let fee_range = max_fee - min_fee;
    let log_factor = 1.0 + (combined_factor * 9.0); // Scale from 1-10
    let optimal_fee = min_fee + (fee_range * (1.0 - (1.0 / log_factor)));
    
    FeeRate::from_sat_per_vb(optimal_fee)
}
