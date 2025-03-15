//! Platform-specific error types
//!
//! This module defines error types for platform-specific operations.

use std::fmt;
use std::io;

/// Error type for platform operations
#[derive(Debug)]
pub enum PlatformError {
    /// Standard I/O error
    IoError(io::Error),
    /// Security-related error
    SecurityError(String),
    /// Operation not supported on this platform
    UnsupportedOperation(String),
    /// Configuration error
    ConfigurationError(String),
    /// Authentication error
    AuthenticationError(String),
    /// Permission error
    PermissionError(String),
    /// Storage error
    StorageError(String),
    /// Memory error
    MemoryError(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::IoError(e) => write!(f, "I/O error: {}", e),
            PlatformError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            PlatformError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            PlatformError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            PlatformError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            PlatformError::PermissionError(msg) => write!(f, "Permission error: {}", msg),
            PlatformError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            PlatformError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
        }
    }
}

impl std::error::Error for PlatformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PlatformError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for PlatformError {
    fn from(err: io::Error) -> Self {
        PlatformError::IoError(err)
    }
}

impl From<String> for PlatformError {
    fn from(msg: String) -> Self {
        PlatformError::SecurityError(msg)
    }
}

impl From<&str> for PlatformError {
    fn from(msg: &str) -> Self {
        PlatformError::SecurityError(msg.to_string())
    }
}

/// Result type for platform operations
pub type PlatformResult<T> = Result<T, PlatformError>; 