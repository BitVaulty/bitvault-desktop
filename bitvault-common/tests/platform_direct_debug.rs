// Direct tests for MockPlatformProvider with better error reporting
//
// This test file directly tests the MockPlatformProvider methods
// with better error reporting to help debug test failures.

use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;

use bitvault_common::platform::PlatformType;
use bitvault_common::platform::provider::PlatformProvider;
use bitvault_common::platform::mock::MockPlatformProvider;

fn log_to_file(message: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/platform_test_log.txt")
        .unwrap();
    
    writeln!(file, "{}", message).unwrap();
}

#[test]
fn test_mock_secure_storage_direct() {
    log_to_file("\n\n--- Starting test_mock_secure_storage_direct ---");
    
    // Create MockPlatformProvider with secure storage
    let mock = MockPlatformProvider::new()
        .with_secure_storage(true);
    
    log_to_file("Created mock with secure storage");
    
    // Test key and value
    let key = "test_key";
    let value = b"test_value";
    
    // Step 1: Store the item
    log_to_file(&format!("Storing key '{}' with value {:?}", key, value));
    match mock.store_secure_item(key, value) {
        Ok(_) => log_to_file("Store succeeded"),
        Err(e) => {
            let err_msg = format!("Store failed: {}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    // Step 2: Retrieve the item
    log_to_file(&format!("Retrieving key '{}'", key));
    match mock.retrieve_secure_item(key) {
        Ok(Some(retrieved)) => {
            log_to_file(&format!("Retrieved value: {:?}", retrieved));
            assert_eq!(retrieved, value, "Retrieved value doesn't match stored value");
            log_to_file("Retrieve succeeded with correct value");
        },
        Ok(None) => {
            let err_msg = "Retrieve returned None when Some was expected";
            log_to_file(err_msg);
            panic!("{}", err_msg);
        },
        Err(e) => {
            let err_msg = format!("Retrieve failed: {}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    // Step 3: Delete the item
    log_to_file(&format!("Deleting key '{}'", key));
    match mock.delete_secure_item(key) {
        Ok(_) => log_to_file("Delete succeeded"),
        Err(e) => {
            let err_msg = format!("Delete failed: {}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    // Step 4: Verify the item is gone
    log_to_file(&format!("Verifying key '{}' is gone", key));
    match mock.retrieve_secure_item(key) {
        Ok(None) => log_to_file("Item is gone as expected"),
        Ok(Some(_)) => {
            let err_msg = "Item still exists after deletion";
            log_to_file(err_msg);
            panic!("{}", err_msg);
        },
        Err(e) => {
            let err_msg = format!("Retrieve after delete failed: {}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    log_to_file("--- Finished test_mock_secure_storage_direct ---");
}

#[test]
fn test_mock_provider_directories() {
    log_to_file("\n\n--- Starting test_mock_provider_directories ---");
    
    // Test paths
    let data_dir = PathBuf::from("/test/data");
    let config_dir = PathBuf::from("/test/config");
    let logs_dir = PathBuf::from("/test/logs");
    let temp_dir = PathBuf::from("/test/temp");
    
    // Create MockPlatformProvider with configured directories
    let mock = MockPlatformProvider::new()
        .with_platform_type(PlatformType::Linux)
        .with_data_dir(data_dir.clone())
        .with_config_dir(config_dir.clone())
        .with_logs_dir(logs_dir.clone())
        .with_temp_dir(temp_dir.clone());
    
    log_to_file("Created mock with configured directories");
    
    // Test get_data_dir
    log_to_file("Testing get_data_dir");
    match mock.get_data_dir() {
        Ok(dir) => {
            log_to_file(&format!("get_data_dir returned: {:?}", dir));
            assert_eq!(dir, data_dir, "get_data_dir returned wrong path");
        },
        Err(e) => {
            let err_msg = format!("get_data_dir failed: {:?}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    // Test get_config_dir
    log_to_file("Testing get_config_dir");
    match mock.get_config_dir() {
        Ok(dir) => {
            log_to_file(&format!("get_config_dir returned: {:?}", dir));
            assert_eq!(dir, config_dir, "get_config_dir returned wrong path");
        },
        Err(e) => {
            let err_msg = format!("get_config_dir failed: {:?}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    // Test get_logs_dir
    log_to_file("Testing get_logs_dir");
    match mock.get_logs_dir() {
        Ok(dir) => {
            log_to_file(&format!("get_logs_dir returned: {:?}", dir));
            assert_eq!(dir, logs_dir, "get_logs_dir returned wrong path");
        },
        Err(e) => {
            let err_msg = format!("get_logs_dir failed: {:?}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    // Test get_temp_dir
    log_to_file("Testing get_temp_dir");
    match mock.get_temp_dir() {
        Ok(dir) => {
            log_to_file(&format!("get_temp_dir returned: {:?}", dir));
            assert_eq!(dir, temp_dir, "get_temp_dir returned wrong path");
        },
        Err(e) => {
            let err_msg = format!("get_temp_dir failed: {:?}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    log_to_file("--- Finished test_mock_provider_directories ---");
}

#[test]
fn test_mock_biometric_auth_direct() {
    log_to_file("\n\n--- Starting test_mock_biometric_auth_direct ---");
    
    // Test with biometric auth available and success
    log_to_file("Test case 1: Biometric auth available and success");
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(true)
        .with_biometric_auth_result(Ok(true));
    
    log_to_file("Created mock with biometric auth and success result");
    
    // Test biometric_auth_available
    let available = mock.biometric_auth_available();
    log_to_file(&format!("biometric_auth_available returned: {}", available));
    assert!(available, "biometric_auth_available should return true");
    
    // Test authenticate_with_biometrics
    match mock.authenticate_with_biometrics("Test authentication") {
        Ok(true) => log_to_file("Authentication succeeded as expected"),
        Ok(false) => {
            let err_msg = "Authentication returned false when true was expected";
            log_to_file(err_msg);
            panic!("{}", err_msg);
        },
        Err(e) => {
            let err_msg = format!("Authentication failed: {}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    // Test with biometric auth available but failure
    log_to_file("\nTest case 2: Biometric auth available but failure");
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(true)
        .with_biometric_auth_result(Ok(false));
    
    log_to_file("Created mock with biometric auth and failure result");
    
    // Test biometric_auth_available
    let available = mock.biometric_auth_available();
    log_to_file(&format!("biometric_auth_available returned: {}", available));
    assert!(available, "biometric_auth_available should return true");
    
    // Test authenticate_with_biometrics
    match mock.authenticate_with_biometrics("Test authentication") {
        Ok(false) => log_to_file("Authentication failed as expected"),
        Ok(true) => {
            let err_msg = "Authentication returned true when false was expected";
            log_to_file(err_msg);
            panic!("{}", err_msg);
        },
        Err(e) => {
            let err_msg = format!("Authentication failed with error: {}", e);
            log_to_file(&err_msg);
            panic!("{}", err_msg);
        }
    }
    
    // Test with biometric auth not available
    log_to_file("\nTest case 3: Biometric auth not available");
    let mock = MockPlatformProvider::new()
        .with_biometric_auth(false);
    
    log_to_file("Created mock without biometric auth");
    
    // Test biometric_auth_available
    let available = mock.biometric_auth_available();
    log_to_file(&format!("biometric_auth_available returned: {}", available));
    assert!(!available, "biometric_auth_available should return false");
    
    // Test authenticate_with_biometrics
    match mock.authenticate_with_biometrics("Test authentication") {
        Err(e) => log_to_file(&format!("Authentication failed as expected with: {}", e)),
        Ok(_) => {
            let err_msg = "Authentication succeeded when it should have failed";
            log_to_file(err_msg);
            panic!("{}", err_msg);
        }
    }
    
    log_to_file("--- Finished test_mock_biometric_auth_direct ---");
} 