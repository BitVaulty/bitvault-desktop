//! Common data types for BitVault wallet
//!
//! These types represent the shared data structures used across different
//! components of the wallet. They are designed to be serializable and
//! to NOT contain sensitive cryptographic material.
//!
//! # Security Boundaries
//!
//! These types are designed to be safely passed across security boundaries:
//! - Between UI and wallet logic
//! - Between persistent storage and memory
//! - For IPC communication between processes
//!
//! IMPORTANT: These types MUST NOT contain private keys, seeds, or other sensitive material.

use bitcoin::{Address, Network};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;
use std::str::FromStr;
use thiserror::Error;
use zeroize::Zeroize;
// Import address module for its error type
use anyhow::Result;
use bdk::FeeRate;
use bdk::TransactionDetails;
use bitcoin::address;

// Constants for Bitcoin-specific values

/// Constant for dust threshold (minimum output value)
pub const DUST_THRESHOLD: u64 = 546;

/// Constant for satoshis per Bitcoin
pub const SATS_PER_BTC: u64 = 100_000_000;

/// Constant for maximum Bitcoin supply in satoshis
pub const MAX_BITCOIN_SUPPLY: u64 = 21_000_000 * SATS_PER_BTC;

/// A string that contains sensitive data that should be zeroed when dropped
///
/// # Security
///
/// This type ensures that sensitive data like private keys, seeds, or passwords
/// is automatically zeroed in memory when the value is dropped, reducing the risk
/// of sensitive data leaking through memory dumps or swapping.
///
/// # Examples
///
/// ```
/// use bitvault_common::types::SensitiveString;
///
/// let password = SensitiveString::new("my_secure_password");
/// // password is automatically zeroed when it goes out of scope
/// ```
#[derive(Zeroize)]
pub struct SensitiveString {
    inner: String,
}

impl SensitiveString {
    /// Create a new SensitiveString
    ///
    /// # Arguments
    /// * `s` - The sensitive string to protect
    ///
    /// # Returns
    /// A new SensitiveString instance
    pub fn new(s: impl Into<String>) -> Self {
        Self { inner: s.into() }
    }

    /// Get a reference to the inner string
    ///
    /// # Security
    ///
    /// Be careful with this method as it allows access to the sensitive data.
    /// Only use it when absolutely necessary and ensure the returned reference
    /// is not persisted or logged.
    ///
    /// # Returns
    /// A reference to the sensitive string
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Expose the secret value (alias for as_str for API compatibility)
    ///
    /// # Security
    ///
    /// Be careful with this method as it allows access to the sensitive data.
    /// Only use it when absolutely necessary and ensure the returned reference
    /// is not persisted or logged.
    ///
    /// # Returns
    /// A reference to the sensitive string
    pub fn expose_secret(&self) -> &str {
        self.as_str()
    }

    /// Clear the sensitive data
    ///
    /// This method zeroes out the string and leaves it empty
    pub fn clear(&mut self) {
        self.inner.zeroize();
    }

    /// Get the length of the string
    ///
    /// This is safe to use as it doesn't expose the content.
    ///
    /// # Returns
    /// The length of the string in bytes
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the string is empty
    ///
    /// # Returns
    /// true if the string is empty, false otherwise
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// Manual implementation for Clone
impl Clone for SensitiveString {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Manual implementation for Debug
impl std::fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SensitiveString([REDACTED], length={})", self.len())
    }
}

// Manual implementations for PartialEq and Eq
impl PartialEq for SensitiveString {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl Eq for SensitiveString {}

// Manual implementations for Serialize and Deserialize
impl Serialize for SensitiveString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SensitiveString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl From<String> for SensitiveString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SensitiveString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl fmt::Display for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

// Implement manual Drop for SensitiveString to ensure zeroization
impl Drop for SensitiveString {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

/// Byte array that contains sensitive data that should be zeroed when dropped
///
/// # Security
///
/// This type ensures that sensitive binary data like private keys, seeds, or
/// other cryptographic material is automatically zeroed in memory when the value
/// is dropped, reducing the risk of sensitive data leaking through memory dumps
/// or swapping.
///
/// # Examples
///
/// ```
/// use bitvault_common::types::SensitiveBytes;
///
/// let key_data = SensitiveBytes::new(vec![0x12, 0x34, 0x56, 0x78]);
/// // key_data is automatically zeroed when it goes out of scope
/// ```
#[derive(Zeroize)]
pub struct SensitiveBytes {
    inner: Vec<u8>,
}

impl SensitiveBytes {
    /// Create a new SensitiveBytes
    ///
    /// # Arguments
    /// * `bytes` - The sensitive bytes to protect
    ///
    /// # Returns
    /// A new SensitiveBytes instance
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            inner: bytes.into(),
        }
    }

