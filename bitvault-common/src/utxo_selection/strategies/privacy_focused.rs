use bitcoin::Amount;
use crate::events::MessageBus;
use crate::utxo_selection::types::{Utxo, SelectionResult};
use crate::utxo_selection::strategies::Strategy;
use crate::utxo_selection::strategies::base;
use crate::utxo_selection::strategies::utils;
use std::collections::HashSet;

/// Strategy for selecting UTXOs with privacy in mind
pub struct PrivacyFocusedStrategy;

impl PrivacyFocusedStrategy {
    /// Create a new PrivacyFocusedStrategy
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for PrivacyFocusedStrategy {
    /// Name of this strategy
    fn name(&self) -> &'static str {
        "PrivacyFocused"
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
        
        // Group UTXOs by address
        let mut address_groups: std::collections::HashMap<String, Vec<&Utxo>> = std::collections::HashMap::new();
        
        for utxo in &available {
            let address_key = if let Some(addr) = &utxo.address {
                addr.clone()
            } else {
                // If no address, use a unique key based on the outpoint
                format!("outpoint:{}", utxo.outpoint)
            };
            
            address_groups.entry(address_key).or_default().push(utxo);
        }
        
        // Initialize selected UTXOs
        let mut selected = Vec::new();
        let mut selected_amount = Amount::from_sat(0);
        let mut used_addresses = HashSet::new();
        
        // Try to select UTXOs from different addresses first
        // Sort addresses by amount in descending order to favor larger UTXOs first
        let mut sorted_addresses: Vec<_> = address_groups.iter().collect();
        sorted_addresses.sort_by(|a, b| {
            let a_total: Amount = a.1.iter().map(|u| u.amount).sum();
            let b_total: Amount = b.1.iter().map(|u| u.amount).sum();
            b_total.cmp(&a_total)
        });
        
        // First pass: try to select one UTXO from each address
        for (address, group) in &sorted_addresses {
            // Skip if we've already selected from this address
            if used_addresses.contains(address) {
                continue;
            }
            
            // Find the best UTXO from this address group
            // For privacy, prefer the larger UTXOs
            let mut sorted_group = group.to_vec();
            sorted_group.sort_by(|a, b| b.amount.cmp(&a.amount));
            
            if let Some(best_utxo) = sorted_group.first() {
                if utils::effective_value(best_utxo, fee_rate) > 0 {
                    selected.push((*best_utxo).clone());
                    selected_amount += best_utxo.amount;
                    used_addresses.insert(address.clone());
                    
                    // Check if we've reached the target amount plus fees
                    let fee = utils::calculate_fee(selected.len(), 2, fee_rate);
                    if selected_amount.to_sat() >= target_amount.to_sat() + fee {
                        break;
                    }
                }
            }
        }
        
        // Second pass: if we still need more funds, select additional UTXOs
        // by effective value
        if selected_amount.to_sat() < target_amount.to_sat() {
            // Get remaining UTXOs that weren't selected in first pass
            let mut remaining: Vec<&Utxo> = available.iter()
                .filter(|u| !selected.iter().any(|s| s.outpoint == u.outpoint))
                .cloned()
                .collect();
            
            // Sort by effective value
            remaining.sort_by(|a, b| {
                let a_val = utils::effective_value(a, fee_rate);
                let b_val = utils::effective_value(b, fee_rate);
                b_val.cmp(&a_val)
            });
            
            // Add UTXOs until target is reached
            for utxo in remaining {
                // Skip if negative effective value
                if utils::effective_value(utxo, fee_rate) <= 0 {
                    continue;
                }
                
                selected.push(utxo.clone());
                selected_amount += utxo.amount;
                
                // Check if we've reached the target amount plus fees
                let fee = utils::calculate_fee(selected.len(), 2, fee_rate);
                if selected_amount.to_sat() >= target_amount.to_sat() + fee {
                    break;
                }
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
        
        // Create final result
        base::create_success_result(
            selected,
            Amount::from_sat(fee),
            Amount::from_sat(change_amount),
        )
    }
} 