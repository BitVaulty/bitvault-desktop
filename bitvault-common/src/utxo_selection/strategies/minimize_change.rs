//! MinimizeChange UTXO selection strategy
//!
//! This module implements the MinimizeChange UTXO selection strategy, which
//! aims to minimize the change amount by selecting UTXOs that closely match the target.

use bitcoin::Amount;
use crate::events::MessageBus;
use crate::utxo_selection::types::{Utxo, SelectionResult};
use crate::utxo_selection::strategies::Strategy;
use crate::utxo_selection::strategies::base;
use crate::utxo_selection::strategies::utils;
use std::time::{Duration, Instant};

/// Strategy for minimizing change by selecting UTXOs that closely match the target
#[derive(Clone)]
pub struct MinimizeChangeStrategy {
    /// Maximum duration to spend on branch and bound algorithm
    max_duration: Duration,
}

impl MinimizeChangeStrategy {
    /// Create a new MinimizeChangeStrategy with default timeout
    pub fn new() -> Self {
        Self {
            // Default to 1 second timeout
            max_duration: Duration::from_millis(1000),
        }
    }
    
    /// Create a new MinimizeChangeStrategy with custom timeout
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            max_duration: Duration::from_millis(timeout_ms),
        }
    }
    
    /// Recursive branch and bound search for the optimal UTXO selection
    fn branch_and_bound(
        &self,
        utxos: &[Utxo],
        target_amount: Amount,
        fee_rate: f32,
        dust_threshold: u64,
        max_depth: usize,
        start_time: &Instant,
    ) -> Option<Vec<Utxo>> {
        // Check timeout first to quickly exit deep recursion
        if start_time.elapsed() > self.max_duration {
            return None;
        }
        
        // If the target is zero or negative, no UTXOs needed
        if target_amount.to_sat() <= 0 {
            return Some(Vec::new());
        }
        
        // If we have no UTXOs or reached max depth, fail
        if utxos.is_empty() || max_depth == 0 {
            return None;
        }
        
        // Sort UTXOs by amount in ascending order
        let sorted_utxos = utils::sort_by_amount_ascending(utxos);
        
        // Initialize best selection and its waste value
        let mut best_selection: Option<Vec<Utxo>> = None;
        let mut best_waste = u64::MAX;
        
        // Early termination: check if the largest UTXO is smaller than the target
        // If so, we need all UTXOs and should skip branch and bound
        let largest_utxo = sorted_utxos.iter().max_by_key(|u| u.amount.to_sat());
        if let Some(largest) = largest_utxo {
            if largest.amount.to_sat() < target_amount.to_sat() {
                let fee = utils::calculate_fee(sorted_utxos.len(), 2, fee_rate);
                let total = sorted_utxos.iter().map(|u| u.amount.to_sat()).sum::<u64>();
                if total >= target_amount.to_sat() + fee {
                    // If all UTXOs together cover the target + fee, return them all
                    return Some(sorted_utxos.clone());
                }
                // Otherwise we can't satisfy the target
                return None;
            }
        }
        
        // Try each UTXO as the starting point
        // Start with UTXOs closer to the target amount for faster results
        let mut utxo_indices: Vec<usize> = (0..sorted_utxos.len()).collect();
        utxo_indices.sort_by(|&a, &b| {
            let a_diff = (sorted_utxos[a].amount.to_sat() as i64 - target_amount.to_sat() as i64).abs();
            let b_diff = (sorted_utxos[b].amount.to_sat() as i64 - target_amount.to_sat() as i64).abs();
            a_diff.cmp(&b_diff)
        });
        
        for &i in &utxo_indices {
            // Check timeout periodically
            if start_time.elapsed() > self.max_duration {
                break;
            }
            
            let utxo = &sorted_utxos[i];
            
            // Skip UTXOs with negative effective value
            if utils::effective_value(utxo, fee_rate) <= 0 {
                continue;
            }
            
            // Create a new selection with this UTXO
            let mut selection = vec![utxo.clone()];
            let mut selection_amount = utxo.amount;
            
            // Calculate fee for this selection with 2 outputs (payment + change)
            let fee = utils::calculate_fee(selection.len(), 2, fee_rate);
            let amount_needed = target_amount + Amount::from_sat(fee);
            
            // If this UTXO is already enough
            if selection_amount >= amount_needed {
                let change_amount = selection_amount - target_amount - Amount::from_sat(fee);
                let waste = change_amount.to_sat();
                
                // If this is better than our current best, update it
                if waste < best_waste {
                    best_selection = Some(selection);
                    best_waste = waste;
                }
                
                // If waste is zero or very small, we're done
                if waste == 0 || waste <= dust_threshold {
                    return best_selection;
                }
            } else {
                // This UTXO alone is not enough, try adding more
                // Prepare remaining UTXOs (those with indices > i)
                let remaining_utxos = sorted_utxos[i+1..].to_vec();
                
                // Recursively try to find a combination with remaining UTXOs
                if let Some(additional_utxos) = self.branch_and_bound(
                    &remaining_utxos,
                    amount_needed - selection_amount,
                    fee_rate,
                    dust_threshold,
                    max_depth - 1,
                    start_time,
                ) {
                    // Combine the current UTXO with the additional ones
                    selection.extend(additional_utxos);
                    selection_amount = utils::total_amount(&selection);
                    
                    // Recalculate fee with the new selection
                    let new_fee = utils::calculate_fee(selection.len(), 2, fee_rate);
                    let new_amount_needed = target_amount + Amount::from_sat(new_fee);
                    
                    // If we have enough, calculate change
                    if selection_amount >= new_amount_needed {
                        let change_amount = selection_amount - target_amount - Amount::from_sat(new_fee);
                        let waste = change_amount.to_sat();
                        
                        // If this is better than our current best, update it
                        if waste < best_waste {
                            best_selection = Some(selection);
                            best_waste = waste;
                        }
                        
                        // If waste is zero or sufficiently small, we're done
                        if waste == 0 || waste <= dust_threshold {
                            return best_selection;
                        }
                    }
                }
            }
        }
        
        // Return the best selection we found
        best_selection
    }
    
    /// Fallback greedy algorithm for when branch and bound times out
    fn greedy_selection(
        &self,
        utxos: &[Utxo],
        target_amount: Amount,
        fee_rate: f32,
    ) -> Vec<Utxo> {
        // Sort UTXOs by proximity to target amount
        let mut sorted_utxos = utxos.to_vec();
        let target_sat = target_amount.to_sat();
        
        // First try to find a single UTXO closest to but >= target + estimated fee
        // Estimate fee for 1 input, 2 outputs
        let estimated_fee = utils::calculate_fee(1, 2, fee_rate);
        let needed_amount = target_sat + estimated_fee;
        
        // Standard greedy approach if we didn't find a good solution above
        sorted_utxos.sort_by(|a, b| {
            let a_diff = if a.amount.to_sat() >= needed_amount {
                a.amount.to_sat() - needed_amount // If larger, get the difference
            } else {
                u64::MAX // If smaller, sort to the end
            };
            
            let b_diff = if b.amount.to_sat() >= needed_amount {
                b.amount.to_sat() - needed_amount
            } else {
                u64::MAX
            };
            
            a_diff.cmp(&b_diff)
        });
        
        // Check if we found a suitable single UTXO
        if let Some(best_utxo) = sorted_utxos.first() {
            if best_utxo.amount.to_sat() >= needed_amount {
                return vec![best_utxo.clone()];
            }
        }
        
        // Otherwise, select UTXOs greedily until we have enough
        // Sort by amount descending to minimize the number of inputs
        sorted_utxos.sort_by(|a, b| b.amount.cmp(&a.amount));
        
        let mut selected = Vec::new();
        let mut selected_amount = 0;
        
        for utxo in &sorted_utxos {
            // Add UTXO to our selection
            selected.push(utxo.clone());
            selected_amount += utxo.amount.to_sat();
            
            // Recalculate fee with current selection size
            let current_fee = utils::calculate_fee(selected.len(), 2, fee_rate);
            
            // Check if we have enough
            if selected_amount >= target_sat + current_fee {
                break;
            }
        }
        
        // Quick validation of the selected UTXOs
        if !selected.is_empty() {
            let total = utils::total_amount(&selected);
            let fee = utils::calculate_fee(selected.len(), 2, fee_rate);
            
            // If we don't have enough, something went wrong
            if total.to_sat() < target_sat + fee {
                // Log the issue
                println!("Warning: Greedy selection failed to select enough UTXOs");
                println!("  Target: {} sats", target_sat);
                println!("  Fee: {} sats", fee);
                println!("  Selected: {} sats", total.to_sat());
                
                // Fall back to selecting all available UTXOs
                if total.to_sat() < target_sat + fee {
                    return utxos.to_vec();
                }
            }
        }
        
        selected
    }
}

