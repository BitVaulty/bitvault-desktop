//! Platform capabilities
//!
//! This module defines the capabilities of different platforms,
//! allowing the rest of the codebase to adapt to what's available.

use super::types::PlatformType;

/// Platform capabilities related to security and storage
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    /// Platform type
    pub platform_type: PlatformType,
    /// Whether this platform has a secure enclave or similar hardware
    pub has_secure_enclave: bool,
    /// Whether this platform supports memory locking
    pub supports_memory_locking: bool,
    /// Whether this platform has a secure storage API
    pub has_secure_storage: bool,
    /// Whether this platform has biometric authentication
    pub has_biometric_auth: bool,
}

impl PlatformCapabilities {
    /// Create new capabilities for a platform
    pub fn new(platform_type: PlatformType) -> Self {
        match platform_type {
            PlatformType::Linux => Self {
                platform_type,
                has_secure_enclave: false,
                supports_memory_locking: true,
                has_secure_storage: false,
                has_biometric_auth: false,
            },
            PlatformType::MacOS => Self {
                platform_type,
                has_secure_enclave: detect_macos_secure_enclave(),
                supports_memory_locking: true,
                has_secure_storage: true,
                has_biometric_auth: detect_macos_biometric(),
            },
            PlatformType::Windows => Self {
                platform_type,
                has_secure_enclave: false,
                supports_memory_locking: true,
                has_secure_storage: true,
                has_biometric_auth: detect_windows_biometric(),
            },
            PlatformType::IOS => Self {
                platform_type,
                has_secure_enclave: true,
                supports_memory_locking: true,
                has_secure_storage: true,
                has_biometric_auth: true,
            },
            PlatformType::Android => Self {
                platform_type,
                has_secure_enclave: detect_android_secure_hardware(),
                supports_memory_locking: true,
                has_secure_storage: true,
                has_biometric_auth: true,
            },
            PlatformType::Other => Self {
                platform_type,
                has_secure_enclave: false,
                supports_memory_locking: false,
                has_secure_storage: false,
                has_biometric_auth: false,
            },
        }
    }
}

/// Detect if macOS has a secure enclave
///
/// Not all Macs have secure enclaves, only those with T2 chips or Apple Silicon
#[cfg(target_os = "macos")]
fn detect_macos_secure_enclave() -> bool {
    // In a real implementation, we would detect actual hardware capability
    // For now, we'll be conservative and just check if it's Apple Silicon
    #[cfg(target_arch = "aarch64")]
    return true;

    #[cfg(not(target_arch = "aarch64"))]
    return false;
}

#[cfg(not(target_os = "macos"))]
fn detect_macos_secure_enclave() -> bool {
    false
}

/// Detect if macOS has biometric authentication (Touch ID)
#[cfg(target_os = "macos")]
fn detect_macos_biometric() -> bool {
    // In a real implementation, we would check if Touch ID is available
    // For now, we'll assume it is on Apple Silicon
    #[cfg(target_arch = "aarch64")]
    return true;

    #[cfg(not(target_arch = "aarch64"))]
    return false;
}

#[cfg(not(target_os = "macos"))]
fn detect_macos_biometric() -> bool {
    false
}

/// Detect if Windows has Windows Hello biometric
#[cfg(target_os = "windows")]
fn detect_windows_biometric() -> bool {
    // In a real implementation, we would check if Windows Hello is available
    // For now, we'll assume it is
    true
}

#[cfg(not(target_os = "windows"))]
fn detect_windows_biometric() -> bool {
    false
}

/// Detect if Android has secure hardware
#[cfg(target_os = "android")]
fn detect_android_secure_hardware() -> bool {
    // In a real implementation, we would check for hardware-backed keystore
    // For now, we'll assume it doesn't to be conservative
    false
}

#[cfg(not(target_os = "android"))]
fn detect_android_secure_hardware() -> bool {
    false
} 