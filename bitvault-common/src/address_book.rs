//! Address Book functionality for BitVault
//!
//! This module provides types and utilities for managing a Bitcoin address book.
//! It allows users to store, categorize, and retrieve addresses with labels and metadata.
//!
//! # Security Considerations
//!
//! - Address book entries only store public addresses, never private keys
//! - All data is validated before storage to prevent injection attacks
//! - Addresses are always validated against the network type
//!
//! # Examples
//!
//! ```
//! use bitvault_common::address_book::{AddressBook, AddressEntry, AddressCategory};
//! use bitcoin::Network;
//!
//! // Create a new address book
//! let mut address_book = AddressBook::new(Network::Bitcoin);
//!
//! // Add an entry
//! address_book.add_entry(
//!     "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
//!     "Satoshi Donation",
//!     Some("First ever Bitcoin address"),
//!     AddressCategory::Donation
//! ).expect("Failed to add entry");
//!
//! // Find entries
//! let entries = address_book.find_by_label("Donation");
//! ```

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use bitcoin::{Address, Network};
use crate::bitcoin_utils;
use crate::types::WalletError;

/// Categories for address book entries
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressCategory {
    /// Personal addresses (family, friends)
    Personal,
    /// Business-related addresses (stores, services)
    Business,
    /// Donation addresses (charities, projects)
    Donation,
    /// Exchange addresses (centralized exchanges)
    Exchange,
    /// Custom user-defined category
    Custom(String),
}

impl Default for AddressCategory {
    fn default() -> Self {
        AddressCategory::Personal
    }
}

impl std::fmt::Display for AddressCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressCategory::Personal => write!(f, "Personal"),
            AddressCategory::Business => write!(f, "Business"),
            AddressCategory::Donation => write!(f, "Donation"),
            AddressCategory::Exchange => write!(f, "Exchange"),
            AddressCategory::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// A single address book entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressEntry {
    /// The Bitcoin address (serialized as string)
    pub address: String,
    /// Human-readable label for this address
    pub label: String,
    /// Optional additional notes about this address
    pub notes: Option<String>,
    /// Category for grouping addresses
    pub category: AddressCategory,
    /// Timestamp when this entry was created (in seconds since UNIX epoch)
    pub created_at: u64,
    /// Timestamp when this entry was last used (in seconds since UNIX epoch)
    pub last_used: Option<u64>,
}

impl AddressEntry {
    /// Create a new address book entry
    ///
    /// # Arguments
    /// * `address` - Bitcoin address as string
    /// * `label` - Human-readable label
    /// * `notes` - Optional notes about the address
    /// * `category` - Category for grouping
    ///
    /// # Returns
    /// * A new AddressEntry with current timestamp
    pub fn new(
        address: String,
        label: String,
        notes: Option<String>,
        category: AddressCategory,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        AddressEntry {
            address,
            label,
            notes,
            category,
            created_at: now,
            last_used: None,
        }
    }

    /// Parse the string address into a Bitcoin Address object
    ///
    /// # Arguments
    /// * `network` - The Bitcoin network to validate against
    ///
    /// # Returns
    /// * Result containing parsed Address or error
    pub fn parse_address(&self, network: Network) -> Result<Address, WalletError> {
        bitcoin_utils::parse_address(&self.address, network)
    }

    /// Update the last used timestamp to now
    pub fn mark_as_used(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        self.last_used = Some(now);
    }
}

/// Address book container for managing addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBook {
    /// Entries in the address book, indexed by address string
    entries: HashMap<String, AddressEntry>,
    /// The Bitcoin network for validating addresses
    network: Network,
}

impl AddressBook {
    /// Create a new empty address book
    ///
    /// # Arguments
    /// * `network` - The Bitcoin network for this address book
    ///
    /// # Returns
    /// * A new empty address book
    pub fn new(network: Network) -> Self {
        AddressBook {
            entries: HashMap::new(),
            network,
        }
    }

    /// Add an entry to the address book
    ///
    /// # Arguments
    /// * `address` - Bitcoin address as string
    /// * `label` - Human-readable label
    /// * `notes` - Optional notes about the address
    /// * `category` - Category for grouping
    ///
    /// # Returns
    /// * Result with () on success, error on failure
    pub fn add_entry(
        &mut self,
        address: &str,
        label: &str,
        notes: Option<&str>,
        category: AddressCategory,
    ) -> Result<(), WalletError> {
        // Validate the address
        if !bitcoin_utils::is_valid_bitcoin_address(address, self.network) {
            return Err(WalletError::InvalidAddress(format!(
                "Invalid address for {} network: {}",
                self.network, address
            )));
        }

        // Validate label is not empty
        if label.trim().is_empty() {
            return Err(WalletError::ValidationError("Label cannot be empty".to_string()));
        }

        // Convert Option<&str> to Option<String>
        let notes_owned = notes.map(|s| s.to_string());

        // Create and add the entry
        let entry = AddressEntry::new(
            address.to_string(),
            label.to_string(),
            notes_owned,
            category,
        );

        self.entries.insert(address.to_string(), entry);
        Ok(())
    }

    /// Get an entry by address
    ///
    /// # Arguments
    /// * `address` - Bitcoin address to look up
    ///
    /// # Returns
    /// * Some(AddressEntry) if found, None otherwise
    pub fn get_entry(&self, address: &str) -> Option<&AddressEntry> {
        self.entries.get(address)
    }