impl Strategy for MinimizeChangeStrategy {
    fn name(&self) -> &'static str {
        "MinimizeChange"
    }
    
    fn select(
        &self,
        utxos: &[Utxo],
        target_amount: Amount,
        fee_rate: f32,
        dust_threshold: u64,
        message_bus: Option<&MessageBus>,
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
        
        // Convert available references to owned UTXOs
        let available_owned = available.iter().map(|&u| u.clone()).collect::<Vec<_>>();
        
        // Simple cases: very small number of UTXOs
        let use_branch_and_bound = available_owned.len() <= 5;
        
        if use_branch_and_bound {
            // Small UTXO set - use full recursion depth
            let max_depth = available_owned.len();
            
            if let Some(selected) = self.branch_and_bound(
                &available_owned,
                target_amount,
                fee_rate,
                dust_threshold,
                max_depth,
                &Instant::now(),
            ) {
                // Calculate fee for the selected UTXOs
                let output_count = if utils::total_amount(&selected) - target_amount <= Amount::from_sat(dust_threshold) {
                    1 // Just payment output
                } else {
                    2 // Payment + change
                };
                
                let fee_amount = utils::calculate_fee(selected.len(), output_count, fee_rate);
                let change_amount = utils::total_amount(&selected) - target_amount - Amount::from_sat(fee_amount);
                
                // Return success
                return base::create_success_result(
                    selected,
                    Amount::from_sat(fee_amount),
                    change_amount,
                );
            }
        }
        
        // For larger UTXO sets, limit the recursion depth and set a timeout
        let start_time = Instant::now();
        
        // Limit max depth based on UTXO count
        let max_depth = match available_owned.len() {
            0..=10 => available_owned.len(),
            11..=50 => 8,
            _ => 5 // Very limited depth for large UTXO sets
        };
        
        // Try branch and bound with timeout
        let selected = if let Some(selected) = self.branch_and_bound(
            &available_owned,
            target_amount,
            fee_rate,
            dust_threshold,
            max_depth,
            &start_time,
        ) {
            selected
        } else {
            // If branch and bound timed out or failed, fall back to greedy algorithm
            if let Some(bus) = message_bus {
                use crate::events::{EventType, MessagePriority};
                use serde_json::json;
                
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "warning": "Branch and bound algorithm timed out",
                        "fallback": "Using greedy algorithm instead",
                        "strategy": "MinimizeChange"
                    }).to_string(),
                    MessagePriority::Low
                );
            }
            
            self.greedy_selection(&available_owned, target_amount, fee_rate)
        };
        
        // If we couldn't find a selection, return insufficient funds
        if selected.is_empty() {
            return base::create_insufficient_funds_result(total_available, target_amount);
        }
        
        // Calculate fee for the selected UTXOs
        let output_count = if utils::total_amount(&selected) - target_amount <= Amount::from_sat(dust_threshold) {
            1 // Just payment output
        } else {
            2 // Payment + change
        };
        
        let fee_amount = utils::calculate_fee(selected.len(), output_count, fee_rate);
        let change_amount = utils::total_amount(&selected) - target_amount - Amount::from_sat(fee_amount);
        
        // Return success
        base::create_success_result(
            selected,
            Amount::from_sat(fee_amount),
            change_amount,
        )
    }
} 