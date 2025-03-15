//! Consolidate UTXO selection strategy
//!
//! This module is a placeholder for the Consolidate UTXO selection strategy.

use bitcoin::Amount;
use crate::events::MessageBus;
use crate::utxo_selection::types::{Utxo, SelectionResult};
use crate::utxo_selection::strategies::Strategy;
use crate::utxo_selection::strategies::base;
use crate::utxo_selection::strategies::utils;

/// Strategy for consolidating many small UTXOs into fewer outputs
pub struct ConsolidateStrategy;

impl ConsolidateStrategy {
    /// Create a new ConsolidateStrategy
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for ConsolidateStrategy {
    fn name(&self) -> &'static str {
        "Consolidate"
    }
    
    fn select(
        &self,
        utxos: &[Utxo],
        target_amount: Amount,
        fee_rate: f32,
        _dust_threshold: u64,
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
        
        // Sort UTXOs by amount in ascending order (smallest first)
        let mut sorted_utxos = utils::sort_by_amount_ascending(&available.iter().cloned().cloned().collect::<Vec<_>>());
        
        // Initialize selected UTXOs
        let mut selected = Vec::new();
        let mut selected_amount = Amount::from_sat(0);
        
        // Select UTXOs one by one until we reach the target amount
        // For consolidation, we prefer to use more small UTXOs
        while !sorted_utxos.is_empty() && selected_amount < target_amount {
            let utxo = sorted_utxos.remove(0); // Take the smallest UTXO
            
            // Only add if it has positive effective value
            if utils::effective_value(&utxo, fee_rate) > 0 {
                selected.push(utxo.clone());
                selected_amount += utxo.amount;
            }
        }
        
        // If we couldn't reach the target amount, try adding larger UTXOs
        if selected_amount < target_amount {
            // Sort remaining UTXOs by descending amount
            sorted_utxos.sort_by(|a, b| b.amount.cmp(&a.amount));
            
            while !sorted_utxos.is_empty() && selected_amount < target_amount {
                let utxo = sorted_utxos.remove(0);
                selected.push(utxo.clone());
                selected_amount += utxo.amount;
            }
        }
        
        // Check if we have enough funds
        if selected_amount < target_amount {
            return base::create_insufficient_funds_result(total_available, target_amount);
        }
        
        // Calculate fee based on tx size
        let fee = utils::calculate_fee(selected.len(), 2, fee_rate); // 2 outputs: payment + change
        
        // Calculate change amount
        let target_sats = target_amount.to_sat();
        let fee_sats = fee;

        // Check if target + fee exceeds selected amount (would cause an overflow)
        if target_sats + fee_sats > selected_amount.to_sat() {
            // Return insufficient funds when we can't cover target + fee
            return base::create_insufficient_funds_result(
                Amount::from_sat(selected_amount.to_sat()),
                Amount::from_sat(target_sats),
            );
        }

        let change_amount = selected_amount.to_sat() - target_sats - fee_sats;
        
        // Create final result
        base::create_success_result(
            selected,
            Amount::from_sat(fee_sats),
            Amount::from_sat(change_amount),
        )
    }
} 