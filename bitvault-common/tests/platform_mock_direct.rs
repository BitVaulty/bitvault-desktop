// Test for MockPlatformProvider directly
//
// This test verifies the MockPlatformProvider functionality directly without setting it as global.

use std::path::PathBuf;
use bitvault_common::platform::PlatformType;
use bitvault_common::platform::mock::MockPlatformProvider;
use bitvault_common::platform::provider::PlatformProvider;

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
    
    // Test that we get the expected values directly from the mock
    assert_eq!(mock.get_platform_type(), PlatformType::Linux);
    assert!(mock.get_capabilities().has_secure_enclave);
    assert!(mock.get_capabilities().supports_memory_locking);
    assert!(mock.get_capabilities().has_secure_storage);
    assert!(mock.get_capabilities().has_biometric_auth);
    
    assert_eq!(mock.get_data_dir().unwrap(), PathBuf::from("/test/data"));
    assert_eq!(mock.get_config_dir().unwrap(), PathBuf::from("/test/config"));
    assert_eq!(mock.get_logs_dir().unwrap(), PathBuf::from("/test/logs"));
    assert_eq!(mock.get_temp_dir().unwrap(), PathBuf::from("/test/temp"));
}

#[test]
fn test_mock_secure_storage() {
    // Create a mock platform provider with secure storage
    let mock = MockPlatformProvider::new()
        .with_secure_storage(true);
    
    // Test secure storage operations
    let key = "test_key";
    let value = b"test_value";
    
    // Store an item
    assert!(mock.store_secure_item(key, value).is_ok());
    
    // Retrieve the item
    let retrieve_result = mock.retrieve_secure_item(key);
    println!("Retrieve result: {:?}", retrieve_result);
    assert!(retrieve_result.is_ok());
    
    // Check the retrieved value
    let retrieved = retrieve_result.unwrap().unwrap();
    assert_eq!(retrieved, value);
    
    // Delete the item
    assert!(mock.delete_secure_item(key).is_ok());
    
    // Item should now be gone
    let after_delete = mock.retrieve_secure_item(key).unwrap();
    assert!(after_delete.is_none());
}

#[test]
fn test_mock_biometric_auth() {
    // Test with biometric auth available and success
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(true)
        .with_biometric_auth_result(Ok(true));
    
    assert!(mock.biometric_auth_available());
    assert!(mock.authenticate_with_biometrics("Test authentication").unwrap());
    
    // Test with biometric auth available but failure
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(true)
        .with_biometric_auth_result(Ok(false));
    
    assert!(mock.biometric_auth_available());
    assert!(!mock.authenticate_with_biometrics("Test authentication").unwrap());
    
    // Test with biometric auth not available
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(false);
    
    assert!(!mock.biometric_auth_available());
    assert!(mock.authenticate_with_biometrics("Test authentication").is_err());
} 