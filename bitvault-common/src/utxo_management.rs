//! UTXO management functionality
//!
//! This module provides utilities for managing Unspent Transaction Outputs (UTXOs)
//! in the BitVault wallet, including selecting UTXOs for transactions, tracking their
//! status, and controlling their availability.
//!
//! # Overview
//!
//! The `UtxoManager` is responsible for:
//! - Maintaining a collection of available UTXOs
//! - Handling UTXO selection for transactions using various strategies
//! - Tracking UTXO state (e.g., frozen/unfrozen)
//! - Publishing events related to UTXO lifecycle and selection
//!
//! # Event-Driven Architecture
//!
//! The UTXO management functionality is built using an event-driven architecture
//! that allows components to communicate asynchronously:
//!
//! - `UtxoEventBus` provides domain-specific events for UTXO operations
//! - `MessageBus` provides general system-wide events
//! - Event subscribers can react to UTXO state changes without direct coupling
//!
//! # Events Published
//!
//! The following events are published by the UTXO manager:
//!
//! ## Domain-Specific Events (UtxoEvent)
//! - `Selected` - When UTXOs are selected for a transaction
//! - `Frozen` - When a UTXO is frozen (prevented from being selected)
//! - `Unfrozen` - When a UTXO is unfrozen (allowed to be selected)
//! - `SelectionFailed` - When UTXO selection fails (e.g., insufficient funds)
//! - `StatusChanged` - When a UTXO's status changes (e.g., added, confirmed)
//!
//! # Usage
//!
//! ```ignore
//! use bitvault_common::utxo_management::UtxoManager;
//! use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
//! use bitvault_common::events::{MessageBus, UtxoEventBus};
//! use bitcoin::{Amount, OutPoint, Txid};
//! use std::sync::Arc;
//! use std::str::FromStr;
//!
//! // Create a general message bus
//! let mut message_bus = MessageBus::new();
//! message_bus.start();
//!
//! // Create a UTXO manager with a new event bus
//! let (mut manager, event_bus) = UtxoManager::with_new_event_bus();
//!
//! // Or connect to an existing general bus
//! let (manager, event_bus) = UtxoManager::with_connected_event_bus(Arc::new(message_bus));
//!
//! // Subscribe to UTXO events
//! let selected_events = event_bus.subscribe("selected");
//! let all_events = event_bus.subscribe_all();
//!
//! // Add UTXOs to the manager
//! let utxo = Utxo::new(
//!     OutPoint::new(Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 0),
//!     Amount::from_sat(50_000),
//!     6, // confirmations
//!     false, // is_change
//! );
//! manager.add_utxo(utxo);
//!
//! // Select UTXOs for a transaction
//! let result = manager.select_utxos(
//!     Amount::from_sat(25_000),
//!     SelectionStrategy::MinimizeFee,
//!     Some(&message_bus),
//!     Some(&event_bus),
//! );
//!
//! // Or use the enhanced selection with event handling
//! let (result, new_bus) = manager.select_utxos_with_events(
//!     Amount::from_sat(25_000),
//!     SelectionStrategy::MinimizeFee,
//!     Some(&message_bus),
//!     true, // create a new event bus if none exists
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
//! # Security Considerations
//!
//! - UTXO data contains sensitive information about wallet state
//! - Events may leak information about available funds if not properly secured
//! - Freezing UTXOs is an important mechanism for preventing certain UTXOs from being spent
//! - Coin selection strategies impact privacy and transaction efficiency

use bitcoin::{Amount, OutPoint};
use crate::events::{MessageBus, EventType, MessagePriority, UtxoEvent, OutPointInfo, UtxoEventBus};
use crate::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use crate::utxo_selection::selector::UtxoSelector;
use serde_json::json;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

/// Manages and selects UTXOs for transactions
pub struct UtxoManager {
    /// The UTXOs available for selection
    utxos: Vec<Utxo>,
    /// Domain-specific UTXO event bus
    utxo_bus: Option<Arc<UtxoEventBus>>,
}

impl UtxoManager {
    /// Creates a new UTXO manager with an empty UTXO set.
    pub fn new() -> Self {
        UtxoManager { 
            utxos: Vec::new(),
            utxo_bus: None,
        }
    }
    
    /// Creates a new UTXO manager with an empty UTXO set and a domain-specific event bus.
    pub fn with_event_bus(utxo_bus: Arc<UtxoEventBus>) -> Self {
        UtxoManager { 
            utxos: Vec::new(),
            utxo_bus: Some(utxo_bus),
        }
    }

