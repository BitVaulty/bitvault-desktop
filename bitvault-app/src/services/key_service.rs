//! KeyService - Key management and secure storage
//!
//! Equivalent to Swift's KeyService, provides secure storage for:
//! - Backup info (vault descriptors, mnemonics, etc.)
//! - PIN codes (encrypted)
//! - Email addresses
//! - Network selection
//! - Lock time settings
//!
//! Uses platform secure storage (keyring on Linux/Windows/macOS)

use keyring::Entry;
use serde::{Deserialize, Serialize};

/// Backup information for a vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub descriptor_mainnet: String,
    pub descriptor_testnet: String,
    pub mnemonic: String,
    pub name: String,
    pub vault_id: String,
    pub is_coowner: bool,
    pub hardware_wallet_types: Vec<String>,
    pub is_single_device: bool,
    pub email: Option<String>,
}

/// Key service for secure storage
/// Equivalent to Swift's KeyService
pub struct KeyService {
    service_name: String,
}

impl KeyService {
    /// Create a new key service with default service name
    pub fn new() -> Self {
        Self {
            service_name: "com.BitVault".to_string(),
        }
    }

    /// Create a new key service with a custom service name
    /// This is useful for testing to isolate test data
    /// 
    /// # Note
    /// This function is primarily intended for testing. In production code,
    /// use `KeyService::new()` which uses the standard service name.
    pub fn with_service_name(service_name: String) -> Self {
        Self { service_name }
    }