    /// Get a reference to the inner bytes
    ///
    /// # Security
    ///
    /// Be careful with this method as it allows access to the sensitive data.
    /// Only use it when absolutely necessary and ensure the returned reference
    /// is not persisted or logged.
    ///
    /// # Returns
    /// A reference to the sensitive bytes
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }

    /// Expose the secret value (alias for as_slice for API compatibility)
    ///
    /// # Security
    ///
    /// Be careful with this method as it allows access to the sensitive data.
    /// Only use it when absolutely necessary and ensure the returned reference
    /// is not persisted or logged.
    ///
    /// # Returns
    /// A reference to the sensitive bytes
    pub fn expose_secret(&self) -> &[u8] {
        self.as_slice()
    }

    /// Alias for as_slice() to maintain API compatibility
    pub fn as_bytes(&self) -> &[u8] {
        self.as_slice()
    }

    /// Clear the sensitive data
    ///
    /// This method zeroes out the bytes and leaves the collection empty
    pub fn clear(&mut self) {
        self.inner.zeroize();
    }

    /// Get a mutable reference to the inner bytes
    ///
    /// # Security
    ///
    /// This method provides mutable access to the sensitive data.
    /// Use with extreme caution and only when absolutely necessary.
    ///
    /// # Returns
    /// A mutable reference to the sensitive bytes
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Alias for as_mut_slice() to maintain API compatibility
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }

    /// Get the length of the byte array
    ///
    /// This is safe to use as it doesn't expose the content.
    ///
    /// # Returns
    /// The length of the byte array
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the byte array is empty
    ///
    /// # Returns
    /// true if the byte array is empty, false otherwise
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Convert to a hexadecimal string representation
    ///
    /// # Security
    ///
    /// This method returns a new String containing the hex representation
    /// of the sensitive data. The returned String does not automatically
    /// get zeroed and should be handled with appropriate care.
    ///
    /// # Returns
    /// A hexadecimal string representation of the bytes
    pub fn to_hex(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut s = String::with_capacity(self.len() * 2);
        for byte in self.inner.iter() {
            write!(s, "{:02x}", byte).expect("Writing to string should not fail");
        }
        s
    }

    /// Convert to a sanitized string suitable for logging
    ///
    /// # Returns
    /// A sanitized string with most of the content redacted
    pub fn to_sanitized_string(&self) -> String {
        if self.is_empty() {
            return "[empty]".to_string();
        }

        // Show the first 2 and last 2 bytes if the data is at least 6 bytes long
        if self.len() >= 6 {
            format!(
                "{}...{} [{} bytes]",
                hex::encode(&self.inner[0..2]),
                hex::encode(&self.inner[self.len() - 2..]),
                self.len()
            )
        } else {
            // For short data, just show length to avoid leaking too much
            format!("[{} bytes]", self.len())
        }
    }
}

// Manual implementation for Clone
impl Clone for SensitiveBytes {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Manual implementation for Debug
impl std::fmt::Debug for SensitiveBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SensitiveBytes([REDACTED], length={})", self.len())
    }
}

// Manual implementations for PartialEq and Eq
impl PartialEq for SensitiveBytes {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl Eq for SensitiveBytes {}

// Manual implementations for Serialize and Deserialize
impl Serialize for SensitiveBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SensitiveBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        Ok(Self::new(bytes))
    }
}

impl From<Vec<u8>> for SensitiveBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

impl From<&[u8]> for SensitiveBytes {
    fn from(bytes: &[u8]) -> Self {
        Self::new(bytes.to_vec())
    }
}

impl fmt::Display for SensitiveBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

