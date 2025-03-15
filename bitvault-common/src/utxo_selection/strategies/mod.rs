//! UTXO selection strategies
//!
//! This module defines various UTXO selection strategies for different use cases.
//! Each strategy implements the `SelectionStrategy` trait and provides specific
//! selection logic optimized for its use case.

use bitcoin::Amount;
use crate::events::MessageBus;
use crate::utxo_selection::types::{Utxo, SelectionResult};

pub mod base;
pub mod minimize_fee;
pub mod minimize_change;
pub mod oldest_first;
pub mod privacy_focused;
pub mod maximize_privacy;
pub mod consolidate;
pub mod avoid_change;
pub mod utils;

// Re-export implementations
pub use minimize_fee::MinimizeFeeStrategy;
pub use minimize_change::MinimizeChangeStrategy;
pub use oldest_first::OldestFirstStrategy;
pub use privacy_focused::PrivacyFocusedStrategy;
pub use maximize_privacy::MaximizePrivacyStrategy;
pub use consolidate::ConsolidateStrategy;
pub use avoid_change::AvoidChangeStrategy;

/// Trait defining a UTXO selection strategy
///
/// Any struct implementing this trait can be used as a strategy
/// for UTXO selection.
pub trait Strategy {
    /// Name of this strategy
    fn name(&self) -> &'static str;
    
    /// Select UTXOs using this strategy
    ///
    /// # Arguments
    /// * `utxos` - Available UTXOs
    /// * `target_amount` - Target amount to select
    /// * `fee_rate` - Fee rate in satoshis per vByte
    /// * `dust_threshold` - Dust threshold in satoshis
    /// * `message_bus` - Optional message bus for emitting events
    ///
    /// # Returns
    /// * Selection result
    fn select(
        &self,
        utxos: &[Utxo],
        target_amount: Amount,
        fee_rate: f32,
        dust_threshold: u64,
        message_bus: Option<&MessageBus>,
    ) -> SelectionResult;
} 