    /// Save backup info for a vault
    /// Equivalent to Swift's saveBackupInfo
    pub fn save_backup_info(
        &self,
        info: &BackupInfo,
        vault: &str,
        network: &str,
    ) -> Result<(), KeyServiceError> {
        let key = format!("backupInfo_{}_{}", network, vault);
        let entry = Entry::new(&self.service_name, &key)
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        let json = serde_json::to_string(info)
            .map_err(|e| KeyServiceError::SerializationError(e.to_string()))?;

        // Try to delete first to ensure overwrite works on all platforms
        // On some platforms (Linux Secret Service), set_password may not overwrite
        let _ = entry.delete_password();

        entry
            .set_password(&json)
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Get backup info for a vault
    /// Equivalent to Swift's getBackupInfo
    pub fn get_backup_info(
        &self,
        vault: &str,
        network: &str,
    ) -> Result<BackupInfo, KeyServiceError> {
        let key = format!("backupInfo_{}_{}", network, vault);
        let entry = Entry::new(&self.service_name, &key)
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        let json = entry
            .get_password()
            .map_err(|_| KeyServiceError::ReadError)?;

        let info: BackupInfo = serde_json::from_str(&json)
            .map_err(|e| KeyServiceError::DeserializationError(e.to_string()))?;

        Ok(info)
    }

    /// Delete backup info for a vault
    /// Equivalent to Swift's deleteBackupInfo
    pub fn delete_backup_info(&self, vault: &str, network: &str) -> Result<(), KeyServiceError> {
        let key = format!("backupInfo_{}_{}", network, vault);
        let entry = Entry::new(&self.service_name, &key)
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        entry
            .delete_password()
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Get all backup infos for a network
    /// Equivalent to Swift's getAllVaultsInfo
    pub fn get_all_vaults_info(&self, network: &str) -> Result<Vec<BackupInfo>, KeyServiceError> {
        let prefix = format!("backupInfo_{}_", network);
        let mut results = Vec::new();

        // Note: keyring doesn't support listing all keys directly
        // This is a limitation - we'd need to maintain a separate index
        // For now, return empty vec and let caller manage the list
        // In a full implementation, we'd use a separate storage mechanism
        // to track which vaults exist

        Ok(results)
    }

    /// Generate seed phrase
    /// Equivalent to Swift's generateSeedPhrase
    pub fn generate_seed_phrase(&self, is_long_phrase: bool) -> Result<String, KeyServiceError> {
        use bdk::keys::bip39::Mnemonic;

        let mnemonic = if is_long_phrase {
            Mnemonic::from_entropy(&rand::random::<[u8; 32]>())
        } else {
            Mnemonic::from_entropy(&rand::random::<[u8; 16]>())
        }
        .map_err(|e| KeyServiceError::GenerationError(e.to_string()))?;

        Ok(mnemonic.to_string())
    }

    /// Set lock time
    /// Equivalent to Swift's setLockTime
    pub fn set_lock_time(&self, lock_time: u64) -> Result<(), KeyServiceError> {
        let entry = Entry::new(&self.service_name, "locktime")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        // Try to delete first to ensure overwrite works on all platforms
        let _ = entry.delete_password();

        entry
            .set_password(&lock_time.to_string())
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Get lock time
    /// Equivalent to Swift's getLockTime
    pub fn get_lock_time(&self) -> Result<Option<u64>, KeyServiceError> {
        let entry = Entry::new(&self.service_name, "locktime")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        match entry.get_password() {
            Ok(value) => {
                let lock_time = value
                    .parse::<u64>()
                    .map_err(|_| KeyServiceError::ParseError)?;
                Ok(Some(lock_time))
            }
            Err(_) => Ok(None),
        }
    }

    /// Delete lock time
    /// Equivalent to Swift's deleteLockTime
    pub fn delete_lock_time(&self) -> Result<(), KeyServiceError> {
        let entry = Entry::new(&self.service_name, "locktime")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        entry
            .delete_password()
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Save network selection
    /// Equivalent to Swift's saveNetwork
    pub fn save_network(&self, network: &str) -> Result<(), KeyServiceError> {
        let entry = Entry::new(&self.service_name, "SelectedNetwork")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        // Try to delete first to ensure overwrite works on all platforms
        // On some platforms (Linux Secret Service), set_password may not overwrite
        let _ = entry.delete_password();

        entry
            .set_password(network)
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Get network selection
    /// Equivalent to Swift's getNetwork
    pub fn get_network(&self) -> Result<Option<String>, KeyServiceError> {
        let entry = Entry::new(&self.service_name, "SelectedNetwork")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }

    /// Delete network selection
    /// Equivalent to Swift's deleteNetwork
    pub fn delete_network(&self) -> Result<(), KeyServiceError> {
        let entry = Entry::new(&self.service_name, "SelectedNetwork")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        entry
            .delete_password()
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Save PIN code (encrypted)
    /// Equivalent to Swift's savePinCode
    /// Note: PIN encryption is handled by PinService in bitvault-common
    /// This method stores the encrypted PIN
    pub fn save_pin_code(&self, encrypted_pin: &str) -> Result<(), KeyServiceError> {
        let entry = Entry::new(&self.service_name, "pinCode")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        entry
            .set_password(encrypted_pin)
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Get PIN code (encrypted)
    /// Equivalent to Swift's getPinCode
    pub fn get_pin_code(&self) -> Result<Option<String>, KeyServiceError> {
        let entry = Entry::new(&self.service_name, "pinCode")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }

    /// Save email
    /// Equivalent to Swift's saveEmail
    pub fn save_email(&self, email: &str) -> Result<(), KeyServiceError> {
        let entry = Entry::new(&self.service_name, "email")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        // Try to delete first to ensure overwrite works on all platforms
        // On some platforms (Linux Secret Service), set_password may not overwrite
        let _ = entry.delete_password();

        entry
            .set_password(email)
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Get email
    /// Equivalent to Swift's getEmail
    pub fn get_email(&self) -> Result<Option<String>, KeyServiceError> {
        let entry = Entry::new(&self.service_name, "email")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }

    /// Delete email
    /// Equivalent to Swift's deleteEmail
    pub fn delete_email(&self) -> Result<(), KeyServiceError> {
        let entry = Entry::new(&self.service_name, "email")
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        entry
            .delete_password()
            .map_err(|e| KeyServiceError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Clear all keychain data
    /// Equivalent to Swift's clearAllKeychainData
    /// Note: keyring doesn't support bulk deletion, so this is a best-effort operation
    pub fn clear_all_keychain_data(&self) -> Result<(), KeyServiceError> {
        // Delete known keys
        let _ = self.delete_lock_time();
        let _ = self.delete_network();
        let _ = self.delete_email();

        // Note: Cannot delete backup info without knowing all vault addresses
        // This is a limitation of the keyring API

        Ok(())
    }
}

/// Errors that can occur during key service operations
#[derive(Debug, thiserror::Error)]
pub enum KeyServiceError {
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Read error: key not found")]
    ReadError,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Generation error: {0}")]
    GenerationError(String),
    #[error("Parse error")]
    ParseError,
}

impl Default for KeyService {
    fn default() -> Self {
        Self::new()
    }
}
