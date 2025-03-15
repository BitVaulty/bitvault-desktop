//! Platform abstraction module
//!
//! This module provides abstractions for platform-specific operations:
//! - File paths and directory handling
//! - Secure storage locations
//! - Memory protection
//! - OS-specific security features
//!
//! By using these abstractions, the rest of the codebase can remain
//! platform-agnostic while still taking advantage of platform-specific
//! security features when available.
//!
//! # Usage
//!
//! Access platform functionality via the global platform() function:
//!
//! ```rust
//! use bitvault_common::platform;
//!
//! let data_dir = platform().get_data_dir().expect("Could not get data directory");
//! ```
//!
//! For backward compatibility, the module also provides direct functions:
//!
//! ```rust
//! use bitvault_common::platform;
//!
//! let data_dir = platform::get_data_dir().expect("Could not get data directory");
//! ```
//!
//! # Security boundaries
//!
//! This module handles several security-sensitive operations:
//!
//! 1. Memory protection for cryptographic keys and sensitive data
//! 2. Access to platform secure storage (keychain, keyring, etc.)
//! 3. Directory management for wallet data and configuration
//!
//! Code that handles sensitive cryptographic material should always use
//! the secure memory functions provided by this module.

pub mod capabilities;
pub mod error;
pub mod memory;
pub mod paths;
pub mod provider;
pub mod types;

// Platform-specific implementations
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "ios")]
pub mod ios;
#[cfg(target_os = "android")]
pub mod android;

// Generic implementation for other platforms
pub mod generic;

// Make mock module public without cfg(test) restriction
pub mod mock;

use std::cell::RefCell;
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use provider::PlatformProvider;

// Re-export key types
pub use error::{PlatformError, PlatformResult};
pub use types::PlatformType;
pub use capabilities::PlatformCapabilities;

// For testing, also export the mock provider - no longer needed since module is public
// #[cfg(test)]
// pub use mock::MockPlatformProvider;

// Create the appropriate platform provider for the current platform
fn create_platform_provider() -> Box<dyn PlatformProvider> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatformProvider::new())
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSPlatformProvider::new())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatformProvider::new())
    }
    #[cfg(target_os = "ios")]
    {
        Box::new(ios::IOSPlatformProvider::new())
    }
    #[cfg(target_os = "android")]
    {
        Box::new(android::AndroidPlatformProvider::new())
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "ios",
        target_os = "android"
    )))]
    {
        Box::new(generic::GenericPlatformProvider::new())
    }
}

// Global provider instance - using a Mutex rather than Arc
static PLATFORM_PROVIDER: Lazy<Mutex<Box<dyn PlatformProvider>>> = 
    Lazy::new(|| Mutex::new(create_platform_provider()));

// Flag to indicate if test provider is active
static TEST_PROVIDER_ACTIVE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

// Actual test provider instance
static TEST_PROVIDER: Lazy<Mutex<Option<Box<dyn PlatformProvider>>>> = 
    Lazy::new(|| Mutex::new(None));

/// Wrapper struct for returning a platform provider reference
pub struct PlatformProviderRef {
    // Hold a reference to the platform provider
    provider: &'static dyn PlatformProvider,
}

impl Deref for PlatformProviderRef {
    type Target = dyn PlatformProvider;

    fn deref(&self) -> &Self::Target {
        self.provider
    }
}

/// Get the global platform provider instance
///
/// This provides access to all platform-specific functionality
///
/// # Examples
///
/// ```
/// use bitvault_common::platform;
///
/// let data_dir = platform().get_data_dir().expect("Could not get data directory");
/// ```
pub fn platform() -> PlatformProviderRef {
    // This is a bit of a hack, but it's the only way to return a reference with 'static lifetime
    // We're using the fact that our platform providers live for the entire program lifetime
    
    // Try to get the test provider if active
    if let Ok(active) = TEST_PROVIDER_ACTIVE.lock() {
        if *active {
            if let Ok(test_provider) = TEST_PROVIDER.lock() {
                if let Some(ref provider) = *test_provider {
                    // Cast the &Box<dyn PlatformProvider> to a &'static dyn PlatformProvider
                    // SAFETY: This is safe because the test provider lives for the program lifetime
                    let provider_ref = unsafe { 
                        std::mem::transmute::<&dyn PlatformProvider, &'static dyn PlatformProvider>(provider.as_ref())
                    };
                    
                    return PlatformProviderRef {
                        provider: provider_ref,
                    };
                }
            }
        }
    }
    
    // Use the global provider
    // SAFETY: This is safe because the global provider lives for the program lifetime
    let provider_ref = unsafe {
        if let Ok(provider) = PLATFORM_PROVIDER.lock() {
            std::mem::transmute::<&dyn PlatformProvider, &'static dyn PlatformProvider>(provider.as_ref())
        } else {
            // If we can't get the lock, panic - this should never happen
            panic!("Failed to acquire lock on platform provider");
        }
    };
    
    PlatformProviderRef {
        provider: provider_ref,
    }
}

/// For tests, allow replacing the global instance
pub fn set_platform_provider(provider: Box<dyn PlatformProvider>) {
    println!("Setting test platform provider");
    
    // Activate the test provider
    if let Ok(mut active) = TEST_PROVIDER_ACTIVE.lock() {
        *active = true;
    }
    
    // Set the test provider
    if let Ok(mut test_provider) = TEST_PROVIDER.lock() {
        *test_provider = Some(provider);
        println!("Test platform provider set successfully");
    } else {
        println!("Failed to set test platform provider - couldn't acquire lock");
    }
}

// Backward compatibility layer

/// Get the current platform type
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn get_platform_type() -> PlatformType {
    platform().get_platform_type()
}

/// Get capabilities of the current platform
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn get_platform_capabilities() -> PlatformCapabilities {
    platform().get_capabilities()
}

/// Get the data directory for the wallet
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn get_data_dir() -> io::Result<PathBuf> {
    platform().get_data_dir()
}

/// Get the config directory for the wallet
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn get_config_dir() -> io::Result<PathBuf> {
    platform().get_config_dir()
}

/// Get the logs directory for the wallet
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn get_logs_dir() -> io::Result<PathBuf> {
    platform().get_logs_dir()
}

/// Get the temp directory for temporary wallet operations
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn get_temp_dir() -> io::Result<PathBuf> {
    platform().get_temp_dir()
}

/// Check if we can use secure memory features on this platform
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn has_secure_memory() -> bool {
    platform().has_secure_memory()
}

/// Lock memory to prevent swapping (if supported by the platform)
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn lock_memory(ptr: *const u8, len: usize) -> Result<(), String> {
    platform().lock_memory(ptr, len)
}

/// Unlock previously locked memory
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn unlock_memory(ptr: *const u8, len: usize) -> Result<(), String> {
    platform().unlock_memory(ptr, len)
}

/// Secure memory allocation for sensitive data
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn secure_alloc(size: usize) -> Vec<u8> {
    platform().secure_alloc(size)
}

/// Securely erase a buffer
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn secure_erase(buffer: &mut [u8]) {
    platform().secure_erase(buffer)
}

/// Check if a directory is writable by creating a temporary file in it
///
/// This is a convenience function that calls the same method on the global
/// platform provider instance.
pub fn check_dir_writable(path: &Path) -> Result<(), String> {
    platform().check_dir_writable(path)
} 