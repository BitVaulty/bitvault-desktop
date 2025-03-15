//! Core types for UTXO selection
//!
//! This module defines the fundamental types used for UTXO selection,
//! including the Utxo and UtxoSet structures, and the selection result types.
//!
//! # Key Types
//!
//! - [`Utxo`]: Represents a single unspent transaction output
//! - [`UtxoSet`]: A collection of UTXOs with helper methods
//! - [`SelectionStrategy`]: Enum representing different selection strategies
//! - [`SelectionResult`]: Result of a UTXO selection operation
//!
//! # Usage
//!
//! These types form the foundation of the UTXO selection framework. They are used
//! by the `UtxoSelector` and strategy implementations to represent and manipulate
//! Bitcoin UTXOs during the selection process.
//!
//! # Example
//!
//! ```no_run
//! use bitvault_common::utxo_selection::types::{Utxo, UtxoSet, SelectionStrategy};
//! use bitcoin::{Amount, OutPoint, Txid, Network};
//! use std::str::FromStr;
//!
//! // Create a new UTXO
//! let utxo = Utxo::new(
//!     OutPoint::new(
//!         Txid::from_str("7967a5185e907a25225574544c31f7b059c1a191d65b53dcc1554d339c4f9efc").unwrap(),
//!         0
//!     ),
//!     Amount::from_sat(10_000),
//!     6, // 6 confirmations
//!     false, // not a change output
//! );
//!
//! // Add metadata to the UTXO
//! let utxo = utxo
//!     .with_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string())
//!     .with_label("Savings".to_string());
//!
//! // Create a UTXO set
//! let mut utxo_set = UtxoSet::new_empty();
//! utxo_set.add(utxo);
//!
//! // Get the total value of the UTXO set
//! let total_value = utxo_set.total_value();
//! assert_eq!(total_value, Amount::from_sat(10_000));
//! ```
//!
//! # Security Considerations
//!
//! - The `Utxo` type stores the full UTXO information but does not contain private keys
//! - Serialization/deserialization is implemented for persistence, but care should be taken
//!   when storing or transmitting serialized UTXOs
//! - The `freeze` and `unfreeze` methods allow controlling which UTXOs are eligible for selection

use bitcoin::{Amount, OutPoint, Txid, Network};
use crate::types::WalletError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Unspent transaction output (UTXO) representation
///
/// A UTXO is an unspent output from a previous Bitcoin transaction
/// that can be used as an input in a new transaction.
///
/// # Fields
///
/// * `outpoint` - Reference to the transaction output (txid and vout)
/// * `amount` - Amount of bitcoin in this UTXO
/// * `confirmations` - Number of confirmations (0 for unconfirmed)
/// * `is_change` - Whether this is a change output from our own transaction
/// * `is_frozen` - Whether this UTXO is frozen (excluded from automatic selection)
/// * `address` - Optional Bitcoin address associated with this UTXO
/// * `derivation_path` - Optional derivation path (for HD wallets)
/// * `label` - Optional user-defined label
/// * `network` - Bitcoin network this UTXO belongs to
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    /// Reference to the transaction output (txid and vout)
    pub outpoint: OutPoint,
    
    /// Amount in this UTXO
    pub amount: Amount,
    
    /// Number of confirmations (0 for unconfirmed)
    pub confirmations: u32,
    
    /// Is this a change output from a previous transaction?
    pub is_change: bool,
    
    /// Is this UTXO frozen (excluded from automatic selection)?
    pub is_frozen: bool,
    
    /// Optional address associated with this UTXO
    pub address: Option<String>,
    
    /// Optional derivation path (for owned addresses)
    pub derivation_path: Option<String>,
    
    /// Optional label for display
    pub label: Option<String>,
    
    /// Network this UTXO belongs to
    pub network: Network,
}

