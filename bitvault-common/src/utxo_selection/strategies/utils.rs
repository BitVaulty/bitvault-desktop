//! Utility functions for UTXO selection
//!
//! This module provides utility functions for UTXO selection strategies.

use bitcoin::Amount;
use crate::utxo_selection::types::Utxo;

/// Calculate the effective value of a UTXO after accounting for the fee to spend it
///
/// # Arguments
/// * `utxo` - UTXO to calculate effective value for
/// * `fee_rate_f32` - Fee rate in satoshis per vByte
///
/// # Returns
/// * Effective value in satoshis (may be negative for dust UTXOs)
pub fn effective_value(utxo: &Utxo, fee_rate_f32: f32) -> i64 {
    let input_size = 68; // Approximate size of a P2WPKH input in vBytes
    let fee = (input_size as f32 * fee_rate_f32) as i64;
    utxo.amount.to_sat() as i64 - fee
}

/// Calculate the waste ratio of a UTXO (fee to spend / amount)
///
/// # Arguments
/// * `utxo` - UTXO to calculate waste ratio for
/// * `fee_rate_f32` - Fee rate in satoshis per vByte
///
/// # Returns
/// * Waste ratio (fee to spend / amount)
pub fn waste_ratio(utxo: &Utxo, fee_rate_f32: f32) -> f32 {
    let amount = utxo.amount.to_sat() as f32;
    if amount == 0.0 {
        return f32::INFINITY;
    }
    
    let input_size = 68; // Approximate size of a P2WPKH input in vBytes
    let fee = input_size as f32 * fee_rate_f32;
    fee / amount
}

/// Calculate the fee for a transaction with the given input and output counts
///
/// # Arguments
/// * `input_count` - Number of inputs in the transaction
/// * `output_count` - Number of outputs in the transaction
/// * `fee_rate` - Fee rate in satoshis per vByte
///
/// # Returns
/// * Fee in satoshis
pub fn calculate_fee(input_count: usize, output_count: usize, fee_rate: f32) -> u64 {
    // Transaction overhead: ~11 vBytes
    // P2WPKH input: ~68 vBytes
    // P2WPKH output: ~31 vBytes
    let tx_size = 11 + (input_count * 68) + (output_count * 31);
    (tx_size as f32 * fee_rate).ceil() as u64
}

/// Calculate the total value of a set of UTXOs
///
/// # Arguments
/// * `utxos` - UTXOs to calculate total value for
///
/// # Returns
/// * Total value in satoshis
pub fn total_value(utxos: &[Utxo]) -> u64 {
    utxos.iter().map(|utxo| utxo.amount.to_sat()).sum()
}

/// Calculate the total value of a set of UTXOs as an Amount
///
/// # Arguments
/// * `utxos` - UTXOs to calculate total value for
///
/// # Returns
/// * Total value as Amount
pub fn total_amount(utxos: &[Utxo]) -> Amount {
    utxos.iter().map(|utxo| utxo.amount).sum()
}

/// Sort UTXOs by effective value in descending order
///
/// # Arguments
/// * `utxos` - UTXOs to sort
/// * `fee_rate` - Fee rate in satoshis per vByte
///
/// # Returns
/// * Sorted vector of UTXOs
pub fn sort_by_effective_value(utxos: &[Utxo], fee_rate: f32) -> Vec<Utxo> {
    let mut sorted = utxos.to_vec();
    sorted.sort_by(|a, b| {
        let a_value = effective_value(a, fee_rate);
        let b_value = effective_value(b, fee_rate);
        b_value.cmp(&a_value) // Sort in descending order
    });
    sorted
}

/// Sort UTXOs by waste ratio in ascending order
///
/// # Arguments
/// * `utxos` - UTXOs to sort
/// * `fee_rate` - Fee rate in satoshis per vByte
///
/// # Returns
/// * Sorted vector of UTXOs
pub fn sort_by_waste_ratio(utxos: &[Utxo], fee_rate: f32) -> Vec<Utxo> {
    let mut sorted = utxos.to_vec();
    sorted.sort_by(|a, b| {
        let a_ratio = waste_ratio(a, fee_rate);
        let b_ratio = waste_ratio(b, fee_rate);
        a_ratio.partial_cmp(&b_ratio).unwrap_or(std::cmp::Ordering::Equal) // Sort in ascending order
    });
    sorted
}

/// Sort UTXOs by confirmation count in descending order (oldest first)
///
/// # Arguments
/// * `utxos` - UTXOs to sort
///
/// # Returns
/// * Sorted vector of UTXOs
pub fn sort_by_confirmations(utxos: &[Utxo]) -> Vec<Utxo> {
    let mut sorted = utxos.to_vec();
    sorted.sort_by(|a, b| b.confirmations.cmp(&a.confirmations)); // Sort in descending order
    sorted
}

/// Sort UTXOs by amount in descending order
///
/// # Arguments
/// * `utxos` - UTXOs to sort
///
/// # Returns
/// * Sorted vector of UTXOs
pub fn sort_by_amount(utxos: &[Utxo]) -> Vec<Utxo> {
    let mut sorted = utxos.to_vec();
    sorted.sort_by(|a, b| b.amount.cmp(&a.amount)); // Sort in descending order
    sorted
}

/// Sort UTXOs by amount in ascending order
///
/// # Arguments
/// * `utxos` - UTXOs to sort
///
/// # Returns
/// * Sorted vector of UTXOs
pub fn sort_by_amount_ascending(utxos: &[Utxo]) -> Vec<Utxo> {
    let mut sorted = utxos.to_vec();
    sorted.sort_by(|a, b| a.amount.cmp(&b.amount)); // Sort in ascending order
    sorted
}

/// Group UTXOs by address
///
/// # Arguments
/// * `utxos` - UTXOs to group
///
/// # Returns
/// * Vector of vectors of UTXOs, where each inner vector contains UTXOs with the same address
pub fn group_by_address(utxos: &[Utxo]) -> Vec<Vec<Utxo>> {
    let mut address_map = std::collections::HashMap::new();
    
    // Group UTXOs by address
    for utxo in utxos {
        let address = utxo.address.clone().unwrap_or_else(|| "unknown".to_string());
        address_map.entry(address).or_insert_with(Vec::new).push(utxo.clone());
    }
    
    // Convert the map to a vector of vectors
    address_map.into_iter().map(|(_, utxos)| utxos).collect()
} 