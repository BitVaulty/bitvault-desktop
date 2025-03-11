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
//!
//! # Security Considerations
//!
//! This library implements various security measures:
//! - Memory protection for sensitive data
//! - Secure logging practices
//! - Type safety for Bitcoin operations
//! - Platform-specific security features
//!
//! This library leverages BDK (Bitcoin Development Kit) for core Bitcoin functionality.

/// Core domain types for BitVault wallet
pub mod types;

/// Bitcoin-related calculations and math utilities
pub mod math;

/// Secure logging functionality
pub mod logging;

/// Platform-specific functionality
pub mod platform;

/// Configuration management
pub mod config;

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
pub use math::{is_dust_amount, min_economical_change};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build timestamp (set at compile time)
pub const BUILD_TIMESTAMP: &str = env!("CARGO_PKG_VERSION");

/// Check if the library was built in debug mode
pub const fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

/// Library initialization
///
/// Sets up any global state required by the library.
///
/// # Returns
/// * Result with () on success, or an error message string
pub fn init() -> Result<(), String> {
    // Initialize logging with default configuration
    let config = logging::LogConfig::default();
    logging::init(&config).map_err(|e| format!("Failed to initialize logging: {}", e))
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
