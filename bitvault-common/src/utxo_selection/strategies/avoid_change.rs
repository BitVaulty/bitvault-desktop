//! AvoidChange UTXO selection strategy
//!
//! This module is a placeholder for the AvoidChange UTXO selection strategy.

use crate::utxo_selection::strategies::Strategy;

pub struct AvoidChangeStrategy;

impl Strategy for AvoidChangeStrategy {
    fn name(&self) -> &'static str {
        "AvoidChange"
    }

    fn select(
        &self,
        utxos: &[crate::utxo_selection::types::Utxo],
        target_amount: bitcoin::Amount,
        fee_rate: f32,
        dust_threshold: u64,
        _message_bus: Option<&crate::events::MessageBus>,
    ) -> crate::utxo_selection::types::SelectionResult {
        let mut selected_utxos = Vec::new();
        let mut total_selected = bitcoin::Amount::from_sat(0);
        let fee_amount = bitcoin::Amount::from_sat((fee_rate * 1000.0) as u64); // Simplified fee calculation

        // Filter out frozen and unconfirmed UTXOs
        let available_utxos: Vec<_> = utxos.iter()
            .filter(|u| !u.is_frozen && u.is_confirmed())
            .collect();

        for utxo in available_utxos {
            if total_selected >= target_amount + fee_amount {
                break;
            }
            selected_utxos.push(utxo.clone());
            total_selected += utxo.amount;
        }

        if total_selected >= target_amount + fee_amount {
            // Calculate change amount - the difference between what we selected and what we need
            let change_amount = total_selected - target_amount - fee_amount;
            
            // For the AvoidChange strategy, if the change is very small (below dust threshold),
            // we add it to the fee instead
            if change_amount.to_sat() > 0 && change_amount.to_sat() < dust_threshold {
                // Add the change to the fee
                let adjusted_fee = fee_amount + change_amount;
                
                crate::utxo_selection::types::SelectionResult::Success {
                    selected: selected_utxos,
                    fee_amount: adjusted_fee,
                    change_amount: bitcoin::Amount::from_sat(0),
                }
            } else {
                crate::utxo_selection::types::SelectionResult::Success {
                    selected: selected_utxos,
                    fee_amount,
                    change_amount,
                }
            }
        } else {
            crate::utxo_selection::types::SelectionResult::InsufficientFunds {
                available: total_selected,
                required: target_amount + fee_amount,
            }
        }
    }
}

impl AvoidChangeStrategy {
    /// Create a new AvoidChangeStrategy
    pub fn new() -> Self {
        Self
    }
} 