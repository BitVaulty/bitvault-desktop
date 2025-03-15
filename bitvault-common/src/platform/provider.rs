//! Platform provider interface
//!
//! This module defines the core trait for platform-specific functionality.
//! All platform-specific implementations must implement this trait.

use std::io;
use std::path::{Path, PathBuf};

use super::capabilities::PlatformCapabilities;
use super::types::PlatformType;

/// Core trait defining the interface for platform-specific operations.
///
/// This trait encompasses all platform-specific functionality that the
/// BitVault wallet requires, including file path handling, secure storage,
/// memory protection, and platform-specific security features.
///
/// # Security Considerations
///
/// Implementations of this trait handle sensitive operations including:
/// - Secure memory management for cryptographic keys and secrets
/// - Platform-specific secure storage for credentials
/// - Biometric authentication (when available)
///
/// # Platform Support
///
/// BitVault supports multiple platforms with varying capabilities:
/// - Linux: Basic platform with optional hardware security module support
/// - macOS: Provides secure enclave, keychain, and biometric authentication
/// - Windows: Provides Windows Hello, DPAPI, and similar security features
/// - iOS/Android: Mobile platforms with secure enclaves and biometric auth
pub trait PlatformProvider: Send + Sync {
    /// Get the type of the current platform
    fn get_platform_type(&self) -> PlatformType;
    
    /// Get the capabilities of the current platform
    fn get_capabilities(&self) -> PlatformCapabilities;
    
    /// Get the data directory for the wallet
    ///
    /// This returns a platform-specific location that follows OS conventions:
    /// - Linux: ~/.local/share/bitvault or XDG_DATA_HOME/bitvault
    /// - macOS: ~/Library/Application Support/BitVault
    /// - Windows: %APPDATA%\BitVault
    ///
    /// # Returns
    /// * PathBuf containing the data directory path
    ///
    /// # Errors
    /// * Returns io::Error if the directory couldn't be determined or created
    fn get_data_dir(&self) -> io::Result<PathBuf>;
    
    /// Get the config directory for the wallet
    ///
    /// This returns a platform-specific location that follows OS conventions:
    /// - Linux: ~/.config/bitvault or XDG_CONFIG_HOME/bitvault
    /// - macOS: ~/Library/Application Support/BitVault
    /// - Windows: %APPDATA%\BitVault
    ///
    /// # Returns
    /// * PathBuf containing the config directory path
    ///
    /// # Errors
    /// * Returns io::Error if the directory couldn't be determined or created
    fn get_config_dir(&self) -> io::Result<PathBuf>;
    
    /// Get the logs directory for the wallet
    ///
    /// This returns a platform-specific location that follows OS conventions:
    /// - Linux: ~/.local/share/bitvault/logs or XDG_DATA_HOME/bitvault/logs
    /// - macOS: ~/Library/Logs/BitVault
    /// - Windows: %APPDATA%\BitVault\logs
    ///
    /// # Returns
    /// * PathBuf containing the logs directory path
    ///
    /// # Errors
    /// * Returns io::Error if the directory couldn't be determined or created
    fn get_logs_dir(&self) -> io::Result<PathBuf>;
    
    /// Get the temp directory for temporary wallet operations
    ///
    /// This returns a platform-specific location that is suitable for temporary files
    /// that will be securely deleted after use.
    ///
    /// # Returns
    /// * PathBuf containing the temp directory path
    ///
    /// # Errors
    /// * Returns io::Error if the directory couldn't be determined or created
    fn get_temp_dir(&self) -> io::Result<PathBuf>;
    
    /// Check if we can use secure memory features on this platform
    ///
    /// This checks if the platform supports memory locking or other
    /// secure memory features that can protect sensitive data.
    ///
    /// # Returns
    /// * true if secure memory features are available, false otherwise
    fn has_secure_memory(&self) -> bool;
    
    /// Lock memory to prevent swapping (if supported by the platform)
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the memory to lock
    /// * `len` - Length of the memory region to lock
    ///
    /// # Returns
    /// * Ok(()) if successful, Err with a message if not supported or failed
    ///
    /// # Security
    /// * This function is critical for protecting sensitive cryptographic material
    fn lock_memory(&self, ptr: *const u8, len: usize) -> Result<(), String>;
    
    /// Unlock previously locked memory
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the memory to unlock
    /// * `len` - Length of the memory region to unlock
    ///
    /// # Returns
    /// * Ok(()) if successful, Err with a message if not supported or failed
    fn unlock_memory(&self, ptr: *const u8, len: usize) -> Result<(), String>;
    
    /// Secure memory allocation for sensitive data
    ///
    /// Attempts to allocate memory in a way that reduces the risk of it being
    /// swapped to disk or appearing in core dumps.
    ///
    /// # Arguments
    /// * `size` - Size of the buffer to allocate in bytes
    ///
    /// # Returns
    /// * A Vec<u8> potentially with better security properties than regular allocations.
    ///
    /// # Security
    /// * The returned buffer should be used for storing sensitive data like keys
    fn secure_alloc(&self, size: usize) -> Vec<u8>;
    
    /// Securely erase a buffer
    ///
    /// Overwrites the buffer with zeros to remove sensitive data.
    ///
    /// # Arguments
    /// * `buffer` - Mutable reference to the buffer to erase
    ///
    /// # Security
    /// * This should use techniques that prevent compiler optimization of the zeroing
    fn secure_erase(&self, buffer: &mut [u8]);
    
    /// Check if a directory is writable by creating a temporary file in it
    ///
    /// # Arguments
    /// * `path` - The path to check for writability
    ///
    /// # Returns
    /// * Ok(()) if the directory is writable, Err with a message otherwise
    fn check_dir_writable(&self, path: &Path) -> Result<(), String>;
    
    /// Store an item in platform-specific secure storage
    ///
    /// # Arguments
    /// * `key` - The key to store the item under
    /// * `value` - The value to store
    ///
    /// # Returns
    /// * Ok(()) if successful, Err with a message if not supported or failed
    ///
    /// # Security
    /// * This uses platform-specific encryption (keychain, keyring, etc.)
    /// * Not all platforms support this feature, check capabilities first
    fn store_secure_item(&self, key: &str, value: &[u8]) -> Result<(), String>;
    
    /// Retrieve an item from platform-specific secure storage
    ///
    /// # Arguments
    /// * `key` - The key to retrieve
    ///
    /// # Returns
    /// * Ok(Some(value)) if the item exists and was retrieved
    /// * Ok(None) if the item doesn't exist
    /// * Err with a message if not supported or failed
    fn retrieve_secure_item(&self, key: &str) -> Result<Option<Vec<u8>>, String>;
    
    /// Delete an item from platform-specific secure storage
    ///
    /// # Arguments
    /// * `key` - The key to delete
    ///
    /// # Returns
    /// * Ok(()) if successful, Err with a message if not supported or failed
    fn delete_secure_item(&self, key: &str) -> Result<(), String>;
    
    /// Check if biometric authentication is available on this platform
    ///
    /// # Returns
    /// * true if biometric authentication is available, false otherwise
    fn biometric_auth_available(&self) -> bool;
    
    /// Authenticate using biometrics
    ///
    /// # Arguments
    /// * `reason` - The reason to display to the user
    ///
    /// # Returns
    /// * Ok(true) if authentication succeeded
    /// * Ok(false) if the user canceled or failed authentication
    /// * Err with a message if not supported or failed
    ///
    /// # Security
    /// * Uses platform-specific APIs for Touch ID, Face ID, Windows Hello, etc.
    fn authenticate_with_biometrics(&self, reason: &str) -> Result<bool, String>;
} 