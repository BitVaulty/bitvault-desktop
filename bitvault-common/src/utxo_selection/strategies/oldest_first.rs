//! OldestFirst UTXO selection strategy
//!
//! This module is a placeholder for the OldestFirst UTXO selection strategy.

use bitcoin::Amount;
use crate::events::MessageBus;
use crate::utxo_selection::types::{Utxo, SelectionResult};
use crate::utxo_selection::strategies::Strategy;
use crate::utxo_selection::strategies::base;
use crate::utxo_selection::strategies::utils;

/// Strategy for selecting oldest UTXOs first based on confirmation count
pub struct OldestFirstStrategy;

impl OldestFirstStrategy {
    /// Create a new OldestFirstStrategy
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for OldestFirstStrategy {
    fn name(&self) -> &'static str {
        "OldestFirst"
    }
    
    fn select(
        &self,
        utxos: &[Utxo],
        target_amount: Amount,
        fee_rate: f32,
        dust_threshold: u64,
        _message_bus: Option<&MessageBus>,
    ) -> SelectionResult {
        // Filter available UTXOs (non-frozen)
        let available: Vec<&Utxo> = utxos.iter()
            .filter(|u| !u.is_frozen)
            .collect();
            
        // Get total available amount
        let total_available: Amount = available.iter()
            .map(|u| u.amount)
            .sum();
            
        // Check if we have enough total value
        if total_available < target_amount {
            return base::create_insufficient_funds_result(total_available, target_amount);
        }
        
        // Sort UTXOs by confirmation count in descending order
        let mut sorted_utxos = available.clone();
        sorted_utxos.sort_by(|a, b| b.confirmations.cmp(&a.confirmations));
        
        // Initialize selected UTXOs
        let mut selected = Vec::new();
        let mut selected_amount = Amount::from_sat(0);
        
        // Select UTXOs one by one until we reach the target amount
        for utxo in &sorted_utxos {
            // Skip UTXOs with negative effective value
            if utils::effective_value(utxo, fee_rate) <= 0 {
                continue;
            }
            
            selected.push((*utxo).clone());
            selected_amount += utxo.amount;
            
            // Check if we've reached the target amount plus fees
            let fee = utils::calculate_fee(selected.len(), 2, fee_rate); // 2 outputs: payment + change
            let amount_needed = target_amount.to_sat() + fee;
            
            if selected_amount.to_sat() >= amount_needed {
                break;
            }
        }
        
        // Check if we have enough selected amount
        if selected_amount.to_sat() < target_amount.to_sat() {
            return base::create_insufficient_funds_result(total_available, target_amount);
        }
        
        // Calculate fee
        let fee = utils::calculate_fee(selected.len(), 2, fee_rate);
        
        // Calculate change amount
        let change_amount = selected_amount.to_sat() - target_amount.to_sat() - fee;
        
        // Check if change is dust and adjust if needed
        let final_fee = if change_amount > 0 && change_amount <= dust_threshold {
            // If change is dust, add it to the fee
            fee + change_amount
        } else {
            fee
        };
        
        // Calculate final change amount
        let final_change = if change_amount <= dust_threshold {
            0
        } else {
            change_amount
        };
        
        // Create final result
        base::create_success_result(
            selected,
            Amount::from_sat(final_fee),
            Amount::from_sat(final_change),
        )
    }
} 