impl Serialize for Utxo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("Utxo", 9)?;
        state.serialize_field("outpoint_txid", &self.outpoint.txid.to_string())?;
        state.serialize_field("outpoint_vout", &self.outpoint.vout)?;
        state.serialize_field("amount_sats", &self.amount.to_sat())?;
        state.serialize_field("confirmations", &self.confirmations)?;
        state.serialize_field("is_change", &self.is_change)?;
        state.serialize_field("is_frozen", &self.is_frozen)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("derivation_path", &self.derivation_path)?;
        state.serialize_field("label", &self.label)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Utxo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserialize, MapAccess, Visitor};
        
        struct UtxoHelper {
            outpoint_txid: String,
            outpoint_vout: u32,
            amount_sats: u64,
            confirmations: u32,
            is_change: bool,
            is_frozen: bool,
            address: Option<String>,
            derivation_path: Option<String>,
            label: Option<String>,
        }
        
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { 
            OutpointTxid, OutpointVout, AmountSats, Confirmations, 
            IsChange, IsFrozen, Address, DerivationPath, Label 
        }
        
        impl<'de> Deserialize<'de> for UtxoHelper {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_struct(
                    "Utxo",
                    &[
                        "outpoint_txid", "outpoint_vout", "amount_sats", "confirmations",
                        "is_change", "is_frozen", "address", "derivation_path", "label",
                    ],
                    UtxoVisitor,
                )
            }
        }
        
        struct UtxoVisitor;
        
        impl<'de> Visitor<'de> for UtxoVisitor {
            type Value = UtxoHelper;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Utxo")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut outpoint_txid = None;
                let mut outpoint_vout = None;
                let mut amount_sats = None;
                let mut confirmations = None;
                let mut is_change = None;
                let mut is_frozen = None;
                let mut address = None;
                let mut derivation_path = None;
                let mut label = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::OutpointTxid => {
                            outpoint_txid = Some(map.next_value()?);
                        }
                        Field::OutpointVout => {
                            outpoint_vout = Some(map.next_value()?);
                        }
                        Field::AmountSats => {
                            amount_sats = Some(map.next_value()?);
                        }
                        Field::Confirmations => {
                            confirmations = Some(map.next_value()?);
                        }
                        Field::IsChange => {
                            is_change = Some(map.next_value()?);
                        }
                        Field::IsFrozen => {
                            is_frozen = Some(map.next_value()?);
                        }
                        Field::Address => {
                            address = Some(map.next_value()?);
                        }
                        Field::DerivationPath => {
                            derivation_path = Some(map.next_value()?);
                        }
                        Field::Label => {
                            label = Some(map.next_value()?);
                        }
                    }
                }
                
                let outpoint_txid = outpoint_txid.ok_or_else(|| de::Error::missing_field("outpoint_txid"))?;
                let outpoint_vout = outpoint_vout.ok_or_else(|| de::Error::missing_field("outpoint_vout"))?;
                let amount_sats = amount_sats.ok_or_else(|| de::Error::missing_field("amount_sats"))?;
                let confirmations = confirmations.ok_or_else(|| de::Error::missing_field("confirmations"))?;
                let is_change = is_change.ok_or_else(|| de::Error::missing_field("is_change"))?;
                let is_frozen = is_frozen.unwrap_or(false);
                
                Ok(UtxoHelper {
                    outpoint_txid,
                    outpoint_vout,
                    amount_sats,
                    confirmations,
                    is_change,
                    is_frozen,
                    address,
                    derivation_path,
                    label,
                })
            }
        }
        
        let helper = UtxoHelper::deserialize(deserializer)?;
        
        let txid = Txid::from_str(&helper.outpoint_txid)
            .map_err(|_| de::Error::custom("Invalid txid"))?;
            
        let outpoint = OutPoint::new(txid, helper.outpoint_vout);
        let amount = Amount::from_sat(helper.amount_sats);
        
        Ok(Utxo {
            outpoint,
            amount,
            confirmations: helper.confirmations,
            is_change: helper.is_change,
            is_frozen: helper.is_frozen,
            address: helper.address,
            derivation_path: helper.derivation_path,
            label: helper.label,
            network: Network::Bitcoin, // Default to mainnet, should be set properly later
        })
    }
}

