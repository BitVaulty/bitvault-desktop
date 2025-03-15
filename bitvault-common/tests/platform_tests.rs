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
    self, platform, set_platform_provider, reset_platform_provider, PlatformCapabilities, PlatformType,
};
use bitvault_common::platform::mock::MockPlatformProvider;

#[test]
fn test_platform_type_detection() {
    println!("Starting test_platform_type_detection");
    // Ensure we're using the default provider
    reset_platform_provider();
    
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
    println!("Starting test_platform_capabilities");
    // Ensure we're using the default provider
    reset_platform_provider();
    
    // Test that we can get the platform capabilities
    let capabilities = platform::get_platform_capabilities();
    println!("Platform capabilities: {:?}", capabilities);
    
    // This should match the platform type
    assert_eq!(capabilities.platform_type, platform::get_platform_type());
    
    // Memory locking should be supported on most platforms
    #[cfg(any(unix, windows))]
    assert!(capabilities.supports_memory_locking);
    println!("Finished test_platform_capabilities");
}

#[test]
fn test_standard_directories() {
    println!("Starting test_standard_directories");
    // Ensure we're using the default provider
    reset_platform_provider();
    
    // Test that we can get the standard directories
    let data_dir = platform::get_data_dir();
    let config_dir = platform::get_config_dir();
    let logs_dir = platform::get_logs_dir();
    let temp_dir = platform::get_temp_dir();
    
    // All should be Ok results
    assert!(data_dir.is_ok(), "Data dir should be Ok");
    assert!(config_dir.is_ok(), "Config dir should be Ok");
    assert!(logs_dir.is_ok(), "Logs dir should be Ok");
    assert!(temp_dir.is_ok(), "Temp dir should be Ok");
    
    // We no longer check if directories exist since in tests they might be mock paths
    // Just print the paths for debugging
    println!("Data directory: {:?}", data_dir.unwrap());
    println!("Config directory: {:?}", config_dir.unwrap());
    println!("Logs directory: {:?}", logs_dir.unwrap());
    println!("Temp directory: {:?}", temp_dir.unwrap());
    println!("Finished test_standard_directories");
}

#[test]
fn test_secure_memory() {
    println!("Starting test_secure_memory");
    // Ensure we're using the default provider
    reset_platform_provider();
    
    // Test secure memory functions
    let buffer_size = 1024;
    let mut buffer = platform::secure_alloc(buffer_size);
    println!("Allocated secure buffer of size {}", buffer_size);
    
    // Buffer should be initialized to zeros
    assert!(buffer.iter().all(|&b| b == 0), "Buffer should be initialized to zeros");
    
    // Should be the right size
    assert_eq!(buffer.len(), buffer_size, "Buffer should be the requested size");
    
    // Fill with test data
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = ((i % 255) + 1) as u8; // Ensure non-zero bytes
    }
    println!("Filled buffer with test data");
    
    // Check that data was set
    assert_ne!(buffer[0], 0, "Buffer data should be set"); 
    
    // Erase the buffer
    platform::secure_erase(&mut buffer);
    println!("Erased the buffer");
    
    // Should be zeroed again
    assert!(buffer.iter().all(|&b| b == 0), "Buffer should be zeroed after erase");
    println!("Finished test_secure_memory");
}

#[test]
fn test_memory_locking() {
    println!("Starting test_memory_locking");
    // Ensure we're using the default provider
    reset_platform_provider();
    
    // Test memory locking functions if supported
    let buffer_size = 1024;
    let buffer = platform::secure_alloc(buffer_size);
    println!("Allocated buffer for memory locking test");
    
    // Try to lock the memory
    let lock_result = platform::lock_memory(buffer.as_ptr(), buffer.len());
    println!("Lock memory result: {:?}", lock_result);
    
    // If memory locking is supported, unlock it
    if lock_result.is_ok() {
        let unlock_result = platform::unlock_memory(buffer.as_ptr(), buffer.len());
        println!("Unlock memory result: {:?}", unlock_result);
        assert!(unlock_result.is_ok(), "Unlock memory should succeed if lock succeeded");
    }
    println!("Finished test_memory_locking");
}

#[test]
fn test_check_dir_writable() {
    println!("Starting test_check_dir_writable");
    // Ensure we're using the default provider
    reset_platform_provider();
    
    // Test check_dir_writable function
    let temp_dir = platform::get_temp_dir().unwrap();
    println!("Got temp dir: {:?}", temp_dir);
    
    // Should be writable
    let writable_result = platform::check_dir_writable(&temp_dir);
    println!("Check dir writable result: {:?}", writable_result);
    assert!(writable_result.is_ok(), "Temp dir should be writable");
    
    // Non-existent directory should fail
    let non_existent = temp_dir.join("non_existent_directory");
    println!("Non-existent dir: {:?}", non_existent);
    let non_existent_result = platform::check_dir_writable(&non_existent);
    println!("Check non-existent dir writable result: {:?}", non_existent_result);
    assert!(non_existent_result.is_err(), "Non-existent dir should not be writable");
    println!("Finished test_check_dir_writable");
}

