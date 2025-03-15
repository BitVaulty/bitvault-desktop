//! Platform type definitions
//!
//! This module defines the core types for platform identification and capabilities.

use std::fmt;

/// Detected platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    /// Linux platforms
    Linux,
    /// macOS platforms
    MacOS,
    /// Windows platforms
    Windows,
    /// iOS (iPhone, iPad)
    IOS,
    /// Android platforms
    Android,
    /// Unknown/other platforms
    Other,
}

impl fmt::Display for PlatformType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformType::Linux => write!(f, "Linux"),
            PlatformType::MacOS => write!(f, "macOS"),
            PlatformType::Windows => write!(f, "Windows"),
            PlatformType::IOS => write!(f, "iOS"),
            PlatformType::Android => write!(f, "Android"),
            PlatformType::Other => write!(f, "Other"),
        }
    }
}

/// Get the current platform type based on compile-time target information
pub fn get_platform_type() -> PlatformType {
    #[cfg(target_os = "linux")]
    {
        #[cfg(target_os = "android")]
        return PlatformType::Android;

        #[cfg(not(target_os = "android"))]
        return PlatformType::Linux;
    }

    #[cfg(target_os = "macos")]
    return PlatformType::MacOS;

    #[cfg(target_os = "windows")]
    return PlatformType::Windows;

    #[cfg(target_os = "ios")]
    return PlatformType::IOS;

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "ios",
        target_os = "android"
    )))]
    return PlatformType::Other;
} 