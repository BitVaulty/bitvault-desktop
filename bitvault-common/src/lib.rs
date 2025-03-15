//! BitVault Common Library
//!
//! This crate provides common functionality for the BitVault Bitcoin wallet
//! including types, utilities, and shared code used across different components.
//!
//! # Modules
//!
//! - `types`: Core domain types and data structures
//! - `math`: Bitcoin-specific mathematical operations
//! - `logging`: Security-aware logging infrastructure
//! - `platform`: Platform-specific functionality
//! - `config`: Configuration management
//! - `config_manager`: Enhanced configuration management with profiles and validation
//! - `events`: Event system for domain-specific and general event publishing/subscribing
//! - `localization`: Internationalization and localization support
//! - `wallet_operations`: Wallet operations using BDK
//! - `address_book`: Bitcoin address book functionality
//! - `utxo_selection`: UTXO selection algorithms and utilities with event-driven architecture
//! - `utxo_management`: UTXO management functionality with event-driven capabilities
//! - `network_status`: Bitcoin network status tracking utilities
//! - `fee_estimation`: Fee estimation utilities
//! - `key_management`: Key management functionality for secure handling of wallet keys
//!
//! # Security Considerations
//!
//! This library implements various security measures:
//! - Memory protection for sensitive data
//! - Secure logging practices
//! - Type safety for Bitcoin operations
//! - Platform-specific security features
//! - Event-driven architecture for better encapsulation of sensitive components
//!
//! This library leverages BDK (Bitcoin Development Kit) for core Bitcoin functionality.

/// Core domain types for BitVault wallet
pub mod types;

/// Bitcoin-related calculations and math utilities
pub mod math;

/// Secure logging functionality
pub mod logging;

/// Platform-specific functionality
///
/// Provides abstractions for platform-specific operations including:
/// - File paths and directory handling
/// - Secure storage locations
/// - Memory protection
/// - OS-specific security features
///
/// This module uses a provider-based architecture that allows for:
/// - Platform-specific implementations with a common interface
/// - Runtime detection of platform capabilities
/// - Testing with mock implementations
/// - Feature flags for optional platform-specific features
pub mod platform;

/// Configuration management
pub mod config;

/// Enhanced configuration management with profiles and validation
pub mod config_manager;

/// Event system for domain-specific and general event publishing/subscribing
///
/// Implements an event-driven architecture pattern with:
/// - General message bus for system-wide events
/// - Domain-specific event buses for targeted functionality
/// - Asynchronous communication between components
/// - Publisher/subscriber pattern
pub mod events;

/// Internationalization and localization support
pub mod localization;

/// Wallet operations using BDK
pub mod wallet_operations;

/// Address book functionality
pub mod address_book;

/// UTXO selection algorithms and utilities (modular implementation)
///
/// Implements the Strategy pattern for UTXO selection with event-driven integration:
/// - Multiple selection strategies (minimize fee, maximize privacy, etc.)
/// - Event publishing for operation monitoring
/// - Domain-specific event bus for UTXO-related events
pub mod utxo_selection;

/// Original UTXO selection implementation (legacy)
/// 
/// @deprecated This is kept for backward compatibility but will be removed in a future version.
/// Use the modular utxo_selection instead.
///
/// This module is not meant to be used directly. Instead, use the types and utilities
/// re-exported at the crate root or from the `utxo_selection` module.
#[deprecated(since = "0.2.0", note = "Use the modular utxo_selection module instead")]
mod utxo_selection_orig;

/// UTXO selection fixed implementation (legacy)
/// 
/// @deprecated This is kept for backward compatibility but will be removed in a future version.
/// Use the modular utxo_selection instead.
#[deprecated(since = "0.2.0", note = "Use the modular utxo_selection module instead")]
mod utxo_selection_fixed;

/// UTXO selection v2 (future implementation)
#[doc(hidden)]
pub mod utxo_selection_v2;

/// UTXO management functionality
///
/// Manages the lifecycle of UTXOs with event-driven capabilities:
/// - UTXO tracking and status management
/// - Integration with UTXO selection strategies
/// - Event publishing for wallet state changes
/// - Domain-specific event handling for UTXO operations
pub mod utxo_management;

/// Bitcoin network status tracking utilities
pub mod network_status;

/// Fee estimation utilities
pub mod fee_estimation;

/// Key management functionality for secure handling of wallet keys
pub mod key_management;

/// Re-export address book types
pub use address_book::{AddressBook, AddressEntry, AddressCategory};

/// Re-export UTXO selection types and utilities
pub use utxo_selection::types::{
    Utxo, UtxoSet, SelectionStrategy, SelectionResult,
};
pub use utxo_selection::selector::UtxoSelector;