impl Utxo {
    /// Create a new UTXO with basic information
    ///
    /// # Arguments
    /// * `outpoint` - The transaction outpoint (txid and vout)
    /// * `amount` - The amount in this UTXO
    /// * `confirmations` - Number of confirmations (0 for unconfirmed)
    /// * `is_change` - Whether this is a change output from a previous transaction
    ///
    /// # Returns
    /// * A new UTXO instance with default values for other fields
    pub fn new(
        outpoint: OutPoint,
        amount: Amount,
        confirmations: u32,
        is_change: bool,
    ) -> Self {
        Self {
            outpoint,
            amount,
            confirmations,
            is_change,
            is_frozen: false,
            address: None,
            derivation_path: None,
            label: None,
            network: Network::Bitcoin, // Default to mainnet
        }
    }
    
    /// Create a new UTXO with network information
    ///
    /// # Arguments
    /// * `outpoint` - The transaction outpoint (txid and vout)
    /// * `amount` - The amount in this UTXO
    /// * `confirmations` - Number of confirmations (0 for unconfirmed)
    /// * `is_change` - Whether this is a change output from a previous transaction
    /// * `network` - Bitcoin network this UTXO belongs to
    ///
    /// # Returns
    /// * A new UTXO instance with specified network and default values for other fields
    pub fn new_with_network(
        outpoint: OutPoint,
        amount: Amount,
        confirmations: u32,
        is_change: bool,
        network: Network,
    ) -> Self {
        Self {
            outpoint,
            amount,
            confirmations,
            is_change,
            is_frozen: false,
            address: None,
            derivation_path: None,
            label: None,
            network,
        }
    }
    
    /// Add address information to this UTXO
    ///
    /// # Arguments
    /// * `address` - The address associated with this UTXO
    ///
    /// # Returns
    /// * Self with address field set
    pub fn with_address(
        mut self,
        address: String,
    ) -> Self {
        self.address = Some(address);
        self
    }
    
    /// Add derivation path information to this UTXO
    ///
    /// # Arguments
    /// * `path` - The derivation path associated with this UTXO
    ///
    /// # Returns
    /// * Self with derivation_path field set
    pub fn with_derivation_path(
        mut self,
        path: String,
    ) -> Self {
        self.derivation_path = Some(path);
        self
    }
    
    /// Add label information to this UTXO
    ///
    /// # Arguments
    /// * `label` - The label for this UTXO
    ///
    /// # Returns
    /// * Self with label field set
    pub fn with_label(
        mut self,
        label: String,
    ) -> Self {
        self.label = Some(label);
        self
    }
    
    /// Set the network for this UTXO
    ///
    /// # Arguments
    /// * `network` - Bitcoin network this UTXO belongs to
    ///
    /// # Returns
    /// * Self with network field set
    pub fn with_network(
        mut self,
        network: Network,
    ) -> Self {
        self.network = network;
        self
    }
    
    /// Check if this UTXO is confirmed
    pub fn is_confirmed(&self) -> bool {
        self.confirmations > 0
    }
    
    /// Check if this UTXO is mature (has enough confirmations for spending)
    pub fn is_mature(&self) -> bool {
        self.confirmations >= 6
    }
    
    /// Check if this UTXO is dust according to network rules
    pub fn is_dust(&self) -> bool {
        crate::types::is_dust(self.amount, self.network)
    }
    
    /// Get a unique identifier for this UTXO
    pub fn id(&self) -> String {
        format!("{}:{}", self.outpoint.txid, self.outpoint.vout)
    }
    
    /// Freeze this UTXO to exclude it from automatic selection
    pub fn freeze(&mut self) {
        self.is_frozen = true;
    }
    
    /// Unfreeze this UTXO to include it in automatic selection
    pub fn unfreeze(&mut self) {
        self.is_frozen = false;
    }
    
    /// Get the UTXO's identifier (alias for id() for backward compatibility)
    pub fn get_id(&self) -> String {
        self.id()
    }
}

/// Collection of UTXOs with utility methods
pub struct UtxoSet {
    /// Vector of available UTXOs
    utxos: Vec<Utxo>,
    /// Bitcoin network
    network: Network,
}

impl UtxoSet {
    /// Create a new empty UTXO set with default network (Bitcoin mainnet)
    pub fn new_empty() -> Self {
        Self {
            utxos: Vec::new(),
            network: Network::Bitcoin,
        }
    }
    