    /// Creates a new UTXO manager with an empty UTXO set and a new domain-specific event bus.
    pub fn with_new_event_bus() -> (Self, Arc<UtxoEventBus>) {
        let utxo_bus = Arc::new(UtxoEventBus::new());
        let manager = UtxoManager {
            utxos: Vec::new(),
            utxo_bus: Some(Arc::clone(&utxo_bus)),
        };
        (manager, utxo_bus)
    }

    /// Creates a new UTXO manager with an event bus connected to a general message bus.
    pub fn with_connected_event_bus(general_bus: Arc<MessageBus>) -> (Self, Arc<UtxoEventBus>) {
        let utxo_bus = Arc::new(UtxoEventBus::with_general_bus(general_bus));
        let manager = UtxoManager {
            utxos: Vec::new(),
            utxo_bus: Some(Arc::clone(&utxo_bus)),
        };
        (manager, utxo_bus)
    }

    /// Sets the domain-specific event bus for this manager.
    pub fn set_event_bus(&mut self, utxo_bus: Arc<UtxoEventBus>) {
        self.utxo_bus = Some(utxo_bus);
    }

    /// Gets a reference to the domain-specific event bus if available.
    pub fn get_event_bus(&self) -> Option<&Arc<UtxoEventBus>> {
        self.utxo_bus.as_ref()
    }

    /// Subscribes to a specific type of UTXO event.
    /// 
    /// # Arguments
    /// * `event_type` - The type of event to subscribe to (e.g., "selected", "frozen", "unfrozen", "status_changed")
    ///
    /// # Returns
    /// * `Option<Receiver<UtxoEvent>>` - A receiver for events, or None if no event bus is available
    pub fn subscribe(&self, event_type: &str) -> Option<Receiver<UtxoEvent>> {
        self.utxo_bus.as_ref().map(|bus| bus.subscribe(event_type))
    }

    /// Subscribes to all UTXO events.
    ///
    /// # Returns
    /// * `Option<Receiver<UtxoEvent>>` - A receiver for all events, or None if no event bus is available
    pub fn subscribe_all(&self) -> Option<Receiver<UtxoEvent>> {
        self.utxo_bus.as_ref().map(|bus| bus.subscribe_all())
    }

    /// Adds a UTXO to the manager.
    pub fn add_utxo(&mut self, utxo: Utxo) {
        // Publish an event if the event bus is available
        if let Some(ref bus) = self.utxo_bus {
            bus.publish(UtxoEvent::StatusChanged {
                outpoint: OutPointInfo::from(&utxo.outpoint),
                status: "added".to_string(),
            });
        }
        
        self.utxos.push(utxo);
    }

    /// Adds multiple UTXOs to the manager.
    pub fn add_utxos(&mut self, utxos: Vec<Utxo>) {
        for utxo in utxos {
            self.add_utxo(utxo);
        }
    }

    /// Freezes a UTXO to prevent it from being selected.
    pub fn freeze_utxo(&mut self, outpoint: &OutPoint) -> bool {
        for utxo in &mut self.utxos {
            if utxo.outpoint == *outpoint {
                utxo.is_frozen = true;
                
                // Publish an event if the event bus is available
                if let Some(ref bus) = self.utxo_bus {
                    bus.publish(UtxoEvent::Frozen {
                        outpoint: OutPointInfo::from(outpoint),
                    });
                }
                
                return true;
            }
        }
        false
    }

    /// Unfreezes a UTXO to allow it to be selected.
    pub fn unfreeze_utxo(&mut self, outpoint: &OutPoint) -> bool {
        for utxo in &mut self.utxos {
            if utxo.outpoint == *outpoint {
                utxo.is_frozen = false;
                
                // Publish an event if the event bus is available
                if let Some(ref bus) = self.utxo_bus {
                    bus.publish(UtxoEvent::Unfrozen {
                        outpoint: OutPointInfo::from(outpoint),
                    });
                }
                
                return true;
            }
        }
        false
    }