// Implement manual Drop for SensitiveBytes to ensure zeroization
impl Drop for SensitiveBytes {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

/// Extended Bitcoin address with additional metadata
///
/// This type wraps the BDK/Bitcoin address type and adds additional metadata
/// like labels, derivation paths, and ownership information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressInfo {
    /// The underlying Bitcoin address
    pub address: Address,

    /// Optional label for the address
    pub label: Option<String>,

    /// Path from the HD wallet root (for owned addresses)
    pub derivation_path: Option<String>,

    /// Whether this address belongs to the wallet
    pub is_owned: bool,
}

// Custom serialization implementation for AddressInfo to handle Address type
impl Serialize for AddressInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Create a helper struct for serialization
        #[derive(Serialize)]
        struct AddressInfoHelper {
            address_str: String,
            network: Network,
            label: Option<String>,
            derivation_path: Option<String>,
            is_owned: bool,
        }

        let helper = AddressInfoHelper {
            address_str: self.address.to_string(),
            network: self.address.network,
            label: self.label.clone(),
            derivation_path: self.derivation_path.clone(),
            is_owned: self.is_owned,
        };

        helper.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AddressInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Create a helper struct for deserialization
        #[derive(Deserialize)]
        struct AddressInfoHelper {
            address_str: String,
            network: Network,
            label: Option<String>,
            derivation_path: Option<String>,
            is_owned: bool,
        }

        let helper = AddressInfoHelper::deserialize(deserializer)?;

        // Parse the address
        let unchecked_address = match Address::from_str(&helper.address_str) {
            Ok(addr) => {
                // Verify network matches
                if addr.network != helper.network {
                    return Err(serde::de::Error::custom(format!(
                        "Address network mismatch: got {:?}, expected {:?}",
                        addr.network, helper.network
                    )));
                }
                addr
            }
            Err(e) => return Err(serde::de::Error::custom(format!("Invalid address: {}", e))),
        };

        // Convert to checked address
        let address = match unchecked_address.require_network(helper.network) {
            Ok(addr) => addr,
            Err(e) => {
                return Err(serde::de::Error::custom(format!(
                    "Network validation error: {}",
                    e
                )))
            }
        };

        Ok(Self {
            address,
            label: helper.label,
            derivation_path: helper.derivation_path,
            is_owned: helper.is_owned,
        })
    }
}

impl AddressInfo {
    /// Creates a new address info from a Bitcoin address
    pub fn new(address: Address, is_owned: bool) -> Self {
        Self {
            address,
            label: None,
            derivation_path: None,
            is_owned,
        }
    }

    /// Creates a new address with validation
    pub fn new_validated(address_str: &str, network: Network) -> Result<Self, WalletError> {
        // Parse as unchecked first
        let unchecked_address = Address::from_str(address_str)
            .map_err(|e| WalletError::InvalidAddress(e.to_string()))?;

        // Verify network matches
        if unchecked_address.network != network {
            return Err(WalletError::InvalidNetworkType(format!(
                "Address is for {:?} but expected {:?}",
                unchecked_address.network, network
            )));
        }

        // Convert to checked address
        let address = unchecked_address
            .require_network(network)
            .map_err(|e| WalletError::InvalidNetworkType(e.to_string()))?;

        Ok(Self::new(address, false))
    }

    /// Creates a new address that is owned by this wallet
    pub fn new_owned(address: Address, derivation_path: String) -> Self {
        Self {
            address,
            label: None,
            derivation_path: Some(derivation_path),
            is_owned: true,
        }
    }

    /// Add a label to the address
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Get a sanitized string representation for logging
    pub fn to_sanitized_string(&self) -> String {
        let addr_str = self.address.to_string();
        if addr_str.len() <= 12 {
            return addr_str;
        }

        let prefix = &addr_str[0..6];
        let suffix = &addr_str[addr_str.len() - 6..];
        format!("{}...{}", prefix, suffix)
    }

    /// Get the label if any
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Get the derivation path if any
    pub fn derivation_path(&self) -> Option<&str> {
        self.derivation_path.as_deref()
    }

    /// Check if this address is owned by the wallet
    pub fn is_owned(&self) -> bool {
        self.is_owned
    }
}

impl fmt::Display for AddressInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(label) = &self.label {
            write!(f, "{} ({})", self.address, label)
        } else {
            write!(f, "{}", self.address)
        }
    }
}

