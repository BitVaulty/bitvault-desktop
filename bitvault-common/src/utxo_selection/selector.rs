//! Main UTXO selector implementation
//!
//! This module provides the main UtxoSelector implementation that delegates
//! to appropriate strategy implementations based on the requested selection strategy.
//!
//! # Overview
//!
//! The `UtxoSelector` is the primary entry point for UTXO selection in the BitVault wallet.
//! It implements the Strategy design pattern, delegating the actual selection logic to
//! specialized strategy implementations based on the user's selection criteria.
//!
//! # Features
//!
//! - Strategy delegation based on selection requirements
//! - Event publication for monitoring and UI feedback via domain-specific event bus
//! - General and domain-specific event bus integration
//! - Dust threshold handling based on network type
//! - Custom fee rate specification
//! - Support for specialized coin control (manual UTXO selection)
//!
//! # Usage
//!
//! ```ignore
//! use bitvault_common::utxo_selection::selector::UtxoSelector;
//! use bitvault_common::utxo_selection::types::{SelectionStrategy, SelectionResult};
//! use bitvault_common::events::{MessageBus, UtxoEventBus};
//! use bitcoin::Amount;
//! use std::sync::Arc;
//!
//! // Create a selector with default settings (1 sat/vB fee rate)
//! let selector = UtxoSelector::new();
//!
//! // Or create with a specific fee rate
//! let selector = UtxoSelector::with_fee_rate(5.0); // 5 sat/vB
//!
//! // Or create with an attached domain-specific event bus
//! let utxo_bus = Arc::new(UtxoEventBus::new());
//! let (selector, event_bus) = UtxoSelector::with_event_bus(utxo_bus);
//!
//! // General message bus for system-wide events
//! let message_bus = MessageBus::new();
//!
//! // Select UTXOs for a transaction
//! let result = selector.select_utxos(
//!     &utxos,
//!     Amount::from_sat(50_000), // target amount
//!     SelectionStrategy::MinimizeFee, // strategy
//!     Some(&message_bus), // Optional general message bus
//!     Some(&event_bus), // Optional domain-specific event bus
//! );
//!
//! // Or use the simplified API that handles event bus creation
//! let (result, created_bus) = selector.select_utxos_with_events(
//!     &utxos,
//!     Amount::from_sat(50_000),
//!     SelectionStrategy::MinimizeFee,
//!     Some(&message_bus),
//! );
//!
//! // Process the result
//! match result {
//!     SelectionResult::Success { selected, fee_amount, change_amount } => {
//!         // Use the selected UTXOs to create a transaction
//!     },
//!     SelectionResult::InsufficientFunds { available, required } => {
//!         // Handle insufficient funds case
//!     }
//! }
//! ```
//!
//! # Event Publication
//!
//! The selector publishes events at key points in the UTXO selection process:
//!
//! ## General Events (via MessageBus)
//! - `EventType::UtxoSelected` - When selection begins
//! - `EventType::UtxoStatusChanged` - When status changes (e.g., insufficient funds)
//! - `EventType::UtxoSelectionCompleted` - When selection completes successfully
//!
//! ## Domain-Specific Events (via UtxoEventBus)
//! - `UtxoEvent::Selected` - When UTXOs are selected successfully
//! - `UtxoEvent::SelectionFailed` - When selection fails (e.g., insufficient funds)
//!
//! # Security Considerations
//!
//! - The selector itself does not handle private keys or signatures
//! - Selection strategies can impact transaction privacy and fees
//! - Fee rate settings directly affect transaction confirmation times
//! - Event data contains sensitive information about wallet state and should be handled accordingly

use bitcoin::{Amount, Network};
use crate::events::{MessageBus, EventType, MessagePriority, UtxoEvent, OutPointInfo, UtxoEventBus};
use serde_json::json;
use crate::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use crate::utxo_selection::strategies::{
    Strategy, MinimizeFeeStrategy, MinimizeChangeStrategy, OldestFirstStrategy,
    PrivacyFocusedStrategy, MaximizePrivacyStrategy, ConsolidateStrategy, AvoidChangeStrategy,
};
use crate::types::get_dust_threshold;
use std::sync::Arc;

