//! Address Book Service
//!
//! Manages saved addresses/contacts for quick access when sending transactions.
//! Similar to the mobile app's RecentAddress feature.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A saved address entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressEntry {
    /// The Bitcoin address
    pub address: String,
    /// Optional label/name for the address
    pub label: Option<String>,
    /// Timestamp when this address was last used
    pub last_used: DateTime<Utc>,
    /// Timestamp when this address was added
    pub created_at: DateTime<Utc>,
}

/// Address book storage (per-vault)
#[derive(Debug, Serialize, Deserialize)]
struct AddressBookData {
    /// Map of address -> entry
    addresses: HashMap<String, AddressEntry>,
}

impl AddressBookData {
    fn new() -> Self {
        Self {
            addresses: HashMap::new(),
        }
    }
}

/// Address book service
pub struct AddressBookService {
    data_dir: PathBuf,
}

impl AddressBookService {
    /// Create a new address book service
    pub fn new() -> Result<Self, String> {
        let data_dir = Self::get_data_directory()?;
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create address book directory: {}", e))?;

        Ok(Self { data_dir })
    }

    /// Get the data directory for address books
    fn get_data_directory() -> Result<PathBuf, String> {
        let data_dir =
            dirs::data_dir().ok_or_else(|| "Could not find data directory".to_string())?;
        Ok(data_dir.join("bitvault").join("address_books"))
    }

    /// Get the file path for a vault's address book
    fn get_vault_file_path(&self, vault_address: &str) -> PathBuf {
        // Sanitize vault address for filename
        let sanitized = vault_address.replace(":", "_").replace("/", "_");
        self.data_dir.join(format!("{}.json", sanitized))
    }

    /// Load address book for a vault
    fn load_address_book(&self, vault_address: &str) -> Result<AddressBookData, String> {
        let file_path = self.get_vault_file_path(vault_address);

        if !file_path.exists() {
            return Ok(AddressBookData::new());
        }

        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read address book: {}", e))?;

        let data: AddressBookData = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse address book: {}", e))?;

        Ok(data)
    }

    /// Save address book for a vault
    fn save_address_book(&self, vault_address: &str, data: &AddressBookData) -> Result<(), String> {
        let file_path = self.get_vault_file_path(vault_address);

        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize address book: {}", e))?;

        fs::write(&file_path, json).map_err(|e| format!("Failed to write address book: {}", e))?;

        Ok(())
    }

    /// Add or update an address in the address book
    pub fn add_address(
        &self,
        vault_address: &str,
        address: String,
        label: Option<String>,
    ) -> Result<(), String> {
        let mut data = self.load_address_book(vault_address)?;

        let now = Utc::now();
        let entry = AddressEntry {
            address: address.clone(),
            label,
            last_used: now,
            created_at: data
                .addresses
                .get(&address)
                .map(|e| e.created_at)
                .unwrap_or(now),
        };

        data.addresses.insert(address, entry);
        self.save_address_book(vault_address, &data)
    }

    /// Get all addresses for a vault, sorted by last used (most recent first)
    pub fn get_addresses(&self, vault_address: &str) -> Result<Vec<AddressEntry>, String> {
        let data = self.load_address_book(vault_address)?;

        let mut entries: Vec<AddressEntry> = data.addresses.values().cloned().collect();
        entries.sort_by(|a, b| b.last_used.cmp(&a.last_used));

        Ok(entries)
    }

    /// Update the last used timestamp for an address
    pub fn update_last_used(&self, vault_address: &str, address: &str) -> Result<(), String> {
        let mut data = self.load_address_book(vault_address)?;

        if let Some(entry) = data.addresses.get_mut(address) {
            entry.last_used = Utc::now();
            self.save_address_book(vault_address, &data)?;
        }

        Ok(())
    }

    /// Update the label for an address
    pub fn update_label(
        &self,
        vault_address: &str,
        address: &str,
        label: Option<String>,
    ) -> Result<(), String> {
        let mut data = self.load_address_book(vault_address)?;

        if let Some(entry) = data.addresses.get_mut(address) {
            entry.label = label;
            self.save_address_book(vault_address, &data)?;
        } else {
            // If address doesn't exist, add it
            self.add_address(vault_address, address.to_string(), label)?;
        }

        Ok(())
    }

    /// Delete an address from the address book
    pub fn delete_address(&self, vault_address: &str, address: &str) -> Result<(), String> {
        let mut data = self.load_address_book(vault_address)?;

        data.addresses.remove(address);
        self.save_address_book(vault_address, &data)
    }

    /// Delete all addresses for a vault
    pub fn delete_all_addresses(&self, vault_address: &str) -> Result<(), String> {
        let file_path = self.get_vault_file_path(vault_address);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to delete address book: {}", e))?;
        }

        Ok(())
    }
}

impl Default for AddressBookService {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback: use current directory if data dir not available
            Self {
                data_dir: PathBuf::from("."),
            }
        })
    }
}