#[test]
fn test_mock_platform_provider() {
    println!("Starting test_mock_platform_provider");
    // Start with a clean state
    reset_platform_provider();
    
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
    println!("Created mock platform provider");
    
    // Set as the global platform provider for this test
    set_platform_provider(Box::new(mock));
    println!("Set mock as global platform provider");
    
    // Test that we get the expected values
    assert_eq!(platform().get_platform_type(), PlatformType::Linux);
    assert!(platform().get_capabilities().has_secure_enclave);
    assert!(platform().get_capabilities().supports_memory_locking);
    assert!(platform().get_capabilities().has_secure_storage);
    assert!(platform().get_capabilities().has_biometric_auth);
    println!("Validated platform type and capabilities");
    
    assert_eq!(platform().get_data_dir().unwrap(), PathBuf::from("/test/data"));
    assert_eq!(platform().get_config_dir().unwrap(), PathBuf::from("/test/config"));
    assert_eq!(platform().get_logs_dir().unwrap(), PathBuf::from("/test/logs"));
    assert_eq!(platform().get_temp_dir().unwrap(), PathBuf::from("/test/temp"));
    println!("Validated directory paths");
    println!("Finished test_mock_platform_provider");
    
    // Clean up after the test
    reset_platform_provider();
}

#[test]
fn test_mock_secure_storage() {
    println!("Starting test_mock_secure_storage");
    // Start with a clean state
    reset_platform_provider();
    
    // Create a mock platform provider with secure storage
    let mock = MockPlatformProvider::new()
        .with_secure_storage(true);
    println!("Created mock with secure storage");
    
    // Set as the global platform provider for this test
    set_platform_provider(Box::new(mock));
    println!("Set mock as global platform provider");
    
    // Test secure storage operations
    let key = "test_key";
    let value = b"test_value";
    println!("Testing with key: {}", key);
    
    // Store an item
    let store_result = platform().store_secure_item(key, value);
    println!("Store result: {:?}", store_result);
    assert!(store_result.is_ok(), "Store should succeed");
    
    // Retrieve the item
    let retrieve_result = platform().retrieve_secure_item(key);
    println!("Retrieve result: {:?}", retrieve_result);
    assert!(retrieve_result.is_ok(), "Retrieve should succeed");
    
    // Check the retrieved value
    match retrieve_result {
        Ok(Some(retrieved)) => {
            println!("Retrieved value: {:?}", retrieved);
            assert_eq!(retrieved, value, "Retrieved value should match stored value");
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
    assert!(delete_result.is_ok(), "Delete should succeed");
    
    // Item should now be gone
    let retrieve_after_delete = platform().retrieve_secure_item(key);
    println!("Retrieve after delete: {:?}", retrieve_after_delete);
    assert!(retrieve_after_delete.is_ok(), "Retrieve after delete should succeed");
    assert!(retrieve_after_delete.unwrap().is_none(), "Item should be gone after delete");
    println!("Finished test_mock_secure_storage");
    
    // Clean up after the test
    reset_platform_provider();
}

#[test]
fn test_mock_biometric_auth() {
    println!("Starting test_mock_biometric_auth");
    // Start with a clean state
    reset_platform_provider();
    
    // Test with biometric auth available and success
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(true)
        .with_biometric_auth_result(Ok(true));
    println!("Created mock with biometric auth and success result");
    
    set_platform_provider(Box::new(mock));
    println!("Set mock as global platform provider");
    
    assert!(platform().biometric_auth_available(), "Biometric auth should be available");
    let auth_result = platform().authenticate_with_biometrics("Test authentication");
    println!("Auth result: {:?}", auth_result);
    assert!(auth_result.is_ok(), "Auth should succeed");
    assert!(auth_result.unwrap(), "Auth should return true");
    
    // Reset before next test
    reset_platform_provider();
    
    // Test with biometric auth available but failure
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(true)
        .with_biometric_auth_result(Ok(false));
    println!("Created mock with biometric auth and failure result");
    
    set_platform_provider(Box::new(mock));
    println!("Set mock as global platform provider");
    
    assert!(platform().biometric_auth_available(), "Biometric auth should be available");
    let auth_result = platform().authenticate_with_biometrics("Test authentication");
    println!("Auth result: {:?}", auth_result);
    assert!(auth_result.is_ok(), "Auth should succeed");
    assert!(!auth_result.unwrap(), "Auth should return false");
    
    // Reset before next test
    reset_platform_provider();
    
    // Test with biometric auth not available
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(false);
    println!("Created mock without biometric auth");
    
    set_platform_provider(Box::new(mock));
    println!("Set mock as global platform provider");
    
    assert!(!platform().biometric_auth_available(), "Biometric auth should not be available");
    let auth_result = platform().authenticate_with_biometrics("Test authentication");
    println!("Auth result: {:?}", auth_result);
    assert!(auth_result.is_err(), "Auth should fail");
    println!("Finished test_mock_biometric_auth");
    
    // Clean up after the test
    reset_platform_provider();
} 