    /// Get a mutable reference to an entry
    ///
    /// # Arguments
    /// * `address` - Bitcoin address to look up
    ///
    /// # Returns
    /// * Some(AddressEntry) if found, None otherwise
    pub fn get_entry_mut(&mut self, address: &str) -> Option<&mut AddressEntry> {
        self.entries.get_mut(address)
    }

    /// Remove an entry by address
    ///
    /// # Arguments
    /// * `address` - Bitcoin address to remove
    ///
    /// # Returns
    /// * true if an entry was removed, false if not found
    pub fn remove_entry(&mut self, address: &str) -> bool {
        self.entries.remove(address).is_some()
    }

    /// Update an existing entry
    ///
    /// # Arguments
    /// * `address` - Bitcoin address to update
    /// * `label` - New label (None to keep existing)
    /// * `notes` - New notes (None to keep existing)
    /// * `category` - New category (None to keep existing)
    ///
    /// # Returns
    /// * Result with () on success, error on failure
    pub fn update_entry(
        &mut self,
        address: &str,
        label: Option<&str>,
        notes: Option<&str>,
        category: Option<AddressCategory>,
    ) -> Result<(), WalletError> {
        // Check if entry exists
        let entry = self.entries.get_mut(address).ok_or_else(|| {
            WalletError::NotFound(format!("Address not found in address book: {}", address))
        })?;

        // Update fields if provided
        if let Some(label) = label {
            if label.trim().is_empty() {
                return Err(WalletError::ValidationError("Label cannot be empty".to_string()));
            }
            entry.label = label.to_string();
        }

        if let Some(notes) = notes {
            entry.notes = Some(notes.to_string());
        }

        if let Some(category) = category {
            entry.category = category;
        }

        Ok(())
    }

    /// Mark an address as used, updating its last_used timestamp
    ///
    /// # Arguments
    /// * `address` - Bitcoin address to mark as used
    ///
    /// # Returns
    /// * Result with () on success, error if address not found
    pub fn mark_as_used(&mut self, address: &str) -> Result<(), WalletError> {
        let entry = self.entries.get_mut(address).ok_or_else(|| {
            WalletError::NotFound(format!("Address not found in address book: {}", address))
        })?;

        entry.mark_as_used();
        Ok(())
    }

    /// Find entries by label (case-insensitive partial match)
    ///
    /// # Arguments
    /// * `label` - Label substring to search for
    ///
    /// # Returns
    /// * Vector of matching AddressEntry references
    pub fn find_by_label(&self, label: &str) -> Vec<&AddressEntry> {
        let search_term = label.to_lowercase();
        self.entries
            .values()
            .filter(|entry| entry.label.to_lowercase().contains(&search_term))
            .collect()
    }

    /// Find entries by category
    ///
    /// # Arguments
    /// * `category` - Category to filter by
    ///
    /// # Returns
    /// * Vector of matching AddressEntry references
    pub fn find_by_category(&self, category: &AddressCategory) -> Vec<&AddressEntry> {
        self.entries
            .values()
            .filter(|entry| match (category, &entry.category) {
                (AddressCategory::Custom(a), AddressCategory::Custom(b)) => a == b,
                (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
            })
            .collect()
    }

    /// Get all entries in the address book
    ///
    /// # Returns
    /// * Vector of all AddressEntry references
    pub fn get_all_entries(&self) -> Vec<&AddressEntry> {
        self.entries.values().collect()
    }

    /// Get the network type for this address book
    ///
    /// # Returns
    /// * The Bitcoin network
    pub fn network(&self) -> Network {
        self.network
    }

    /// Get the number of entries in the address book
    ///
    /// # Returns
    /// * Number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the address book is empty
    ///
    /// # Returns
    /// * true if empty, false otherwise
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Import entries from another address book
    ///
    /// # Arguments
    /// * `other` - Another address book to import from
    /// * `overwrite` - Whether to overwrite existing entries
    ///
    /// # Returns
    /// * Number of entries imported
    pub fn import_entries(&mut self, other: &AddressBook, overwrite: bool) -> usize {
        if self.network != other.network {
            // Don't import if networks don't match
            return 0;
        }

        let mut imported = 0;
        for (address, entry) in &other.entries {
            if !self.entries.contains_key(address) || overwrite {
                self.entries.insert(address.clone(), entry.clone());
                imported += 1;
            }
        }
        imported
    }

    /// Serialize the address book to JSON
    ///
    /// # Returns
    /// * Result with JSON string on success, error on failure
    pub fn to_json(&self) -> Result<String, WalletError> {
        serde_json::to_string(self).map_err(|e| {
            WalletError::SerializationError(format!("Failed to serialize address book: {}", e))
        })
    }

    /// Deserialize from JSON string
    ///
    /// # Arguments
    /// * `json` - JSON string to parse
    ///
    /// # Returns
    /// * Result with AddressBook on success, error on failure
    pub fn from_json(json: &str) -> Result<Self, WalletError> {
        serde_json::from_str(json).map_err(|e| {
            WalletError::DeserializationError(format!("Failed to deserialize address book: {}", e))
        })
    }
} 