    /// Create a new UTXO set with the given UTXOs and network
    pub fn new(utxos: Vec<Utxo>, network: Network) -> Self {
        Self { utxos, network }
    }
    
    /// Get all UTXOs in this set
    pub fn get_all(&self) -> &[Utxo] {
        &self.utxos
    }
    
    /// Get a specific UTXO by outpoint
    ///
    /// # Arguments
    /// * `outpoint` - The transaction outpoint to look for
    ///
    /// # Returns
    /// * Some reference to the UTXO if found, None otherwise
    pub fn get(&self, outpoint: &OutPoint) -> Option<&Utxo> {
        self.utxos.iter().find(|utxo| utxo.outpoint == *outpoint)
    }
    
    /// Get a mutable reference to a specific UTXO by outpoint
    ///
    /// # Arguments
    /// * `outpoint` - The transaction outpoint to look for
    ///
    /// # Returns
    /// * Some mutable reference to the UTXO if found, None otherwise
    pub fn get_mut(&mut self, outpoint: &OutPoint) -> Option<&mut Utxo> {
        self.utxos.iter_mut().find(|utxo| utxo.outpoint == *outpoint)
    }
    
    /// Add a UTXO to this set
    ///
    /// # Arguments
    /// * `utxo` - The UTXO to add
    ///
    /// # Returns
    /// * true if the UTXO was added, false if a UTXO with the same outpoint already exists
    pub fn add(&mut self, utxo: Utxo) -> bool {
        if self.get(&utxo.outpoint).is_some() {
            return false;
        }
        
        self.utxos.push(utxo);
        true
    }
    
    /// Remove a UTXO from this set
    ///
    /// # Arguments
    /// * `outpoint` - The transaction outpoint to remove
    ///
    /// # Returns
    /// * Some UTXO if it was removed, None if it wasn't found
    pub fn remove(&mut self, outpoint: &OutPoint) -> Option<Utxo> {
        if let Some(index) = self.utxos.iter().position(|utxo| utxo.outpoint == *outpoint) {
            Some(self.utxos.remove(index))
        } else {
            None
        }
    }
    
    /// Get the total value of all UTXOs in this set
    pub fn total_value(&self) -> Amount {
        self.utxos.iter().map(|utxo| utxo.amount).sum()
    }
    
    /// Get the total value of confirmed UTXOs in this set
    pub fn total_value_confirmed(&self, min_confirmations: u32) -> Amount {
        self.utxos.iter()
            .filter(|utxo| utxo.confirmations >= min_confirmations)
            .map(|utxo| utxo.amount)
            .sum()
    }
    
    /// Count non-dust UTXOs in this set
    pub fn non_dust_count(&self) -> usize {
        self.utxos.iter()
            .filter(|utxo| !utxo.is_dust())
            .count()
    }
    
    /// Get the network this UTXO set is for
    pub fn network(&self) -> Network {
        self.network
    }
    
    /// Check if this UTXO set is empty
    pub fn is_empty(&self) -> bool {
        self.utxos.is_empty()
    }
    
    /// Get the number of UTXOs in this set
    pub fn len(&self) -> usize {
        self.utxos.len()
    }
    
    /// Get the total amount of all UTXOs in this set (alias for total_value)
    pub fn get_total(&self) -> Amount {
        self.total_value()
    }
    
    /// Get all available (unfrozen and confirmed) UTXOs
    pub fn get_available(&self) -> Vec<&Utxo> {
        self.utxos.iter()
            .filter(|utxo| !utxo.is_frozen && utxo.is_confirmed())
            .collect()
    }
    
    /// Freeze a UTXO in this set
    ///
    /// # Arguments
    /// * `outpoint` - The transaction outpoint to freeze
    ///
    /// # Returns
    /// * Ok if the UTXO was found and frozen, Err otherwise
    pub fn freeze(&mut self, outpoint: &OutPoint) -> Result<(), WalletError> {
        if let Some(utxo) = self.get_mut(outpoint) {
            utxo.freeze();
            Ok(())
        } else {
            Err(WalletError::NotFound(outpoint.to_string()))
        }
    }
    