/// Backward compatibility re-exports for utxo_selection module
#[deprecated(since = "0.2.0", note = "Use imports from utxo_selection sub-modules instead")]
pub mod utxo_selection_compat {
    pub use crate::utxo_selection::types::{Utxo, UtxoSet, SelectionStrategy, SelectionResult};
    pub use crate::utxo_selection::selector::UtxoSelector;
}

// Re-export important Bitcoin and BDK types
pub use bdk::{blockchain, wallet, Balance, FeeRate, TransactionDetails};
pub use bitcoin::{Address, Amount, Network, OutPoint, Transaction, Txid};

/// Re-export common types for convenience
pub use types::{
    sanitize_for_display, AddressInfo, FeeEstimates, FeePriority, SensitiveBytes, SensitiveString,
    WalletError, WalletInfo, WalletSettings, WalletTransaction, DUST_THRESHOLD, MAX_BITCOIN_SUPPLY,
    SATS_PER_BTC,
};

/// Re-export math utilities for convenience
pub use math::{
    is_dust_amount, min_economical_change, calculate_fee, estimate_tx_size,
    get_input_size, get_output_size, estimate_tx_size_detailed
};

/// Re-export localization utilities
pub use localization::{format_amount, tr, BitVaultLocale, BitcoinUnit, AmountDisplayOptions};

/// Re-export network status types and utilities
pub use network_status::{
    NetworkStatus,
    CongestionLevel,
    NetworkStatusError,
    NetworkStatusProvider,
    MempoolStatus,
    TransactionConfirmationStatus,
    BlockInfo,
};

// Re-export for testing purposes only
#[cfg(test)]
pub use network_status::MockNetworkStatusProvider;

// Re-export mock platform provider for testing only
#[cfg(test)]
pub use platform::mock::MockPlatformProvider;

// Re-export platform testing utilities
pub use platform::mock;
pub use platform::set_platform_provider;
pub use platform::reset_platform_provider;

/// Re-export fee estimation types and utilities
pub use fee_estimation::{
    FeeEstimationError,
    FeeRecommendations,
    HistoricalFeeData,
    estimate_fee,
    calculate_total_fee,
    adjust_fee_for_congestion,
    create_recommendations,
    create_recommendations_from_provider,
};

/// Re-export event types
pub use events::{
    MessageBus,
    EventType,
    MessagePriority,
    KeyManagementEvent,
    KeyManagementBus,
    UtxoEvent,
    UtxoEventBus,
    OutPointInfo,
};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build timestamp (set at compile time)
pub const BUILD_TIMESTAMP: &str = env!("CARGO_PKG_VERSION");

/// Check if the library was built in debug mode
pub const fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

use std::sync::Once;

// Ensure initialization happens only once
static INIT: Once = Once::new();

/// Library initialization
///
/// Sets up any global state required by the library.
/// This function can be safely called multiple times - it will only
/// initialize once to prevent issues in tests and concurrent environments.
///
/// # Returns
/// * Result with () on success, or an error message string
pub fn init() -> Result<(), String> {
    // Use a thread-local to store initialization result
    thread_local! {
        static INIT_RESULT: std::cell::RefCell<Option<Result<(), String>>> = std::cell::RefCell::new(None);
    }
    
    // Only run initialization once
    let mut needs_init = false;
    INIT.call_once(|| {
        needs_init = true;
    });

    // Perform initialization if needed and store result
    if needs_init {
        let result = {
            // Initialize logging with default configuration
            let config = logging::LogConfig::default();
            logging::init(&config).map_err(|e| format!("Failed to initialize logging: {}", e))
        };
        
        // Store result for future calls
        INIT_RESULT.with(|cell| {
            *cell.borrow_mut() = Some(result.clone());
        });
        
        result
    } else {
        // Return cached result
        INIT_RESULT.with(|cell| {
            cell.borrow().clone().unwrap_or(Ok(()))
        })
    }
}

/// Get supported platform capabilities
///
/// # Returns
/// * Information about what capabilities are available on the current platform
pub fn get_platform_capabilities() -> platform::PlatformCapabilities {
    platform::get_platform_capabilities()
}

/// Bitcoin utility functions for common operations
///
/// This module provides high-level utilities for working with Bitcoin types from
/// the bitcoin and BDK crates. It handles address validation, formatting, and
/// conversions between various units.
///
/// Note: For BTC/satoshi conversions, use bitcoin::Amount directly:
/// - Amount::from_btc(btc).unwrap().to_sat() for BTC to satoshis
/// - Amount::from_sat(sats).to_btc() for satoshis to BTC
pub mod bitcoin_utils {
    use crate::types::WalletError;
    use bitcoin::address::NetworkUnchecked;
    use bitcoin::{Address, Amount, Network, Txid};
    use std::str::FromStr;

