//! UTXO selection module
//!
//! This module provides a modular, event-driven approach to UTXO (Unspent Transaction Output) selection
//! for Bitcoin transactions in the BitVault wallet.
//!
//! # Overview
//!
//! UTXO selection is a critical component of any Bitcoin wallet. It determines which
//! unspent transaction outputs to use when creating a new transaction. The selection
//! strategy impacts:
//!
//! - Transaction fees
//! - Privacy characteristics
//! - UTXO consolidation/fragmentation
//! - Change output amounts
//!
//! # Module Structure
//!
//! The UTXO selection module is organized as follows:
//!
//! - `types.rs` - Core data structures and types for UTXO selection
//! - `selector.rs` - Main UTXO selector implementation (Strategy pattern context)
//! - `strategies/` - Strategy implementations for different selection algorithms
//!   - `minimize_fee.rs` - Minimize transaction fee strategy
//!   - `minimize_change.rs` - Minimize change amount strategy
//!   - `oldest_first.rs` - Select oldest UTXOs first
//!   - `privacy_focused.rs` - Enhance privacy by avoiding address reuse
//!   - `maximize_privacy.rs` - Maximize privacy by using more inputs
//!   - `consolidate.rs` - Consolidate UTXOs to reduce fragmentation
//!   - `avoid_change.rs` - Try to avoid change outputs completely
//!
//! # Event-Driven Architecture
//!
//! The UTXO selection module integrates with the BitVault event system in several ways:
//!
//! - `UtxoSelector` publishes events via both general and domain-specific event buses
//! - Events are published at key points in the selection process:
//!   - When selection begins
//!   - When selection completes successfully
//!   - When selection fails due to insufficient funds
//!
//! ## Integration Methods
//!
//! Several approaches are available for event integration:
//!
//! - Direct integration via `select_utxos` with message bus parameters
//! - Domain-specific event bus via `select_utxos` with `UtxoEventBus` parameter
//! - Automatic event bus creation via `select_utxos_with_events`
//! - Factory methods like `with_event_bus` and `with_fee_rate_and_event_bus`
//!
//! # Feature Highlights
//!
//! - **Strategy Pattern**: Easily swap selection algorithms without changing client code
//! - **Event Publication**: Monitor UTXO selection process via events
//! - **Flexible Fee Control**: Adjust fee rates to match network conditions
//! - **Multiple Selection Algorithms**: Choose the right strategy for each transaction
//! - **Privacy Enhancements**: Strategies designed for improved transaction privacy
//!
//! # Typical Usage
//!
//! ```no_run
//! use bitvault_common::utxo_selection::selector::UtxoSelector;
//! use bitvault_common::utxo_selection::types::{SelectionStrategy, SelectionResult};
//! use bitvault_common::utxo_selection::types::Utxo;
//! use bitvault_common::events::{MessageBus, UtxoEventBus};
//! use bitcoin::{Amount, OutPoint, Txid};
//! use std::sync::Arc;
//! use std::str::FromStr;
//!
//! // Create some sample UTXOs
//! let utxos = vec![
//!     Utxo::new(
//!         OutPoint::new(Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(), 0),
//!         Amount::from_sat(50_000),
//!         6, // confirmations
//!         false, // is_change
//!     )
//! ];
//!
//! // Create a selector with event bus integration
//! let (selector, event_bus) = UtxoSelector::with_event_bus(Arc::new(UtxoEventBus::new()));
//!
//! // Select UTXOs using the minimize fee strategy
//! let result = selector.select_utxos(
//!     &utxos,
//!     Amount::from_sat(50_000),
//!     SelectionStrategy::MinimizeFee,
//!     None,
//!     Some(event_bus.as_ref()),
//! );
//!
//! // Process the result
//! match result {
//!     SelectionResult::Success { selected, fee_amount, change_amount } => {
//!         // Use selected UTXOs to create a transaction
//!     },
//!     SelectionResult::InsufficientFunds { available, required } => {
//!         // Handle insufficient funds case
//!     }
//! }
//! ```
//!
//! # Security Considerations
//!
//! The UTXO selection module handles sensitive wallet information:
//!
//! - Available UTXOs represent funds controlled by the wallet
//! - Selection strategies can affect transaction privacy
//! - Events published during selection contain information about wallet state
//!
//! ## Security Boundary Documentation
//!
//! This module implements several important security boundaries:
//!
//! 1. **UTXO/Transaction Boundary**: Separates UTXO management from transaction creation
//!    - UTXO selection precedes transaction building but must be isolated from it
//!    - Selected UTXOs are passed across this boundary with minimal additional info
//!    - Ensures that transaction creation cannot accidentally modify UTXO state
//!
//! 2. **Strategy/Core Boundary**: Isolates selection strategies from core selection logic
//!    - Strategies receive read-only access to UTXOs
//!    - Results from strategies are validated before being used
//!    - Prevents strategy implementation bugs from corrupting wallet state
//!
//! 3. **Event Publication Boundary**: Controls information flow to observers
//!    - Published events must be sanitized to avoid leaking sensitive data
//!    - Events should contain only minimal necessary information
//!    - Different event types are used based on security context of receivers
//!
//! Care should be taken to ensure that events published by the selector do not
//! leak sensitive information across security boundaries. Implementations should
//! sanitize event payloads and use appropriate event types when crossing boundaries.

pub mod types;
pub mod selector;
pub mod strategies; 