    /// Unfreeze a UTXO in this set
    ///
    /// # Arguments
    /// * `outpoint` - The transaction outpoint to unfreeze
    ///
    /// # Returns
    /// * Ok if the UTXO was found and unfrozen, Err otherwise
    pub fn unfreeze(&mut self, outpoint: &OutPoint) -> Result<(), WalletError> {
        if let Some(utxo) = self.get_mut(outpoint) {
            utxo.unfreeze();
            Ok(())
        } else {
            Err(WalletError::NotFound(outpoint.to_string()))
        }
    }
}

/// UTXO selection strategy
///
/// This enum represents the different strategies that can be used
/// for selecting UTXOs when creating a transaction.
///
/// Each strategy optimizes for different criteria such as fees,
/// privacy, or UTXO consolidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStrategy {
    /// Minimize transaction fee by using larger UTXOs
    ///
    /// This strategy selects the fewest, largest UTXOs necessary to satisfy
    /// the target amount, reducing the number of inputs and therefore the
    /// transaction fee.
    MinimizeFee,
    
    /// Minimize change amount to reduce UTXO count
    ///
    /// This strategy tries to find a combination of UTXOs that results in
    /// the smallest possible change output, potentially eliminating the need
    /// for a change output entirely if an exact match can be found.
    MinimizeChange,
    
    /// Select oldest UTXOs first (most confirmations)
    ///
    /// This strategy prioritizes UTXOs with the most confirmations, which
    /// can be useful for coin control and to prevent prioritization issues
    /// with unconfirmed outputs.
    OldestFirst,
    
    /// Focus on privacy by avoiding address reuse
    ///
    /// This strategy selects UTXOs from different addresses to enhance privacy
    /// by avoiding address clustering.
    PrivacyFocused,
    
    /// Maximize privacy by using more UTXOs
    ///
    /// This strategy selects more UTXOs from different sources to maximize
    /// transaction privacy, even at the cost of higher fees.
    MaximizePrivacy,
    
    /// Consolidate multiple UTXOs into fewer outputs
    ///
    /// This strategy prioritizes selecting many smaller UTXOs to help
    /// consolidate the wallet's UTXO set, reducing future transaction fees.
    Consolidate,
    
    /// Use specific coin selection by user choice
    ///
    /// This strategy is used when the user has manually selected specific
    /// UTXOs to use in a transaction.
    CoinControl,
    
    /// Try to avoid change outputs completely
    ///
    /// This strategy attempts to find a combination of UTXOs that results
    /// in no change output at all, by matching the target amount exactly
    /// (accounting for fees).
    AvoidChange,
}

/// Result of UTXO selection
///
/// This enum represents the possible outcomes of a UTXO selection operation:
/// either success with selected UTXOs, fee, and change amounts, or
/// insufficient funds.
pub enum SelectionResult {
    /// Selection successful
    ///
    /// # Fields
    ///
    /// * `selected` - Vector of selected UTXOs
    /// * `fee_amount` - Estimated fee amount for the transaction
    /// * `change_amount` - Change amount (if any) that will be returned to the wallet
    Success {
        /// Selected UTXOs
        selected: Vec<Utxo>,
        /// Estimated fee amount
        fee_amount: Amount,
        /// Change amount
        change_amount: Amount,
    },
    
    /// Insufficient funds
    ///
    /// # Fields
    ///
    /// * `available` - Total available amount in the wallet
    /// * `required` - Required amount for the transaction
    InsufficientFunds {
        /// Available amount
        available: Amount,
        /// Required amount
        required: Amount,
    },
}

impl std::fmt::Debug for SelectionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionResult::Success { selected, fee_amount, change_amount } => {
                f.debug_struct("Success")
                    .field("selected", selected)
                    .field("fee_amount", fee_amount)
                    .field("change_amount", change_amount)
                    .finish()
            }
            SelectionResult::InsufficientFunds { available, required } => {
                f.debug_struct("InsufficientFunds")
                    .field("available", available)
                    .field("required", required)
                    .finish()
            }
        }
    }
} 