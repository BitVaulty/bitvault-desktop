//! Generic platform implementation
//!
//! This module provides a fallback implementation for platforms that don't have
//! specific support. It implements basic functionality where possible but
//! will not provide advanced features like secure storage or biometric authentication.

use std::io;
use std::path::{Path, PathBuf};

use super::capabilities::PlatformCapabilities;
use super::memory;
use super::paths;
use super::provider::PlatformProvider;
use super::types::{get_platform_type, PlatformType};

/// Generic implementation for platforms without specific support
///
/// This implementation provides basic functionality but will not support
/// advanced features that require platform-specific APIs.
pub struct GenericPlatformProvider {
    capabilities: PlatformCapabilities,
}

impl GenericPlatformProvider {
    /// Create a new generic platform provider
    pub fn new() -> Self {
        let platform_type = get_platform_type();
        let capabilities = PlatformCapabilities::new(platform_type);
        Self { capabilities }
    }
}

impl PlatformProvider for GenericPlatformProvider {
    fn get_platform_type(&self) -> PlatformType {
        self.capabilities.platform_type
    }

    fn get_capabilities(&self) -> PlatformCapabilities {
        self.capabilities.clone()
    }

    fn get_data_dir(&self) -> io::Result<PathBuf> {
        paths::get_other_data_dir()
    }

    fn get_config_dir(&self) -> io::Result<PathBuf> {
        // Use the same directory as data on generic platforms
        self.get_data_dir()
    }

    fn get_logs_dir(&self) -> io::Result<PathBuf> {
        let data_dir = self.get_data_dir()?;
        paths::get_default_logs_dir(&data_dir)
    }

    fn get_temp_dir(&self) -> io::Result<PathBuf> {
        paths::get_temp_dir()
    }

    fn has_secure_memory(&self) -> bool {
        self.capabilities.supports_memory_locking
    }

    fn lock_memory(&self, _ptr: *const u8, _len: usize) -> Result<(), String> {
        // Generic implementation doesn't support memory locking
        Err("Memory locking not supported on this platform".to_string())
    }

    fn unlock_memory(&self, _ptr: *const u8, _len: usize) -> Result<(), String> {
        // Generic implementation doesn't support memory unlocking
        Err("Memory unlocking not supported on this platform".to_string())
    }

    fn secure_alloc(&self, size: usize) -> Vec<u8> {
        // Generic secure allocation without memory locking
        memory::secure_alloc(size, false)
    }

    fn secure_erase(&self, buffer: &mut [u8]) {
        memory::secure_erase(buffer)
    }

    fn check_dir_writable(&self, path: &Path) -> Result<(), String> {
        paths::check_dir_writable(path).map_err(|e| e.to_string())
    }

    fn store_secure_item(&self, _key: &str, _value: &[u8]) -> Result<(), String> {
        // Generic implementation doesn't support secure storage
        Err("Secure storage not supported on this platform".to_string())
    }

    fn retrieve_secure_item(&self, _key: &str) -> Result<Option<Vec<u8>>, String> {
        // Generic implementation doesn't support secure storage
        Err("Secure storage not supported on this platform".to_string())
    }

    fn delete_secure_item(&self, _key: &str) -> Result<(), String> {
        // Generic implementation doesn't support secure storage
        Err("Secure storage not supported on this platform".to_string())
    }

    fn biometric_auth_available(&self) -> bool {
        false
    }

    fn authenticate_with_biometrics(&self, _reason: &str) -> Result<bool, String> {
        // Generic implementation doesn't support biometric authentication
        Err("Biometric authentication not supported on this platform".to_string())
    }
} 