/// UTXO selector that uses various strategies to select UTXOs
///
/// This struct is the main entry point for UTXO selection. It delegates to
/// specific strategy implementations based on the requested selection criteria.
///
/// # Design Pattern
///
/// The `UtxoSelector` implements the Strategy pattern, where:
/// - `UtxoSelector` is the context
/// - `Strategy` trait defines the interface for strategies
/// - Concrete strategy classes implement specific selection algorithms
pub struct UtxoSelector {
    /// Network to use for dust calculations
    network: Network,
    /// Fee rate in satoshis per vByte
    fee_rate: f32,
    /// Custom minimize change strategy (if provided)
    custom_minimize_change: Option<MinimizeChangeStrategy>,
}

impl UtxoSelector {
    /// Create a new UTXO selector with default settings
    ///
    /// # Returns
    ///
    /// A new `UtxoSelector` with:
    /// - Network set to Bitcoin mainnet
    /// - Fee rate set to 1 sat/vByte
    /// - No custom minimize change strategy
    pub fn new() -> Self {
        Self {
            network: Network::Bitcoin,
            fee_rate: 1.0, // Default to 1 sat/vByte
            custom_minimize_change: None,
        }
    }
    
    /// Create a new UTXO selector with the specified fee rate
    ///
    /// # Arguments
    ///
    /// * `fee_rate` - Fee rate in satoshis per vByte
    ///
    /// # Returns
    ///
    /// A new `UtxoSelector` with the specified fee rate
    pub fn with_fee_rate(fee_rate: f32) -> Self {
        Self {
            network: Network::Bitcoin,
            fee_rate,
            custom_minimize_change: None,
        }
    }
    
    /// Create a new UTXO selector with a custom minimize change strategy
    ///
    /// # Arguments
    ///
    /// * `strategy` - Custom minimize change strategy implementation
    ///
    /// # Returns
    ///
    /// A new `UtxoSelector` with the specified custom strategy
    pub fn with_minimize_change_strategy(strategy: MinimizeChangeStrategy) -> Self {
        Self {
            network: Network::Bitcoin,
            fee_rate: 1.0,
            custom_minimize_change: Some(strategy),
        }
    }
    
    /// Set the network for this selector
    pub fn with_network(mut self, network: Network) -> Self {
        self.network = network;
        self
    }
    
    /// Set the fee rate for this selector
    pub fn set_fee_rate(&mut self, fee_rate: f32) -> &mut Self {
        self.fee_rate = fee_rate;
        self
    }
    
    /// Get the current fee rate
    pub fn fee_rate(&self) -> f32 {
        self.fee_rate
    }
    
    /// Get the dust threshold for the current network
    ///
    /// This determines the minimum amount for a valid output.
    /// Amounts below this threshold are considered "dust" and typically
    /// cannot be economically spent.
    fn dust_threshold(&self) -> u64 {
        get_dust_threshold(self.network)
    }
    
    /// Create a new UTXO selector with an attached domain-specific event bus
    ///
    /// # Arguments
    ///
    /// * `utxo_bus` - Domain-specific UTXO event bus for event publication
    ///
    /// # Returns
    ///
    /// A new `UtxoSelector` instance with the specified event bus
    pub fn with_event_bus(utxo_bus: Arc<UtxoEventBus>) -> (Self, Arc<UtxoEventBus>) {
        let selector = Self::new();
        (selector, utxo_bus)
    }

    /// Create a new UTXO selector with a custom fee rate and an attached domain-specific event bus
    ///
    /// # Arguments
    ///
    /// * `fee_rate` - Fee rate in satoshis per vByte
    /// * `utxo_bus` - Domain-specific UTXO event bus for event publication
    ///
    /// # Returns
    ///
    /// A new `UtxoSelector` instance with the specified fee rate and event bus
    pub fn with_fee_rate_and_event_bus(fee_rate: f32, utxo_bus: Arc<UtxoEventBus>) -> (Self, Arc<UtxoEventBus>) {
        let selector = Self::with_fee_rate(fee_rate);
        (selector, utxo_bus)
    }
    
