//! MinimizeFee UTXO selection strategy
//!
//! This module implements the MinimizeFee UTXO selection strategy, which
//! aims to minimize transaction fees by selecting larger UTXOs first.

use bitcoin::Amount;
use crate::events::MessageBus;
use crate::utxo_selection::types::{Utxo, SelectionResult};
use crate::utxo_selection::strategies::Strategy;
use crate::utxo_selection::strategies::base;
use crate::utxo_selection::strategies::utils;

/// Strategy for minimizing fees by selecting larger UTXOs first
pub struct MinimizeFeeStrategy;

impl MinimizeFeeStrategy {
    /// Create a new MinimizeFeeStrategy
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for MinimizeFeeStrategy {
    fn name(&self) -> &'static str {
        "MinimizeFee"
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
        
        // Sort UTXOs by effective value in descending order
        let sorted_utxos = utils::sort_by_effective_value(&available.iter().cloned().cloned().collect::<Vec<_>>(), fee_rate);
        
        // Initialize selected UTXOs
        let mut selected = Vec::new();
        let mut selected_amount = Amount::from_sat(0);
        
        // Select UTXOs one by one until we reach the target amount
        for utxo in sorted_utxos {
            // Skip UTXOs with negative effective value
            if utils::effective_value(&utxo, fee_rate) <= 0 {
                continue;
            }
            
            // Add UTXO to selected set
            selected.push(utxo.clone());
            selected_amount += utxo.amount;
            
            // Check if we've reached the target amount plus fees
            let fee = utils::calculate_fee(selected.len(), 2, fee_rate); // 2 outputs: payment + change
            let amount_needed = target_amount + Amount::from_sat(fee);
            
            if selected_amount >= amount_needed {
                // Calculate change amount
                let change_amount = selected_amount - target_amount - Amount::from_sat(fee);
                
                // Check if change is dust
                if change_amount.to_sat() <= dust_threshold && change_amount.to_sat() > 0 {
                    // Change would be dust, try to find a better solution
                    continue;
                }
                
                // If change is not needed, adjust fee for single output
                let actual_fee = if change_amount.to_sat() <= dust_threshold {
                    utils::calculate_fee(selected.len(), 1, fee_rate) // 1 output: payment only
                } else {
                    fee
                };
                
                // Final change amount
                let final_change = selected_amount - target_amount - Amount::from_sat(actual_fee);
                
                return base::create_success_result(
                    selected,
                    Amount::from_sat(actual_fee),
                    final_change,
                );
            }
        }
        
        // If we get here, we couldn't find a suitable selection
        // This should be rare since we already checked total_available >= target_amount
        base::create_insufficient_funds_result(total_available, target_amount)
    }
} 