/// Transaction status in the wallet
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction is pending confirmation
    Pending,
    /// Transaction is confirmed with this many confirmations
    Confirmed(u32),
    /// Transaction failed or was rejected
    Failed,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Confirmed(confirms) => {
                if *confirms == 1 {
                    write!(f, "Confirmed (1 confirmation)")
                } else {
                    write!(f, "Confirmed ({} confirmations)", confirms)
                }
            }
            TransactionStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// Extended transaction details for UI presentation and history tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletTransaction {
    /// The underlying Bitcoin transaction details from BDK
    pub details: TransactionDetails,

    /// Transaction memo/note (user supplied)
    pub memo: Option<String>,

    /// Addresses involved in this transaction (for UI display)
    pub addresses: Vec<AddressInfo>,

    /// Timestamp of the transaction (in seconds since Unix epoch)
    pub timestamp: u64,
}

impl WalletTransaction {
    /// Check if this transaction is incoming (receiving funds)
    pub fn is_incoming(&self) -> bool {
        self.details.received > self.details.sent
    }

    /// Check if this transaction is outgoing (sending funds)
    pub fn is_outgoing(&self) -> bool {
        self.details.sent >= self.details.received
    }

    /// Get the absolute amount (ignoring direction)
    pub fn absolute_amount(&self) -> u64 {
        if self.is_incoming() {
            self.details.received - self.details.sent
        } else {
            self.details.sent - self.details.received
        }
    }
}

/// Fee priority levels
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeePriority {
    /// Low priority (several hours)
    Low,
    /// Medium priority (within an hour)
    Medium,
    /// High priority (next block)
    High,
}

impl fmt::Display for FeePriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeePriority::Low => write!(f, "Low"),
            FeePriority::Medium => write!(f, "Medium"),
            FeePriority::High => write!(f, "High"),
        }
    }
}

/// Fee estimation targets
#[derive(Clone, Debug)]
pub struct FeeEstimates {
    /// Low priority (several hours to confirm)
    pub low: FeeRate,
    /// Medium priority (within an hour)
    pub medium: FeeRate,
    /// High priority (next block)
    pub high: FeeRate,
}

impl FeeEstimates {
    /// Get fee rate for the specified priority level
    pub fn get_fee_rate(&self, priority: FeePriority) -> FeeRate {
        match priority {
            FeePriority::Low => self.low,
            FeePriority::Medium => self.medium,
            FeePriority::High => self.high,
        }
    }
}

// Custom serialization for FeeEstimates
impl Serialize for FeeEstimates {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("FeeEstimates", 3)?;
        state.serialize_field("low", &self.low.as_sat_per_vb())?;
        state.serialize_field("medium", &self.medium.as_sat_per_vb())?;
        state.serialize_field("high", &self.high.as_sat_per_vb())?;
        state.end()
    }
}

// Custom deserialization for FeeEstimates
impl<'de> Deserialize<'de> for FeeEstimates {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct FeeEstimatesHelper {
            low: f32,
            medium: f32,
            high: f32,
        }

        let helper = FeeEstimatesHelper::deserialize(deserializer)?;
        Ok(FeeEstimates {
            low: FeeRate::from_sat_per_vb(helper.low),
            medium: FeeRate::from_sat_per_vb(helper.medium),
            high: FeeRate::from_sat_per_vb(helper.high),
        })
    }
}

/// Descriptor template types for wallet creation/import
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DescriptorTemplate {
    /// Legacy (P2PKH)
    Legacy,
    /// SegWit (P2WPKH)
    SegWit,
    /// Nested SegWit (P2SH-P2WPKH)
    NestedSegWit,
    /// Taproot (P2TR)
    Taproot,
    /// Custom descriptor template
    Custom(String),
}

impl fmt::Display for DescriptorTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DescriptorTemplate::Legacy => write!(f, "Legacy (P2PKH)"),
            DescriptorTemplate::SegWit => write!(f, "Native SegWit (P2WPKH)"),
            DescriptorTemplate::NestedSegWit => write!(f, "Nested SegWit (P2SH-P2WPKH)"),
            DescriptorTemplate::Taproot => write!(f, "Taproot (P2TR)"),
            DescriptorTemplate::Custom(template) => write!(f, "Custom ({})", template),
        }
    }
}

