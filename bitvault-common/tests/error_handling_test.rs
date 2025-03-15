use bitvault_common::error::{BitVaultError, BitVaultResult, ErrorContext, with_io_context, from_anyhow, security_error};
use bitvault_common::types::WalletError;
use std::io;
use std::path::Path;
use std::fs::File;
use std::error::Error as StdError;
use anyhow::anyhow;

#[test]
fn test_error_context() {
    // Create a simple error
    let result: Result<(), _> = Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    
    // Add context to the error
    let with_context = with_io_context(result, "Failed to open config file");
    
    // Verify the error has the correct context
    match with_context {
        Ok(_) => panic!("Expected an error"),
        Err(e) => {
            assert!(matches!(e, BitVaultError::Io { .. }));
            assert!(e.to_string().contains("Failed to open config file"));
        }
    }
}

#[test]
fn test_error_categories() {
    // Create different error types
    let io_error = BitVaultError::Io {
        context: "IO error".to_string(),
        source: io::Error::new(io::ErrorKind::NotFound, "File not found"),
    };
    
    let security_error = BitVaultError::Security {
        context: "Security error".to_string(),
        source: None,
    };
    
    // Verify error categories
    assert_eq!(io_error.category().to_string(), "Io");
    assert_eq!(security_error.category().to_string(), "Security");
    
    // Verify user messages
    assert!(io_error.user_message().contains("File operation error"));
    assert_eq!(security_error.user_message(), "A security error occurred");
}

#[test]
fn test_validation_error() {
    // Create a validation error
    let error = BitVaultError::validation("Invalid input");
    
    // Verify error type and message
    assert!(matches!(error, BitVaultError::Validation(_)));
    assert!(error.to_string().contains("Invalid input"));
}

// Helper function that returns a Result
fn helper_function(succeed: bool) -> BitVaultResult<String> {
    if succeed {
        Ok("Success".to_string())
    } else {
        Err(BitVaultError::validation("Operation failed"))
    }
}

#[test]
fn test_error_propagation() {
    // Test successful case
    let result = helper_function(true);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");
    
    // Test error case
    let result = helper_function(false);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, BitVaultError::Validation(_)));
    assert!(err.to_string().contains("Operation failed"));
}

#[test]
fn test_security_error() {
    // Create a security error
    let error = security_error("Unauthorized access attempt");
    
    // Verify error type and message
    assert!(matches!(error, BitVaultError::Security { .. }));
    assert!(error.to_string().contains("Unauthorized access attempt"));
    
    // Check user message for security errors
    assert_eq!(error.user_message(), "A security error occurred");
    assert_eq!(error.category().to_string(), "Security");
}

#[test]
fn test_wallet_error() {
    // Create a domain-specific wallet error
    let address = "not-a-bitcoin-address";
    let error = BitVaultError::Wallet(
        WalletError::InvalidAddress(format!("Invalid address format: {}", address))
    );
    
    // Verify error type and message
    assert!(matches!(error, BitVaultError::Wallet(_)));
    assert!(error.to_string().contains("Invalid address format"));
    assert!(error.to_string().contains(address));
    
    // The WalletError::InvalidAddress directly returns its message
    assert!(error.user_message().contains("Invalid address"));
    assert_eq!(error.category().to_string(), "Wallet");
}

#[test]
fn test_anyhow_error_conversion() {
    // Create an anyhow error
    let anyhow_error = anyhow!("Something went wrong with anyhow");
    let anyhow_result: anyhow::Result<()> = Err(anyhow_error);
    
    // Convert to BitVaultError
    let bitvault_result = from_anyhow(anyhow_result, "Operation failed");
    
    // Verify error type and message
    assert!(bitvault_result.is_err());
    let error = bitvault_result.unwrap_err();
    
    // Check that error contains the correct messages
    assert!(error.to_string().contains("Operation failed"));
    
    // Check user message - should start with "Unexpected error: "
    assert!(error.user_message().contains("Unexpected error"));
}

#[test]
fn test_nonexistent_file_error() {
    // Attempt to open a nonexistent file
    let path = Path::new("/nonexistent/config.json");
    let file_result = File::open(path);
    
    // Add context to the error
    let with_context = with_io_context(
        file_result, 
        format!("Failed to open config file at '{}'", path.display())
    );
    
    // Verify the error has the correct context
    match with_context {
        Ok(_) => panic!("Expected an error"),
        Err(e) => {
            assert!(matches!(e, BitVaultError::Io { .. }));
            assert!(e.to_string().contains("Failed to open config file at"));
            assert!(e.to_string().contains("/nonexistent/config.json"));
            
            // Check user message
            assert!(e.user_message().contains("File operation error"));
        }
    }
}

// Read a config file with proper error handling
fn read_config_file(path: &Path) -> BitVaultResult<String> {
    let file_result = File::open(path);
    let mut file = with_io_context(file_result, format!("Failed to open config file at '{}'", path.display()))?;
    
    let mut contents = String::new();
    let read_result = std::io::Read::read_to_string(&mut file, &mut contents);
    with_io_context(read_result, "Failed to read config file contents")?;
    
    Ok(contents)
}

#[test]
fn test_error_propagation_chain() {
    // Attempt to read a nonexistent file through our helper function
    let path = Path::new("/nonexistent/test.json");
    let result = read_config_file(path);
    
    // Verify the error propagates correctly
    assert!(result.is_err());
    let error = result.unwrap_err();
    
    // Check error type and message
    assert!(matches!(error, BitVaultError::Io { .. }));
    assert!(error.to_string().contains("Failed to open config file at"));
    assert!(error.to_string().contains("/nonexistent/test.json"));
    
    // Verify it has a source using the StdError trait
    let has_source = error.source().is_some();
    assert!(has_source, "Error should have a source");
} 