    /// Select UTXOs using the specified strategy
    ///
    /// This is the main method for UTXO selection. It delegates to the
    /// appropriate strategy implementation based on the requested strategy.
    ///
    /// # Arguments
    ///
    /// * `utxos` - Available UTXOs to select from
    /// * `target_amount` - Amount to be spent (not including fees)
    /// * `strategy` - Selection strategy to use
    /// * `message_bus` - Optional message bus for event publication
    /// * `utxo_bus` - Optional domain-specific UTXO event bus
    ///
    /// # Returns
    ///
    /// * `SelectionResult` - The result of UTXO selection
    pub fn select_utxos(
        &self,
        utxos: &[Utxo],
        target_amount: Amount,
        strategy: SelectionStrategy,
        message_bus: Option<&MessageBus>,
        utxo_bus: Option<&UtxoEventBus>,
    ) -> SelectionResult {
        // Log the selection request using the general message bus
        if let Some(bus) = message_bus {
            bus.publish(
                EventType::UtxoSelected,
                &json!({
                    "target_amount": target_amount.to_sat(),
                    "strategy": format!("{:?}", strategy),
                    "utxo_count": utxos.len(),
                }).to_string(),
                MessagePriority::Low,
            );
        }
        
        // Available UTXOs for selection (excluding frozen)
        let available: Vec<Utxo> = utxos.iter()
            .filter(|u| !u.is_frozen)
            .cloned()
            .collect();
            
        // Get total available amount
        let total_available: Amount = available.iter()
            .map(|u| u.amount)
            .sum();
            
        // Check if we have enough total value
        if total_available < target_amount {
            // Publish to general message bus
            if let Some(bus) = message_bus {
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "reason": "insufficient_funds",
                        "available": total_available.to_sat(),
                        "required": target_amount.to_sat(),
                    }).to_string(),
                    MessagePriority::Medium,
                );
            }
            
            // Publish to domain-specific UTXO event bus
            if let Some(bus) = utxo_bus {
                bus.publish(UtxoEvent::SelectionFailed {
                    reason: "insufficient_funds".to_string(),
                    strategy: format!("{:?}", strategy),
                    target_amount: target_amount.to_sat(),
                    available_amount: total_available.to_sat(),
                });
            }
            
            return SelectionResult::InsufficientFunds {
                available: total_available,
                required: target_amount,
            };
        }
        
        // Select strategy implementation based on the requested strategy
        let strategy_impl: Box<dyn Strategy> = match strategy {
            SelectionStrategy::MinimizeFee => Box::new(MinimizeFeeStrategy::new()),
            SelectionStrategy::MinimizeChange => {
                if let Some(custom_strategy) = &self.custom_minimize_change {
                    // Use custom minimize change strategy if provided
                    Box::new(custom_strategy.clone())
                } else {
                    // Otherwise use default
                    Box::new(MinimizeChangeStrategy::new())
                }
            },
            SelectionStrategy::OldestFirst => Box::new(OldestFirstStrategy::new()),
            SelectionStrategy::PrivacyFocused => Box::new(PrivacyFocusedStrategy::new()),
            SelectionStrategy::MaximizePrivacy => Box::new(MaximizePrivacyStrategy::new()),
            SelectionStrategy::Consolidate => Box::new(ConsolidateStrategy::new()),
            SelectionStrategy::AvoidChange => Box::new(AvoidChangeStrategy::new()),
            SelectionStrategy::CoinControl => {
                // CoinControl is handled separately, in select_coin_control method
                // For now, fall back to MinimizeFee strategy
                Box::new(MinimizeFeeStrategy::new())
            },
        };
        
        // Execute the selected strategy
        let selection_result = strategy_impl.select(
            &available,
            target_amount,
            self.fee_rate,
            self.dust_threshold(),
            message_bus,
        );
        
        // If successful, publish event with selected UTXOs
        match &selection_result {
            SelectionResult::Success { selected, fee_amount, change_amount } => {
                // Publish to general message bus
                if let Some(bus) = message_bus {
                    bus.publish(
                        EventType::UtxoSelectionCompleted,
                        &json!({
                            "strategy": format!("{:?}", strategy),
                            "selected_count": selected.len(),
                            "fee_amount": fee_amount.to_sat(),
                            "change_amount": change_amount.to_sat(),
                            "selected_utxos": selected.iter().map(|u| {
                                json!({
                                    "txid": u.outpoint.txid.to_string(),
                                    "vout": u.outpoint.vout,
                                    "amount": u.amount.to_sat(),
                                })
                            }).collect::<Vec<_>>(),
                        }).to_string(),
                        MessagePriority::Medium,
                    );
                }
                
                // Publish to domain-specific UTXO event bus
                if let Some(bus) = utxo_bus {
                    bus.publish(UtxoEvent::Selected {
                        utxos: selected.iter().map(|u| OutPointInfo::from(&u.outpoint)).collect(),
                        strategy: format!("{:?}", strategy),
                        target_amount: target_amount.to_sat(),
                        fee_amount: fee_amount.to_sat(),
                        change_amount: Some(change_amount.to_sat()),
                    });
                }
            },
            SelectionResult::InsufficientFunds { available, required } => {
                // This case was already handled above, but we could add more detailed information here
                if let Some(bus) = utxo_bus {
                    bus.publish(UtxoEvent::SelectionFailed {
                        reason: "insufficient_funds_after_fees".to_string(),
                        strategy: format!("{:?}", strategy),
                        target_amount: required.to_sat(),
                        available_amount: available.to_sat(),
                    });
                }
            }
        }
        
        selection_result
    }

    /// Select specific UTXOs for coin control
    ///
    /// This method handles the special case of coin control, where the user
    /// has manually selected specific UTXOs to use in a transaction.
    ///
    /// # Arguments
    ///
    /// * `selected_utxos` - UTXOs that the user wants to use
    /// * `target_amount` - Target amount to send
    /// * `message_bus` - Optional message bus for emitting events
    /// * `utxo_bus` - Optional domain-specific UTXO event bus
    ///
    /// # Returns
    ///
    /// * `SelectionResult` - The result of the selection process
    pub fn select_coin_control(
        &self,
        selected_utxos: &[Utxo],
        target_amount: Amount,
        message_bus: Option<&MessageBus>,
        utxo_bus: Option<&UtxoEventBus>,
    ) -> SelectionResult {
        // Log the selection request
        if let Some(bus) = message_bus {
            bus.publish(
                EventType::UtxoSelected,
                &json!({
                    "target_amount": target_amount.to_sat(),
                    "strategy": "CoinControl",
                    "utxo_count": selected_utxos.len(),
                }).to_string(),
                MessagePriority::Low,
            );
        }
        
        // If no utxos provided, return error
        if selected_utxos.is_empty() {
            if let Some(bus) = message_bus {
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "reason": "no_utxos_provided",
                        "strategy": "CoinControl",
                    }).to_string(),
                    MessagePriority::Medium,
                );
            }
            
            if let Some(bus) = utxo_bus {
                bus.publish(UtxoEvent::SelectionFailed {
                    reason: "no_utxos_provided".to_string(),
                    strategy: "CoinControl".to_string(),
                    target_amount: target_amount.to_sat(),
                    available_amount: 0,
                });
            }
            
            return SelectionResult::InsufficientFunds {
                available: Amount::from_sat(0),
                required: target_amount,
            };
        }
        
        // Calculate the total amount of the selected UTXOs
        let total_available: Amount = selected_utxos.iter().map(|u| u.amount).sum();
        
        // Check if we have enough total value
        if total_available < target_amount {
            if let Some(bus) = message_bus {
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "reason": "insufficient_funds",
                        "available": total_available.to_sat(),
                        "required": target_amount.to_sat(),
                    }).to_string(),
                    MessagePriority::Medium,
                );
            }
            
            if let Some(bus) = utxo_bus {
                bus.publish(UtxoEvent::SelectionFailed {
                    reason: "insufficient_funds".to_string(),
                    strategy: "CoinControl".to_string(),
                    target_amount: target_amount.to_sat(),
                    available_amount: total_available.to_sat(),
                });
            }
            
            return SelectionResult::InsufficientFunds {
                available: total_available,
                required: target_amount,
            };
        }
        
        // Calculate fee based on transaction size
        let input_cost = selected_utxos.len() as u64 * 68;
        let output_cost = 2 * 34; // Payment + change
        let base_cost = 10; // Base tx overhead
        let total_size = input_cost + output_cost + base_cost;
        let fee_amount = Amount::from_sat((total_size as f32 * self.fee_rate).ceil() as u64);
        
        // Calculate change amount
        let change_amount = total_available - target_amount - fee_amount;
        
        // Check if the result is valid with fees
        if change_amount < Amount::from_sat(0) {
            if let Some(bus) = utxo_bus {
                bus.publish(UtxoEvent::SelectionFailed {
                    reason: "insufficient_funds_after_fees".to_string(),
                    strategy: "CoinControl".to_string(),
                    target_amount: target_amount.to_sat(),
                    available_amount: total_available.to_sat(),
                });
            }
            
            return SelectionResult::InsufficientFunds {
                available: total_available,
                required: target_amount + fee_amount,
            };
        }
        
        // Create success result
        let result = SelectionResult::Success {
            selected: selected_utxos.to_vec(),
            fee_amount,
            change_amount,
        };
        
        // Publish successful selection events
        if let Some(bus) = message_bus {
            bus.publish(
                EventType::UtxoSelectionCompleted,
                &json!({
                    "strategy": "CoinControl",
                    "selected_count": selected_utxos.len(),
                    "fee_amount": fee_amount.to_sat(),
                    "change_amount": change_amount.to_sat(),
                    "selected_utxos": selected_utxos.iter().map(|u| {
                        json!({
                            "txid": u.outpoint.txid.to_string(),
                            "vout": u.outpoint.vout,
                            "amount": u.amount.to_sat(),
                        })
                    }).collect::<Vec<_>>(),
                }).to_string(),
                MessagePriority::Medium,
            );
        }
        
        // Publish to domain-specific UTXO event bus
        if let Some(bus) = utxo_bus {
            bus.publish(UtxoEvent::Selected {
                utxos: selected_utxos.iter().map(|u| OutPointInfo::from(&u.outpoint)).collect(),
                strategy: "CoinControl".to_string(),
                target_amount: target_amount.to_sat(),
                fee_amount: fee_amount.to_sat(),
                change_amount: Some(change_amount.to_sat()),
            });
        }
        
        result
    }

    /// Enhanced selection method that creates its own domain-specific event bus if not provided
    ///
    /// This method simplifies the usage of the event-driven architecture by managing
    /// the creation of necessary event buses internally.
    ///
    /// # Arguments
    ///
    /// * `utxos` - Available UTXOs to select from
    /// * `target_amount` - Amount to be spent (not including fees)
    /// * `strategy` - Selection strategy to use
    /// * `message_bus` - Optional message bus for event publication
    ///
    /// # Returns
    ///
    /// * `(SelectionResult, Option<Arc<UtxoEventBus>>)` - The result of UTXO selection and optionally the created event bus
    pub fn select_utxos_with_events(
        &self,
        utxos: &[Utxo],
        target_amount: Amount,
        strategy: SelectionStrategy,
        message_bus: Option<&MessageBus>,
    ) -> (SelectionResult, Option<Arc<UtxoEventBus>>) {
        // Create a new UTXO event bus without connecting to general bus
        let utxo_bus = Some(Arc::new(UtxoEventBus::new()));
        
        // Use the standard selection method with our event bus
        let result = self.select_utxos(
            utxos,
            target_amount,
            strategy,
            message_bus,
            utxo_bus.as_ref().map(|bus| bus.as_ref()),
        );
        
        (result, utxo_bus)
    }
} 