/// Common error types for wallet operations
///
/// # Security
///
/// These error types are designed to provide meaningful errors without leaking
/// sensitive information. Error messages should be carefully constructed to avoid
/// revealing implementation details that could be used in attacks.
///
/// # Examples
///
/// ```
/// use bitvault_common::types::WalletError;
///
/// let error = WalletError::InvalidAddress("invalid_address".to_string());
/// assert!(error.to_string().contains("Invalid address"));
/// ```
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid transaction ID: {0}")]
    InvalidTransactionId(String),

    #[error("Invalid network type: {0}")]
    InvalidNetworkType(String),

    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Negative amount not allowed")]
    NegativeAmount,

    #[error("Amount exceeds maximum Bitcoin supply")]
    ExcessiveAmount,

    #[error("Amount math error: {0}")]
    AmountMathError(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Dust output: {amount} satoshis (minimum is {minimum} satoshis)")]
    DustOutput { amount: u64, minimum: u64 },

    #[error("Insufficient funds: needed {needed}, available {available}")]
    InsufficientFunds { needed: u64, available: u64 },

    #[error("BDK error: {0}")]
    BdkError(String),

    #[error("Bitcoin error: {0}")]
    BitcoinError(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<bdk::Error> for WalletError {
    fn from(err: bdk::Error) -> Self {
        match err {
            bdk::Error::Generic(msg) => Self::BdkError(format!("Generic BDK error: {}", msg)),
            bdk::Error::Descriptor(desc_err) => {
                Self::BdkError(format!("Descriptor error: {}", desc_err))
            }
            bdk::Error::Key(key_err) => Self::BdkError(format!("Key error: {}", key_err)),
            bdk::Error::Miniscript(ms_err) => {
                Self::BdkError(format!("Miniscript error: {}", ms_err))
            }
            bdk::Error::Signer(signer_err) => {
                Self::BdkError(format!("Signer error: {}", signer_err))
            }
            _ => Self::BdkError(format!("Other BDK error: {}", err)),
        }
    }
}

impl From<address::Error> for WalletError {
    fn from(err: address::Error) -> Self {
        WalletError::InvalidAddress(format!("Bitcoin error: {}", err))
    }
}

/// Settings for the wallet
#[derive(Clone, Debug)]
pub struct WalletSettings {
    /// Network in use
    pub network: Network,
    /// Whether to use Tor for connections
    pub use_tor: bool,
    /// Default fee rate selection
    pub default_fee_level: FeePriority,
    /// Custom fee rate if selected
    pub custom_fee_rate: Option<FeeRate>,
    /// Whether to display amounts in BTC or sats
    pub display_as_bitcoin: bool,
    /// User preferred currency for fiat conversion
    pub preferred_fiat: String,
    /// Whether to display fiat equivalents
    pub show_fiat_amounts: bool,
    /// Additional user-defined settings
    pub custom_settings: HashMap<String, String>,
}

// Custom serialization for WalletSettings
impl Serialize for WalletSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WalletSettings", 8)?;
        state.serialize_field("network", &self.network)?;
        state.serialize_field("use_tor", &self.use_tor)?;
        state.serialize_field("default_fee_level", &self.default_fee_level)?;

        // Serialize custom_fee_rate as f32 if Some, or None
        if let Some(fee_rate) = self.custom_fee_rate {
            state.serialize_field("custom_fee_rate", &fee_rate.as_sat_per_vb())?;
        } else {
            state.serialize_field("custom_fee_rate", &Option::<f32>::None)?;
        }

        state.serialize_field("display_as_bitcoin", &self.display_as_bitcoin)?;
        state.serialize_field("preferred_fiat", &self.preferred_fiat)?;
        state.serialize_field("show_fiat_amounts", &self.show_fiat_amounts)?;
        state.serialize_field("custom_settings", &self.custom_settings)?;
        state.end()
    }
}

