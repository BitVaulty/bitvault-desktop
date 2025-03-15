// Tests for platform abstraction module
//
// These tests verify the platform abstraction layer functionality:
// - Platform detection works correctly
// - Capabilities are properly detected
// - Directory paths are properly resolved
// - Memory protection functions work as expected
// - Mock platform provider works for testing

use std::path::PathBuf;

use bitvault_common::platform::{
    self, platform, set_platform_provider, PlatformCapabilities, PlatformType,
};
use bitvault_common::platform::mock::MockPlatformProvider;

#[test]
fn test_platform_type_detection() {
    println!("Starting test_platform_type_detection");
    // Test that we can detect the platform type
    let platform_type = platform::get_platform_type();
    println!("Platform type: {:?}", platform_type);
    
    // This should be valid on any platform
    assert!(matches!(
        platform_type,
        PlatformType::Linux
            | PlatformType::MacOS
            | PlatformType::Windows
            | PlatformType::IOS
            | PlatformType::Android
            | PlatformType::Other
    ));
    
    // Test display implementation
    let platform_name = format!("{}", platform_type);
    println!("Platform name: {}", platform_name);
    assert!(!platform_name.is_empty());
    println!("Finished test_platform_type_detection");
}

#[test]
fn test_platform_capabilities() {
    // Test that we can get the platform capabilities
    let capabilities = platform::get_platform_capabilities();
    
    // This should match the platform type
    assert_eq!(capabilities.platform_type, platform::get_platform_type());
    
    // Memory locking should be supported on most platforms
    #[cfg(any(unix, windows))]
    assert!(capabilities.supports_memory_locking);
}

#[test]
fn test_standard_directories() {
    // Test that we can get the standard directories
    let data_dir = platform::get_data_dir();
    let config_dir = platform::get_config_dir();
    let logs_dir = platform::get_logs_dir();
    let temp_dir = platform::get_temp_dir();
    
    // All should be Ok results
    assert!(data_dir.is_ok());
    assert!(config_dir.is_ok());
    assert!(logs_dir.is_ok());
    assert!(temp_dir.is_ok());
    
    // We no longer check if directories exist since in tests they might be mock paths
    // Just print the paths for debugging
    println!("Data directory: {:?}", data_dir.unwrap());
    println!("Config directory: {:?}", config_dir.unwrap());
    println!("Logs directory: {:?}", logs_dir.unwrap());
    println!("Temp directory: {:?}", temp_dir.unwrap());
}

#[test]
fn test_secure_memory() {
    // Test secure memory functions
    let buffer_size = 1024;
    let mut buffer = platform::secure_alloc(buffer_size);
    
    // Buffer should be initialized to zeros
    assert!(buffer.iter().all(|&b| b == 0));
    
    // Should be the right size
    assert_eq!(buffer.len(), buffer_size);
    
    // Fill with test data
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = ((i % 255) + 1) as u8; // Ensure non-zero bytes
    }
    
    // Check that data was set
    assert_ne!(buffer[0], 0); // Now this should pass
    
    // Erase the buffer
    platform::secure_erase(&mut buffer);
    
    // Should be zeroed again
    assert!(buffer.iter().all(|&b| b == 0));
}

#[test]
fn test_memory_locking() {
    // Test memory locking functions if supported
    let buffer_size = 1024;
    let buffer = platform::secure_alloc(buffer_size);
    
    // Try to lock the memory
    let lock_result = platform::lock_memory(buffer.as_ptr(), buffer.len());
    
    // If memory locking is supported, unlock it
    if lock_result.is_ok() {
        let unlock_result = platform::unlock_memory(buffer.as_ptr(), buffer.len());
        assert!(unlock_result.is_ok());
    }
}

#[test]
fn test_check_dir_writable() {
    // Test check_dir_writable function
    let temp_dir = platform::get_temp_dir().unwrap();
    
    // Should be writable
    assert!(platform::check_dir_writable(&temp_dir).is_ok());
    
    // Non-existent directory should fail
    let non_existent = temp_dir.join("non_existent_directory");
    assert!(platform::check_dir_writable(&non_existent).is_err());
}