    /// Selects UTXOs for a transaction based on the given strategy.
    ///
    /// # Arguments
    /// * `target` - The target amount to send
    /// * `strategy` - The strategy to use for selecting UTXOs
    /// * `message_bus` - Optional message bus for emitting events
    /// * `utxo_bus` - Optional UTXO event bus for emitting events
    ///
    /// # Returns
    /// * `SelectionResult` - The result of the selection process
    pub fn select_utxos(
        &self, 
        target: Amount, 
        strategy: SelectionStrategy, 
        message_bus: Option<&MessageBus>,
        utxo_bus: Option<&UtxoEventBus>
    ) -> SelectionResult {
        println!("[DEBUG] select_utxos called with target: {}, strategy: {:?}", target, strategy);
        
        // CoinControl strategy requires pre-selected UTXOs, so it can't be used through this interface
        if strategy == SelectionStrategy::CoinControl {
            println!("[DEBUG] CoinControl strategy detected, returning error");
            if let Some(bus) = message_bus {
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "error": "CoinControl strategy requires pre-selected UTXOs",
                        "recommendation": "Use select_coin_control method instead"
                    }).to_string(),
                    MessagePriority::Medium
                );
            }
            
            // Also publish to the domain-specific event bus if available
            let event_bus = utxo_bus.or_else(|| self.utxo_bus.as_ref().map(AsRef::as_ref));
            if let Some(bus) = event_bus {
                bus.publish(UtxoEvent::SelectionFailed {
                    reason: "coin_control_requires_preselected_utxos".to_string(),
                    strategy: "CoinControl".to_string(),
                    target_amount: target.to_sat(),
                    available_amount: 0,
                });
            }
            
            return SelectionResult::InsufficientFunds {
                available: Amount::from_sat(0),
                required: target,
            };
        }
        
        // Log the request if message bus is provided
        if let Some(bus) = message_bus {
            bus.publish(
                EventType::UtxoStatusChanged,
                &json!({
                    "action": "utxo_selection_start",
                    "strategy": format!("{:?}", strategy),
                    "target_amount": target.to_sat(),
                    "available_utxos": self.utxos.len()
                }).to_string(),
                MessagePriority::Low
            );
        }
        
        // Print available UTXOs for debugging
        println!("[DEBUG] Available UTXOs for selection:");
        for (i, utxo) in self.utxos.iter().enumerate() {
            println!("[DEBUG]   {}: amount={}, confirmations={}, frozen={}", 
                    i, utxo.amount, utxo.confirmations, utxo.is_frozen);
        }
        
        // Create a UTXO selector and select UTXOs
        println!("[DEBUG] Calling UtxoSelector with target: {}", target);
        let selector = UtxoSelector::new();
        
        // Use the provided utxo_bus or fall back to the instance's utxo_bus
        let event_bus = utxo_bus.or_else(|| self.utxo_bus.as_ref().map(AsRef::as_ref));
        
        selector.select_utxos(
            &self.utxos,
            target,
            strategy,
            message_bus,
            event_bus,
        )
    }

    /// Enhanced selection method that provides more flexibility with events 
    ///
    /// # Arguments
    /// * `target` - The target amount to send
    /// * `strategy` - The strategy to use for selecting UTXOs
    /// * `message_bus` - Optional message bus for emitting events
    /// * `create_event_bus` - Whether to create a new event bus if one doesn't exist
    ///
    /// # Returns
    /// * `(SelectionResult, Option<Arc<UtxoEventBus>>)` - The result and the event bus if created
    pub fn select_utxos_with_events(
        &self,
        target: Amount,
        strategy: SelectionStrategy,
        message_bus: Option<&MessageBus>,
        create_event_bus: bool,
    ) -> (SelectionResult, Option<Arc<UtxoEventBus>>) {
        // If we already have an event bus, use it
        if let Some(ref existing_bus) = self.utxo_bus {
            let result = self.select_utxos(target, strategy, message_bus, Some(existing_bus));
            return (result, None); // No new bus created
        }
        
        // Create a new event bus if requested and none exists
        if create_event_bus {
            // Create a new UTXO event bus
            let utxo_bus = Arc::new(UtxoEventBus::new());
            
            // Create a selector and run the selection
            let selector = UtxoSelector::new();
            let result = selector.select_utxos(
                &self.utxos,
                target,
                strategy,
                message_bus,
                Some(utxo_bus.as_ref()),
            );
            
            return (result, Some(utxo_bus));
        } else {
            // Just run the regular selection without an event bus
            let result = self.select_utxos(target, strategy, message_bus, None);
            (result, None)
        }
    }

    /// Selects UTXOs for coin control based on the given outpoints and target amount.
    ///
    /// # Arguments
    /// * `selected_outpoints` - The outpoints of UTXOs that the user wants to use
    /// * `target` - The target amount to send
    /// * `message_bus` - Optional message bus for emitting events
    /// * `utxo_bus` - Optional UTXO event bus for emitting events
    ///
    /// # Returns
    /// * `SelectionResult` - The result of the selection process
    pub fn select_coin_control(&self, selected_outpoints: &[OutPoint], target: Amount, message_bus: Option<&MessageBus>, utxo_bus: Option<&UtxoEventBus>) -> SelectionResult {
        println!("[DEBUG] select_coin_control called with target: {}, outpoints: {:?}", target, selected_outpoints);
        
        // Log the selection request if message bus is provided
        if let Some(bus) = message_bus {
            bus.publish(
                EventType::UtxoStatusChanged,
                &json!({
                    "action": "coin_control_selection_start",
                    "outpoints_count": selected_outpoints.len(),
                    "target_amount": target.to_sat(),
                }).to_string(),
                MessagePriority::Low
            );
        }
        
        // Find the UTXOs corresponding to the selected outpoints
        let selected_utxos: Vec<Utxo> = selected_outpoints.iter()
            .filter_map(|outpoint| {
                let utxo = self.utxos.iter().find(|u| u.outpoint == *outpoint);
                if utxo.is_none() && message_bus.is_some() {
                    println!("[DEBUG] UTXO not found for outpoint: {:?}", outpoint);
                    // Log each missing UTXO
                    if let Some(bus) = message_bus {
                        bus.publish(
                            EventType::UtxoStatusChanged,
                            &json!({
                                "warning": "UTXO not found",
                                "outpoint": outpoint.to_string(),
                            }).to_string(),
                            MessagePriority::Medium
                        );
                    }
                }
                utxo.cloned()
            })
            .collect();
        
        // Print selected UTXOs for debugging
        println!("[DEBUG] Selected UTXOs:");
        for (i, utxo) in selected_utxos.iter().enumerate() {
            println!("[DEBUG]   {}: amount={}, confirmations={}, frozen={}", 
                    i, utxo.amount, utxo.confirmations, utxo.is_frozen);
        }
        
        // Check if we have found all the requested UTXOs
        if selected_utxos.len() != selected_outpoints.len() {
            println!("[DEBUG] Warning: Some requested UTXOs were not found");
            println!("[DEBUG]   requested: {}, found: {}, missing: {}", 
                    selected_outpoints.len(), selected_utxos.len(), 
                    selected_outpoints.len() - selected_utxos.len());
            
            // Some UTXOs were not found, log a warning and continue with what we have
            if let Some(bus) = message_bus {
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "warning": "Some requested UTXOs were not found",
                        "requested": selected_outpoints.len(),
                        "found": selected_utxos.len(),
                        "missing": selected_outpoints.len() - selected_utxos.len(),
                    }).to_string(),
                    MessagePriority::Medium
                );
            }
        }
        
        // Check if any selected UTXOs are frozen
        let frozen_utxos: Vec<&Utxo> = selected_utxos.iter()
            .filter(|u| u.is_frozen)
            .collect();
            
        if !frozen_utxos.is_empty() {
            println!("[DEBUG] Error: Some selected UTXOs are frozen");
            println!("[DEBUG]   frozen_count: {}", frozen_utxos.len());
            
            // Log a warning about frozen UTXOs
            if let Some(bus) = message_bus {
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "warning": "Some selected UTXOs are frozen",
                        "frozen_count": frozen_utxos.len(),
                        "frozen_outpoints": frozen_utxos.iter().map(|u| u.outpoint.to_string()).collect::<Vec<String>>(),
                    }).to_string(),
                    MessagePriority::High
                );
            }
            
            // Return insufficient funds with a note
            return SelectionResult::InsufficientFunds {
                available: Amount::from_sat(0),
                required: target,
            };
        }
        
        if selected_utxos.is_empty() {
            println!("[DEBUG] Error: No valid UTXOs found for coin control");
            
            // Log the error
            if let Some(bus) = message_bus {
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "error": "No valid UTXOs found for coin control",
                        "requested": selected_outpoints.len(),
                    }).to_string(),
                    MessagePriority::High
                );
            }
            
            return SelectionResult::InsufficientFunds {
                available: Amount::from_sat(0),
                required: target,
            };
        }
        
        // Calculate total amount in selected UTXOs
        let total_selected: Amount = selected_utxos.iter()
            .map(|u| u.amount)
            .sum();
        
        println!("[DEBUG] Total selected amount: {}", total_selected);
            
        // Check if we have enough total value
        if total_selected < target {
            println!("[DEBUG] Error: Insufficient funds in selected UTXOs");
            println!("[DEBUG]   available: {}, required: {}, missing: {}", 
                    total_selected, target, target.to_sat() - total_selected.to_sat());
            
            // Log the insufficient funds error
            if let Some(bus) = message_bus {
                bus.publish(
                    EventType::UtxoStatusChanged,
                    &json!({
                        "error": "Insufficient funds in selected UTXOs",
                        "available": total_selected.to_sat(),
                        "required": target.to_sat(),
                        "missing": target.to_sat() - total_selected.to_sat(),
                    }).to_string(),
                    MessagePriority::High
                );
            }
            
            return SelectionResult::InsufficientFunds {
                available: total_selected,
                required: target,
            };
        }
        
        // Create a UTXO selector 
        let selector = UtxoSelector::new();
        
        // Instead of manually calculating fees, etc. let the selector do it
        // This ensures consistent fee calculation logic
        let result = selector.select_coin_control(
            &selected_utxos, 
            target, 
            message_bus,
            utxo_bus.or_else(|| self.utxo_bus.as_ref().map(AsRef::as_ref)),
        );
        
        // Log results
        if let Some(bus) = message_bus {
            match &result {
                SelectionResult::Success { selected, fee_amount, change_amount } => {
                    bus.publish(
                        EventType::UtxoStatusChanged,
                        &json!({
                            "success": "Coin control selection completed",
                            "selected_count": selected.len(),
                            "total_amount": selected_utxos.iter().map(|u| u.amount.to_sat()).sum::<u64>(),
                            "fee_amount": fee_amount.to_sat(),
                            "change_amount": change_amount.to_sat(),
                        }).to_string(),
                        MessagePriority::Low
                    );
                },
                SelectionResult::InsufficientFunds { available, required } => {
                    bus.publish(
                        EventType::UtxoStatusChanged,
                        &json!({
                            "error": "Insufficient funds after fee calculation",
                            "available": available.to_sat(),
                            "required": required.to_sat(),
                            "missing": required.to_sat() - available.to_sat(),
                        }).to_string(),
                        MessagePriority::High
                    );
                }
            }
        }
        
        result
    }

    /// Convenience method to select UTXOs using the MinimizeFee strategy
    pub fn select_utxos_minimize_fee(&self, target: Amount, message_bus: Option<&MessageBus>) -> SelectionResult {
        self.select_utxos(target, SelectionStrategy::MinimizeFee, message_bus, None)
    }
    
    /// Convenience method to select UTXOs using the MinimizeChange strategy
    pub fn select_utxos_minimize_change(&self, target: Amount, message_bus: Option<&MessageBus>) -> SelectionResult {
        self.select_utxos(target, SelectionStrategy::MinimizeChange, message_bus, None)
    }
    
    /// Convenience method to select UTXOs using the OldestFirst strategy
    pub fn select_utxos_oldest_first(&self, target: Amount, message_bus: Option<&MessageBus>) -> SelectionResult {
        self.select_utxos(target, SelectionStrategy::OldestFirst, message_bus, None)
    }
    
    /// Convenience method to select UTXOs using the PrivacyFocused strategy
    pub fn select_utxos_privacy_focused(&self, target: Amount, message_bus: Option<&MessageBus>) -> SelectionResult {
        self.select_utxos(target, SelectionStrategy::PrivacyFocused, message_bus, None)
    }
    
    /// Convenience method to select UTXOs using the MaximizePrivacy strategy
    pub fn select_utxos_maximize_privacy(&self, target: Amount, message_bus: Option<&MessageBus>) -> SelectionResult {
        self.select_utxos(target, SelectionStrategy::MaximizePrivacy, message_bus, None)
    }

    /// Convenience method to select UTXOs using the Consolidate strategy
    pub fn select_utxos_consolidate(&self, target: Amount, message_bus: Option<&MessageBus>) -> SelectionResult {
        self.select_utxos(target, SelectionStrategy::Consolidate, message_bus, None)
    }
    
    /// Convenience method to select UTXOs using the AvoidChange strategy
    pub fn select_utxos_avoid_change(&self, target: Amount, message_bus: Option<&MessageBus>) -> SelectionResult {
        self.select_utxos(target, SelectionStrategy::AvoidChange, message_bus, None)
    }

    pub fn set_utxo_message_bus(&mut self, utxo_bus: Arc<UtxoEventBus>) {
        if let Some(ref _existing_bus) = self.utxo_bus {
            // Already initialized
            return;
        }
        
        self.utxo_bus = Some(utxo_bus);
    }
} 