    /// Parse and validate a Bitcoin address
    ///
    /// This is a simple wrapper around bitcoin::Address parsing with network validation
    pub fn parse_address(address: &str, network: Network) -> Result<Address, WalletError> {
        // Parse the address string as unchecked first
        let unchecked_addr = Address::<NetworkUnchecked>::from_str(address)
            .map_err(|e| WalletError::InvalidAddress(format!("Invalid address format: {}", e)))?;

        // Check if the address belongs to the expected network
        if unchecked_addr.network != network {
            return Err(WalletError::InvalidNetworkType(format!(
                "Address belongs to {} network, but expected {}",
                unchecked_addr.network, network
            )));
        }

        // Convert to a checked address
        let checked_addr = unchecked_addr.assume_checked();

        Ok(checked_addr)
    }

    /// Check if a string is a valid Bitcoin address for the specified network
    ///
    /// # Examples
    ///
    /// ```
    /// use bitcoin::Network;
    /// use bitvault_common::bitcoin_utils;
    ///
    /// // Check valid addresses
    /// assert!(bitcoin_utils::is_valid_bitcoin_address(
    ///     "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    ///     Network::Bitcoin
    /// ));
    ///
    /// assert!(bitcoin_utils::is_valid_bitcoin_address(
    ///     "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
    ///     Network::Bitcoin
    /// ));
    ///
    /// // Check testnet address with correct network
    /// assert!(bitcoin_utils::is_valid_bitcoin_address(
    ///     "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
    ///     Network::Testnet
    /// ));
    ///
    /// // Wrong network should return false
    /// assert!(!bitcoin_utils::is_valid_bitcoin_address(
    ///     "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
    ///     Network::Bitcoin
    /// ));
    ///
    /// // Invalid addresses should return false
    /// assert!(!bitcoin_utils::is_valid_bitcoin_address(
    ///     "invalid-address",
    ///     Network::Bitcoin
    /// ));
    /// ```
    pub fn is_valid_bitcoin_address(address: &str, network: Network) -> bool {
        Address::<NetworkUnchecked>::from_str(address)
            .map(|addr| addr.network == network)
            .unwrap_or(false)
    }

    /// Validate if a string is a valid transaction ID
    pub fn is_valid_txid(txid: &str) -> bool {
        Txid::from_str(txid).is_ok()
    }

    /// Format an amount with appropriate units (BTC or sats)
    ///
    /// This provides nicer formatting than the default Display implementation
    pub fn format_bitcoin_amount(amount: Amount, as_btc: bool) -> String {
        if as_btc {
            format!("{:.8} BTC", amount.to_btc())
        } else {
            format!("{} sats", amount.to_sat())
        }
    }

    /// Parse a string containing a Bitcoin amount
    ///
    /// Handles both BTC and satoshi denominations from user input
    pub fn parse_bitcoin_amount(s: &str) -> Result<Amount, WalletError> {
        let s = s.trim().to_lowercase();

        if s.is_empty() {
            return Err(WalletError::InvalidAmount(
                "Empty amount string".to_string(),
            ));
        }

        // Try to parse as satoshis if it ends with "sats" or "sat"
        if s.ends_with(" sats") || s.ends_with(" sat") {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.is_empty() {
                return Err(WalletError::InvalidAmount(
                    "Invalid satoshi format".to_string(),
                ));
            }

            match parts[0].parse::<u64>() {
                Ok(sats) => Ok(Amount::from_sat(sats)),
                Err(_) => Err(WalletError::InvalidAmount(format!(
                    "Invalid satoshi value: {}",
                    parts[0]
                ))),
            }
        }
        // Try to parse as BTC if it ends with "btc"
        else if s.ends_with(" btc") || s.ends_with("btc") {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.is_empty() {
                return Err(WalletError::InvalidAmount("Invalid BTC format".to_string()));
            }

            let btc_str = parts[0];
            match btc_str.parse::<f64>() {
                Ok(btc) => {
                    if !btc.is_finite() || btc < 0.0 {
                        return Err(WalletError::InvalidAmount(format!(
                            "Invalid BTC value: {}",
                            btc
                        )));
                    }

                    // Convert BTC to satoshis
                    let sats = (btc * 100_000_000.0).round() as u64;
                    Ok(Amount::from_sat(sats))
                }
                Err(_) => Err(WalletError::InvalidAmount(format!(
                    "Invalid BTC value: {}",
                    btc_str
                ))),
            }
        }
        // Try to parse directly using bitcoin::Amount::from_str
        else {
            match Amount::from_str(&s) {
                Ok(amount) => Ok(amount),
                Err(_) => {
                    // Fall back to assuming it's a decimal BTC value
                    match s.parse::<f64>() {
                        Ok(btc) => {
                            if !btc.is_finite() || btc < 0.0 {
                                return Err(WalletError::InvalidAmount(format!(
                                    "Invalid BTC value: {}",
                                    btc
                                )));
                            }

                            // Convert BTC to satoshis
                            let sats = (btc * 100_000_000.0).round() as u64;
                            Ok(Amount::from_sat(sats))
                        }
                        Err(_) => Err(WalletError::InvalidAmount(format!(
                            "Could not parse amount: {}",
                            s
                        ))),
                    }
                }
            }
        }
    }