// Custom deserialization for WalletSettings
impl<'de> Deserialize<'de> for WalletSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WalletSettingsHelper {
            network: Network,
            use_tor: bool,
            default_fee_level: FeePriority,
            custom_fee_rate: Option<f32>,
            display_as_bitcoin: bool,
            preferred_fiat: String,
            show_fiat_amounts: bool,
            custom_settings: HashMap<String, String>,
        }

        let helper = WalletSettingsHelper::deserialize(deserializer)?;
        Ok(WalletSettings {
            network: helper.network,
            use_tor: helper.use_tor,
            default_fee_level: helper.default_fee_level,
            custom_fee_rate: helper.custom_fee_rate.map(FeeRate::from_sat_per_vb),
            display_as_bitcoin: helper.display_as_bitcoin,
            preferred_fiat: helper.preferred_fiat,
            show_fiat_amounts: helper.show_fiat_amounts,
            custom_settings: helper.custom_settings,
        })
    }
}

// Add the Default implementation back
impl Default for WalletSettings {
    fn default() -> Self {
        Self {
            network: Network::Bitcoin,
            use_tor: false,
            default_fee_level: FeePriority::Medium,
            custom_fee_rate: None,
            display_as_bitcoin: true,
            preferred_fiat: "USD".to_string(),
            show_fiat_amounts: true,
            custom_settings: HashMap::new(),
        }
    }
}

/// Wallet metadata information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    /// Unique wallet identifier
    pub id: String,
    /// User-provided wallet name
    pub name: String,
    /// Wallet descriptor template type
    pub descriptor_type: DescriptorTemplate,
    /// Bitcoin network
    pub network: Network,
    /// When the wallet was created (Unix timestamp)
    pub created_at: u64,
    /// Last time the wallet was used (Unix timestamp)
    pub last_used: u64,
    /// Whether this is the default wallet
    pub is_default: bool,
}

/// Seed import/export format
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SeedFormat {
    /// BIP39 mnemonic (12/15/18/21/24 words)
    Bip39Mnemonic,
    /// Electrum mnemonic
    ElectrumMnemonic,
    /// Hardware wallet (no direct seed access)
    HardwareWallet,
    /// Watch-only (no private keys)
    WatchOnly,
}

/// Backup method configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Whether cloud backup is enabled
    pub cloud_enabled: bool,
    /// Whether local backup is enabled
    pub local_enabled: bool,
    /// Backup encryption method
    pub encryption: BackupEncryption,
    /// Custom backup settings
    pub settings: HashMap<String, String>,
}

/// Backup encryption methods
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackupEncryption {
    /// No encryption
    None,
    /// Password-based encryption
    Password,
    /// Key-based encryption
    Key,
}

/// Sanitize a string for display (to avoid leaking sensitive data)
///
/// This function keeps the first `prefix_chars` characters and replaces
/// the rest with asterisks.
pub fn sanitize_for_display(input: &str, prefix_chars: usize) -> String {
    if input.len() <= prefix_chars {
        return input.to_string();
    }

    let visible = &input[0..prefix_chars];
    let hidden = "*".repeat(input.len() - prefix_chars);
    format!("{}{}", visible, hidden)
}

/// Bitcoin script types based on BDK
pub enum ScriptType {
    /// Pay to Public Key Hash (legacy)
    Pkh,
    /// Pay to Script Hash (wrapped SegWit)
    Sh,
    /// Pay to Witness Public Key Hash (native SegWit)
    Wpkh,
    /// Pay to Witness Script Hash
    Wsh,
    /// Pay to Taproot (Taproot)
    Tr,
}

/// Convert BDK's script type to a human-readable string
pub fn script_type_to_string(script_type: ScriptType) -> &'static str {
    match script_type {
        ScriptType::Pkh => "Legacy (P2PKH)",
        ScriptType::Sh => "Wrapped SegWit (P2SH)",
        ScriptType::Wpkh => "Native SegWit (P2WPKH)",
        ScriptType::Wsh => "Native SegWit Script (P2WSH)",
        ScriptType::Tr => "Taproot (P2TR)",
    }
}

/// Calculate fee rate given transaction size and fee
pub fn calculate_fee_rate(fee_sats: u64, tx_vsize: usize) -> Option<FeeRate> {
    if tx_vsize == 0 {
        return None;
    }

    let rate_f = fee_sats as f32 / tx_vsize as f32;
    if rate_f <= 0.0 {
        return None;
    }

    let rate = rate_f.ceil();
    Some(FeeRate::from_sat_per_vb(rate))
}
