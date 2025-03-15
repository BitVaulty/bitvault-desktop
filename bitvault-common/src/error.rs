//! Standardized error handling for BitVault
//!
//! This module provides a centralized approach to error handling for the BitVault wallet.
//! It defines a standardized error type hierarchy, proper context support, and ensures 
//! security-sensitive errors don't leak sensitive information.
//!
//! # Security Considerations
//!
//! - Error messages are carefully constructed to avoid leaking sensitive information
//! - Security-related errors are categorized separately with sanitized messaging
//! - Context is added to errors without exposing implementation details
//!
//! # Usage
//!
//! ```
//! use bitvault_common::error::{BitVaultError, ErrorContext};
//! use std::io;
//!
//! fn some_operation() -> Result<(), BitVaultError> {
//!     let file_result = std::fs::File::open("config.json")
//!         .context("Failed to open configuration file")?;
//!     
//!     // Rest of the operation...
//!     Ok(())
//! }
//! ```

use std::fmt;
use std::error::Error as StdError;
use thiserror::Error;
use crate::platform::error::PlatformError;
use crate::types::WalletError;
use std::io;

/// The main error type for the BitVault wallet application
///
/// This enum categorizes all possible errors that can occur in the application,
/// providing specific error types for different domains while maintaining
/// a consistent error handling approach.
#[derive(Debug, Error)]
pub enum BitVaultError {
    /// Wallet operation errors
    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),
    
    /// Network-related errors
    #[error("Network error: {context}")]
    Network {
        context: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    /// Configuration errors
    #[error("Configuration error: {context}")]
    Config {
        context: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    /// Security-related errors (carefully constructed to avoid leaking sensitive info)
    #[error("Security error: {context}")]
    Security {
        context: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    /// Platform-specific errors
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),
    
    /// Validation errors - Using String instead of ValidationError to avoid circular dependency
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// I/O errors
    #[error("I/O error: {context}")]
    Io {
        context: String,
        #[source]
        source: io::Error,
    },
    
    /// Serialization/deserialization errors
    #[error("Serialization error: {context}")]
    Serialization {
        context: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    /// External API errors
    #[error("External API error: {context}")]
    ExternalApi {
        context: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    /// Unexpected errors that don't fit other categories
    #[error("Unexpected error: {context}")]
    Unexpected {
        context: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

/// Extension trait for adding context to error types
pub trait ErrorContext<T, E> {
    /// Add context to an error
    ///
    /// This method allows adding human-readable context to an error result,
    /// making it more informative while preserving the original error as the source.
    fn context(self, context: impl Into<String>) -> Result<T, E>;
}

// Implement ErrorContext for generic errors (except io::Error which has a special implementation)
impl<T, E: Into<BitVaultError>> ErrorContext<T, BitVaultError> for Result<T, E> 
where
    E: 'static,
    E: std::fmt::Debug,
    E: std::any::Any,
{
    fn context(self, context: impl Into<String>) -> Result<T, BitVaultError> {
        self.map_err(|err| {
            let err = err.into();
            match err {
                BitVaultError::Network { source, .. } => BitVaultError::Network {
                    context: context.into(),
                    source,
                },
                BitVaultError::Config { source, .. } => BitVaultError::Config {
                    context: context.into(),
                    source,
                },
                BitVaultError::Security { source, .. } => BitVaultError::Security {
                    context: context.into(),
                    source,
                },
                BitVaultError::Io { source, .. } => BitVaultError::Io {
                    context: context.into(),
                    source,
                },
                BitVaultError::Serialization { source, .. } => BitVaultError::Serialization {
                    context: context.into(),
                    source,
                },
                BitVaultError::ExternalApi { source, .. } => BitVaultError::ExternalApi {
                    context: context.into(),
                    source,
                },
                BitVaultError::Unexpected { source, .. } => BitVaultError::Unexpected {
                    context: context.into(),
                    source,
                },
                // For other error types, wrap them in Unexpected
                _ => BitVaultError::Unexpected {
                    context: context.into(),
                    source: Some(Box::new(err)),
                },
            }
        })
    }
}

// Separate implementation for io::Error to avoid trait conflict
pub fn with_io_context<T>(result: Result<T, io::Error>, context: impl Into<String>) -> BitVaultResult<T> {
    result.map_err(|err| BitVaultError::Io {
        context: context.into(),
        source: err,
    })
}

// Implement From<io::Error> for BitVaultError
impl From<io::Error> for BitVaultError {
    fn from(err: io::Error) -> Self {
        BitVaultError::Io {
            context: err.to_string(),
            source: err,
        }
    }
}

// Implement From<serde_json::Error> for BitVaultError
impl From<serde_json::Error> for BitVaultError {
    fn from(err: serde_json::Error) -> Self {
        BitVaultError::Serialization {
            context: format!("JSON serialization error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// Implement From<toml::de::Error> for BitVaultError
impl From<toml::de::Error> for BitVaultError {
    fn from(err: toml::de::Error) -> Self {
        BitVaultError::Serialization {
            context: format!("TOML deserialization error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// Implement From<toml::ser::Error> for BitVaultError
impl From<toml::ser::Error> for BitVaultError {
    fn from(err: toml::ser::Error) -> Self {
        BitVaultError::Serialization {
            context: format!("TOML serialization error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

/// Create a new security error with context
///
/// This function is specifically for creating security-related errors,
/// ensuring they have appropriate context without leaking sensitive information.
pub fn security_error<S: Into<String>>(context: S) -> BitVaultError {
    BitVaultError::Security {
        context: context.into(),
        source: None,
    }
}

/// Create a new security error with context and source
pub fn security_error_with_source<S: Into<String>, E: StdError + Send + Sync + 'static>(
    context: S,
    source: E,
) -> BitVaultError {
    BitVaultError::Security {
        context: context.into(),
        source: Some(Box::new(source)),
    }
}

/// Create a new configuration error with context
pub fn config_error<S: Into<String>>(context: S) -> BitVaultError {
    BitVaultError::Config {
        context: context.into(),
        source: None,
    }
}

/// Create a new configuration error with context and source
pub fn config_error_with_source<S: Into<String>, E: StdError + Send + Sync + 'static>(
    context: S,
    source: E,
) -> BitVaultError {
    BitVaultError::Config {
        context: context.into(),
        source: Some(Box::new(source)),
    }
}

/// Create a new network error with context
pub fn network_error<S: Into<String>>(context: S) -> BitVaultError {
    BitVaultError::Network {
        context: context.into(),
        source: None,
    }
}

/// Create a new network error with context and source
pub fn network_error_with_source<S: Into<String>, E: StdError + Send + Sync + 'static>(
    context: S,
    source: E,
) -> BitVaultError {
    BitVaultError::Network {
        context: context.into(),
        source: Some(Box::new(source)),
    }
}

/// Type alias for a Result with BitVaultError
pub type BitVaultResult<T> = Result<T, BitVaultError>;

// Add conversions from other error types

// Implement From for NetworkStatusError
impl From<crate::network_status::NetworkStatusError> for BitVaultError {
    fn from(err: crate::network_status::NetworkStatusError) -> Self {
        BitVaultError::Network {
            context: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

// Implement From for FeeEstimationError
impl From<crate::fee_estimation::FeeEstimationError> for BitVaultError {
    fn from(err: crate::fee_estimation::FeeEstimationError) -> Self {
        BitVaultError::Network {
            context: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

// Implement From for ConfigError
impl From<crate::config_manager::ConfigError> for BitVaultError {
    fn from(err: crate::config_manager::ConfigError) -> Self {
        BitVaultError::Config {
            context: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

// Implement From for std::str::Utf8Error
impl From<std::str::Utf8Error> for BitVaultError {
    fn from(err: std::str::Utf8Error) -> Self {
        BitVaultError::Unexpected {
            context: format!("UTF-8 encoding error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// Implement From for std::string::FromUtf8Error
impl From<std::string::FromUtf8Error> for BitVaultError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        BitVaultError::Unexpected {
            context: format!("UTF-8 conversion error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// Implement From for bitcoin::consensus::encode::Error
impl From<bitcoin::consensus::encode::Error> for BitVaultError {
    fn from(err: bitcoin::consensus::encode::Error) -> Self {
        BitVaultError::Serialization {
            context: format!("Bitcoin serialization error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// Implement From for bitcoin::address::Error
impl From<bitcoin::address::Error> for BitVaultError {
    fn from(err: bitcoin::address::Error) -> Self {
        BitVaultError::Validation(format!("Invalid Bitcoin address: {}", err))
    }
}

// Implement From for bdk::Error
impl From<bdk::Error> for BitVaultError {
    fn from(err: bdk::Error) -> Self {
        BitVaultError::Wallet(WalletError::from(err))
    }
}

// Special handling for anyhow::Error using a conversion function
// instead of implementing From which conflicts with trait bounds
pub fn from_anyhow<T>(result: Result<T, anyhow::Error>, context: impl Into<String>) -> BitVaultResult<T> {
    match result {
        Ok(val) => Ok(val),
        Err(err) => Err(BitVaultError::Unexpected {
            context: context.into(),
            source: Some(Box::new(ErrorWrapper(err))),
        }),
    }
}

// Wrapper for anyhow::Error to implement StdError
#[derive(Debug)]
struct ErrorWrapper(anyhow::Error);

impl fmt::Display for ErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ErrorWrapper {}

/// Type alias for BitVaultResult with context information
///
/// Use this type for functions that would benefit from adding context to errors
pub type ContextResult<T> = Result<T, BitVaultError>;

/// Error category for logging and metrics purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Wallet operation errors
    Wallet,
    /// Network-related errors
    Network,
    /// Configuration errors
    Config,
    /// Security-related errors
    Security,
    /// Platform-specific errors
    Platform,
    /// Validation errors
    Validation,
    /// I/O errors
    Io,
    /// Serialization/deserialization errors
    Serialization,
    /// External API errors
    ExternalApi,
    /// Unexpected errors that don't fit other categories
    Unexpected,
}

impl ErrorCategory {
    /// Convert the error category to a string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            ErrorCategory::Wallet => "Wallet",
            ErrorCategory::Network => "Network",
            ErrorCategory::Config => "Config",
            ErrorCategory::Security => "Security",
            ErrorCategory::Platform => "Platform",
            ErrorCategory::Validation => "Validation",
            ErrorCategory::Io => "Io",
            ErrorCategory::Serialization => "Serialization",
            ErrorCategory::ExternalApi => "ExternalApi",
            ErrorCategory::Unexpected => "Unexpected",
        }
    }
}

impl BitVaultError {
    /// Get the category of this error for metrics and logging
    pub fn category(&self) -> ErrorCategory {
        match self {
            BitVaultError::Wallet(_) => ErrorCategory::Wallet,
            BitVaultError::Network { .. } => ErrorCategory::Network,
            BitVaultError::Config { .. } => ErrorCategory::Config,
            BitVaultError::Security { .. } => ErrorCategory::Security,
            BitVaultError::Platform(_) => ErrorCategory::Platform,
            BitVaultError::Validation(_) => ErrorCategory::Validation,
            BitVaultError::Io { .. } => ErrorCategory::Io,
            BitVaultError::Serialization { .. } => ErrorCategory::Serialization,
            BitVaultError::ExternalApi { .. } => ErrorCategory::ExternalApi,
            BitVaultError::Unexpected { .. } => ErrorCategory::Unexpected,
        }
    }
    
    /// Get a sanitized message suitable for displaying to users
    ///
    /// This method ensures that sensitive information is not leaked in error messages
    /// that might be shown to users in the UI.
    pub fn user_message(&self) -> String {
        match self {
            BitVaultError::Wallet(wallet_err) => {
                // Handle wallet errors specially to avoid leaking sensitive info
                match wallet_err {
                    WalletError::Crypto(_) | WalletError::Security(_) => {
                        "A security error occurred".to_string()
                    }
                    WalletError::InvalidDerivationPath(_) => {
                        "Invalid derivation path".to_string()
                    }
                    _ => wallet_err.to_string(),
                }
            }
            BitVaultError::Security { .. } => "A security error occurred".to_string(),
            BitVaultError::Network { context, .. } => {
                format!("Network error: {}", context)
            }
            BitVaultError::Config { context, .. } => {
                format!("Configuration error: {}", context)
            }
            BitVaultError::Platform(platform_err) => {
                // Don't leak security errors
                match platform_err {
                    PlatformError::SecurityError(_) => "A security error occurred".to_string(),
                    _ => platform_err.to_string(),
                }
            }
            BitVaultError::Validation(message) => format!("Validation error: {}", message),
            BitVaultError::Io { context, .. } => {
                format!("File operation error: {}", context)
            }
            BitVaultError::Serialization { context, .. } => {
                format!("Data format error: {}", context)
            }
            BitVaultError::ExternalApi { context, .. } => {
                format!("External service error: {}", context)
            }
            BitVaultError::Unexpected { context, .. } => {
                format!("Unexpected error: {}", context)
            }
        }
    }
}

/// Additions to the BitVaultError enum to handle specific variant constructors
impl BitVaultError {
    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        BitVaultError::Validation(message.into())
    }
    
    /// Create a new security validation error
    pub fn security_validation<S: Into<String>>(message: S) -> Self {
        security_error(format!("Security validation: {}", message.into()))
    }
} 