//! Base utilities for UTXO selection strategies
//!
//! This module provides common utilities and helper functions that are
//! useful across multiple selection strategies.

use bitcoin::Amount;
use crate::utxo_selection::types::{Utxo, SelectionResult};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

/// Calculate transaction fee based on input and output counts
///
/// # Arguments
/// * `input_count` - Number of inputs in the transaction
/// * `output_count` - Number of outputs in the transaction
/// * `fee_rate` - Fee rate in satoshis per vByte
///
/// # Returns
/// * Estimated fee in satoshis
pub fn calculate_fee(input_count: usize, output_count: usize, fee_rate: f32) -> u64 {
    // Convert fee rate to Decimal for more precise calculation
    let fee_rate_decimal = Decimal::from_f32(fee_rate).unwrap_or(Decimal::new(1, 0));
    
    // Estimate transaction size
    // P2WPKH input: ~68 vBytes
    // P2WPKH output: ~31 vBytes
    // Transaction overhead: ~11 vBytes
    let tx_size = 11 + (input_count * 68) + (output_count * 31);
    
    // Calculate fee
    let fee = fee_rate_decimal * Decimal::from(tx_size);
    
    // Convert to satoshis
    fee.to_u64().unwrap_or(1000)
}

/// Filter available UTXOs
///
/// # Arguments
/// * `utxos` - All UTXOs
/// * `min_confirmations` - Minimum number of confirmations required (0 to include unconfirmed)
///
/// # Returns
/// * Vector of available UTXOs
pub fn filter_available_utxos(utxos: &[Utxo], min_confirmations: u32) -> Vec<&Utxo> {
    utxos.iter()
        .filter(|utxo| !utxo.is_frozen && utxo.confirmations >= min_confirmations)
        .collect()
}

/// Check if we need a change output based on change amount and dust threshold
///
/// # Arguments
/// * `change_amount` - Potential change amount
/// * `dust_threshold` - Dust threshold in satoshis
///
/// # Returns
/// * true if change is needed, false otherwise
pub fn is_change_needed(change_amount: Amount, dust_threshold: u64) -> bool {
    change_amount.to_sat() != 0 && change_amount.to_sat() > dust_threshold
}

/// Calculate total value of UTXOs
///
/// # Arguments
/// * `utxos` - UTXOs to calculate total value for
///
/// # Returns
/// * Total value in satoshis
pub fn total_value(utxos: &[Utxo]) -> u64 {
    utxos.iter().map(|utxo| utxo.amount.to_sat()).sum()
}

/// Calculate total value as Amount
///
/// # Arguments
/// * `utxos` - UTXOs to calculate total value for
///
/// # Returns
/// * Total value as Amount
pub fn total_amount(utxos: &[Utxo]) -> Amount {
    utxos.iter().map(|utxo| utxo.amount).sum()
}

/// Calculate effective value of a UTXO after accounting for the fee to spend it
///
/// # Arguments
/// * `utxo` - UTXO to calculate effective value for
/// * `fee_rate` - Fee rate in satoshis per vByte
///
/// # Returns
/// * Effective value in satoshis (may be negative for dust UTXOs)
pub fn effective_value(utxo: &Utxo, fee_rate: f32) -> i64 {
    // Approximate input size in vBytes (P2WPKH)
    let input_size = 68;
    
    // Calculate fee to spend this input
    let input_fee = (input_size as f32 * fee_rate) as i64;
    
    // Calculate effective value
    utxo.amount.to_sat() as i64 - input_fee
}

/// Calculate waste ratio of a UTXO (fee to spend / amount)
///
/// # Arguments
/// * `utxo` - UTXO to calculate waste ratio for
/// * `fee_rate` - Fee rate in satoshis per vByte
///
/// # Returns
/// * Waste ratio (fee to spend / amount)
pub fn waste_ratio(utxo: &Utxo, fee_rate: f32) -> f32 {
    let amount = utxo.amount.to_sat() as f32;
    if amount == 0.0 {
        return f32::INFINITY;
    }
    
    // Approximate input size in vBytes (P2WPKH)
    let input_size = 68;
    
    // Calculate fee to spend this input
    let input_fee = input_size as f32 * fee_rate;
    
    // Calculate waste ratio
    input_fee / amount
}

/// Create a success result with the given parameters
///
/// # Arguments
/// * `selected` - Selected UTXOs
/// * `fee_amount` - Fee amount
/// * `change_amount` - Change amount
///
/// # Returns
/// * SelectionResult::Success with the given parameters
pub fn create_success_result(
    selected: Vec<Utxo>,
    fee_amount: Amount,
    change_amount: Amount,
) -> SelectionResult {
    SelectionResult::Success {
        selected,
        fee_amount,
        change_amount,
    }
}

/// Create an insufficient funds result with the given parameters
///
/// # Arguments
/// * `available` - Available amount
/// * `required` - Required amount
///
/// # Returns
/// * SelectionResult::InsufficientFunds with the given parameters
pub fn create_insufficient_funds_result(
    available: Amount,
    required: Amount,
) -> SelectionResult {
    SelectionResult::InsufficientFunds {
        available,
        required,
    }
} 