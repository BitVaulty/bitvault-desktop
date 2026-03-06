//! Tests for error handling in desktop app

use bitvault_common::error::{BitVaultError, ErrorCategory};

#[test]
fn test_error_user_message_extraction() {
    let error = BitVaultError::Config("Invalid configuration".to_string());
    let message = error.user_message();
    assert_eq!(message, "Invalid configuration");
    assert!(!message.contains("Configuration error:"));
}

#[test]
fn test_error_category_mapping() {
    assert_eq!(
        BitVaultError::Config("test".to_string()).category(),
        ErrorCategory::Configuration
    );
    assert_eq!(
        BitVaultError::Network("test".to_string()).category(),
        ErrorCategory::Network
    );
    assert_eq!(
        BitVaultError::Storage("test".to_string()).category(),
        ErrorCategory::Storage
    );
}

#[test]
fn test_error_recoverability() {
    assert!(BitVaultError::Network("test".to_string()).is_recoverable());
    assert!(BitVaultError::Storage("test".to_string()).is_recoverable());
    assert!(!BitVaultError::Config("test".to_string()).is_recoverable());
    assert!(!BitVaultError::Validation("test".to_string()).is_recoverable());
}