    /// Format a fiat amount with proper currency symbol
    pub fn format_fiat(amount: f64, currency: &str, decimals: usize) -> String {
        if !amount.is_finite() {
            return "Invalid amount".to_string();
        }

        let symbol = match currency.to_uppercase().as_str() {
            "USD" => "$",
            "EUR" => "€",
            "GBP" => "£",
            "JPY" => "¥",
            "CNY" | "RMB" => "¥",
            "KRW" => "₩",
            "BRL" => "R$",
            "INR" => "₹",
            _ => "",
        };

        // Different formatting for JPY and other non-decimal currencies
        if currency.to_uppercase() == "JPY" || currency.to_uppercase() == "KRW" {
            // These currencies typically don't show decimal places
            let formatted = format!("{}", amount.round() as i64);

            if symbol.is_empty() {
                format!("{} {}", formatted, currency)
            } else {
                format!("{}{}", symbol, formatted)
            }
        } else {
            let formatted = format!("{:.*}", decimals, amount);

            if symbol.is_empty() {
                format!("{} {}", formatted, currency)
            } else {
                format!("{}{}", symbol, formatted)
            }
        }
    }

    /// Calculate the fiat value of a Bitcoin amount based on exchange rate
    pub fn calculate_fiat_value(amount: Amount, exchange_rate: f64) -> f64 {
        if !exchange_rate.is_finite() || exchange_rate < 0.0 {
            return 0.0;
        }
        amount.to_btc() * exchange_rate
    }
}

/// Version information for the common library
pub mod version {
    use super::VERSION;

    /// Get the version string
    pub fn get_version() -> String {
        VERSION.to_string()
    }
}

// No test modules declared here - integration tests are in the tests/ directory

// Backward compatibility wrapper for existing code
#[doc(hidden)]
pub mod compat {
    // UTXO selection types and utilities
    pub use crate::utxo_selection::types::{Utxo, UtxoSet, SelectionStrategy, SelectionResult};
    pub use crate::utxo_selection::selector::UtxoSelector;
    
    pub use crate::address_book::{AddressBook, AddressEntry, AddressCategory};
    
    // This implementation will be used if old code tries to call add_entry with 4 arguments
    impl crate::address_book::AddressBook {
        #[allow(unused_variables)]
        pub fn add_entry_compat(
            &mut self,
            address: &str,
            label: &str,
            notes: Option<&str>,
            category: crate::address_book::AddressCategory,
            message_bus: Option<&dyn std::any::Any>,
        ) -> Result<(), crate::types::WalletError> {
            self.add_entry_simple(address, label, notes, category)
        }
    }
}

#[cfg(test)]
mod address_book_test {
    use crate::address_book::{AddressBook, AddressCategory};
    use bitcoin::Network;
    
    #[test]
    fn address_book() {
        // Create a new address book for mainnet
        let mut address_book = AddressBook::new(Network::Bitcoin);
        
        // Add an entry
        address_book.add_entry_simple(
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
            "Satoshi Donation",
            Some("First ever Bitcoin address"),
            AddressCategory::Donation
        ).expect("Failed to add entry");
        
        // Verify the entry was added
        let entry = address_book.get_entry("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa").expect("Entry not found");
        assert_eq!(entry.label, "Satoshi Donation");
    }
}

#[cfg(test)]
mod utxo_selection_test {
    // Import directly from the crate root where these are re-exported
    use crate::{Utxo, UtxoSelector, SelectionStrategy, SelectionResult};
    use bitcoin::{Amount, Network, OutPoint, Txid};
    use std::str::FromStr;
    
    #[test]
    fn utxo_selection_orig() {
        // Simple test to verify imports
        let selector = UtxoSelector::new();
        let strategy = SelectionStrategy::MinimizeFee;
        
        // Create a test UTXO with a valid txid
        let txid = Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let outpoint = OutPoint::new(txid, 0);
        let amount = Amount::from_sat(10000);
        let _utxo = Utxo::new(outpoint, amount, 0, false);
        
        // Just testing that types can be used correctly
        let _result: Option<SelectionResult> = None;
    }
}

// Re-export platform types and functions for convenience
pub use platform::{
    get_platform_type, 
    platform,
    PlatformError,
    PlatformResult
};

// Re-export platform types directly from their modules
pub use platform::types::PlatformType;
pub use platform::capabilities::PlatformCapabilities;