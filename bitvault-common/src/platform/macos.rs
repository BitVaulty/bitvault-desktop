//! macOS platform implementation
//!
//! This module provides macOS-specific implementations of platform features.
//! Currently, this is a minimal implementation that will be expanded in the future.

use std::io;
use std::path::{Path, PathBuf};

use super::capabilities::PlatformCapabilities;
use super::error::{PlatformError, PlatformResult};
use super::memory;
use super::paths;
use super::provider::PlatformProvider;
use super::types::{get_platform_type, PlatformType};

/// macOS-specific platform implementation
pub struct MacOSPlatformProvider {
    capabilities: PlatformCapabilities,
}

impl MacOSPlatformProvider {
    /// Create a new macOS platform provider
    pub fn new() -> Self {
        let platform_type = get_platform_type();
        let capabilities = PlatformCapabilities::new(platform_type);
        Self { capabilities }
    }
}

impl PlatformProvider for MacOSPlatformProvider {
    fn get_platform_type(&self) -> PlatformType {
        self.capabilities.platform_type
    }

    fn get_capabilities(&self) -> PlatformCapabilities {
        self.capabilities.clone()
    }

    fn get_data_dir(&self) -> io::Result<PathBuf> {
        paths::get_macos_data_dir()
    }

    fn get_config_dir(&self) -> io::Result<PathBuf> {
        paths::get_desktop_config_dir(PlatformType::MacOS)
    }

    fn get_logs_dir(&self) -> io::Result<PathBuf> {
        paths::get_macos_logs_dir()
    }

    fn get_temp_dir(&self) -> io::Result<PathBuf> {
        paths::get_temp_dir()
    }

    fn has_secure_memory(&self) -> bool {
        self.capabilities.supports_memory_locking
    }

    fn lock_memory(&self, ptr: *const u8, len: usize) -> Result<(), String> {
        #[cfg(unix)]
        {
            match memory::lock_memory_unix(ptr, len) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // Handle unsupported operations differently than other errors
                    if let PlatformError::UnsupportedOperation(msg) = e {
                        // This is expected on some platforms or configurations, return a more informative message
                        Err(format!("Memory locking not supported: {}", msg))
                    } else {
                        Err(e.to_string())
                    }
                }
            }
        }

        #[cfg(not(unix))]
        {
            // This should never happen on macOS, but just in case:
            Err("Memory locking not implemented for this platform".to_string())
        }
    }

    fn unlock_memory(&self, ptr: *const u8, len: usize) -> Result<(), String> {
        #[cfg(unix)]
        {
            match memory::unlock_memory_unix(ptr, len) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // Handle unsupported operations differently than other errors
                    if let PlatformError::UnsupportedOperation(msg) = e {
                        // This is expected on some platforms or configurations, return a more informative message
                        Err(format!("Memory unlocking not supported: {}", msg))
                    } else {
                        Err(e.to_string())
                    }
                }
            }
        }

        #[cfg(not(unix))]
        {
            // This should never happen on macOS, but just in case:
            Err("Memory unlocking not implemented for this platform".to_string())
        }
    }

    fn secure_alloc(&self, size: usize) -> Vec<u8> {
        memory::secure_alloc(size, self.has_secure_memory())
    }

    fn secure_erase(&self, buffer: &mut [u8]) {
        memory::secure_erase(buffer)
    }

    fn check_dir_writable(&self, path: &Path) -> Result<(), String> {
        paths::check_dir_writable(path).map_err(|e| e.to_string())
    }

    fn store_secure_item(&self, key: &str, value: &[u8]) -> Result<(), String> {
        // On macOS, we would use the Keychain API
        // For now, return an error since we haven't implemented this yet
        Err("Secure storage not yet implemented on macOS".to_string())
    }

    fn retrieve_secure_item(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        // On macOS, we would use the Keychain API
        // For now, return an error since we haven't implemented this yet
        Err("Secure storage not yet implemented on macOS".to_string())
    }

    fn delete_secure_item(&self, key: &str) -> Result<(), String> {
        // On macOS, we would use the Keychain API
        // For now, return an error since we haven't implemented this yet
        Err("Secure storage not yet implemented on macOS".to_string())
    }

    fn biometric_auth_available(&self) -> bool {
        // Touch ID availability would be checked here
        self.capabilities.has_biometric_auth
    }

    fn authenticate_with_biometrics(&self, _reason: &str) -> Result<bool, String> {
        // Touch ID would be used here
        Err("Biometric authentication not yet implemented on macOS".to_string())
    }
} 