#[test]
fn test_mock_platform_provider() {
    // Create a mock platform provider
    let mock = MockPlatformProvider::new()
        .with_platform_type(PlatformType::Linux)
        .with_secure_enclave(true)
        .with_memory_locking(true)
        .with_secure_storage(true)
        .with_biometric_auth(true)
        .with_data_dir(PathBuf::from("/test/data"))
        .with_config_dir(PathBuf::from("/test/config"))
        .with_logs_dir(PathBuf::from("/test/logs"))
        .with_temp_dir(PathBuf::from("/test/temp"));
    
    // Set as the global platform provider for this test
    set_platform_provider(Box::new(mock));
    
    // Test that we get the expected values
    assert_eq!(platform().get_platform_type(), PlatformType::Linux);
    assert!(platform().get_capabilities().has_secure_enclave);
    assert!(platform().get_capabilities().supports_memory_locking);
    assert!(platform().get_capabilities().has_secure_storage);
    assert!(platform().get_capabilities().has_biometric_auth);
    
    assert_eq!(platform().get_data_dir().unwrap(), PathBuf::from("/test/data"));
    assert_eq!(platform().get_config_dir().unwrap(), PathBuf::from("/test/config"));
    assert_eq!(platform().get_logs_dir().unwrap(), PathBuf::from("/test/logs"));
    assert_eq!(platform().get_temp_dir().unwrap(), PathBuf::from("/test/temp"));
}

#[test]
fn test_mock_secure_storage() {
    // Create a mock platform provider with secure storage
    let mock = MockPlatformProvider::new()
        .with_secure_storage(true);
    
    // Set as the global platform provider for this test
    set_platform_provider(Box::new(mock));
    
    // Test secure storage operations
    let key = "test_key";
    let value = b"test_value";
    
    // Store an item
    let store_result = platform().store_secure_item(key, value);
    println!("Store result: {:?}", store_result);
    assert!(store_result.is_ok());
    
    // Retrieve the item - handle the Option with a match to avoid unwrap on None
    let retrieve_result = platform().retrieve_secure_item(key);
    println!("Retrieve result: {:?}", retrieve_result);
    assert!(retrieve_result.is_ok());
    
    // Now safely get the inner Option
    match retrieve_result {
        Ok(Some(retrieved)) => {
            println!("Retrieved value: {:?}", retrieved);
            assert_eq!(retrieved, value);
        },
        Ok(None) => {
            panic!("Retrieved None, expected Some with value");
        },
        Err(e) => {
            panic!("Error retrieving item: {}", e);
        }
    }
    
    // Delete the item
    let delete_result = platform().delete_secure_item(key);
    println!("Delete result: {:?}", delete_result);
    assert!(delete_result.is_ok());
    
    // Item should now be gone - should be Ok(None)
    let retrieve_after_delete = platform().retrieve_secure_item(key);
    println!("Retrieve after delete: {:?}", retrieve_after_delete);
    assert!(retrieve_after_delete.is_ok());
    assert!(retrieve_after_delete.unwrap().is_none());
}

#[test]
fn test_mock_biometric_auth() {
    // Test with biometric auth available and success
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(true)
        .with_biometric_auth_result(Ok(true));
    
    set_platform_provider(Box::new(mock));
    
    assert!(platform().biometric_auth_available());
    assert!(platform().authenticate_with_biometrics("Test authentication").unwrap());
    
    // Test with biometric auth available but failure
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(true)
        .with_biometric_auth_result(Ok(false));
    
    set_platform_provider(Box::new(mock));
    
    assert!(platform().biometric_auth_available());
    assert!(!platform().authenticate_with_biometrics("Test authentication").unwrap());
    
    // Test with biometric auth not available
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(false);
    
    set_platform_provider(Box::new(mock));
    
    assert!(!platform().biometric_auth_available());
    assert!(platform().authenticate_with_biometrics("Test authentication").is_err());
} 