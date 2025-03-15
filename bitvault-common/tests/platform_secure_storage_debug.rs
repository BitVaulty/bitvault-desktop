// Debug test for secure storage functionality
// This is a simplified version of the secure storage test to diagnose issues

use bitvault_common::platform::mock::MockPlatformProvider;
use bitvault_common::platform::{set_platform_provider, platform, reset_platform_provider};

#[test]
fn test_mock_secure_storage_debug() {
    println!("Starting test_mock_secure_storage_debug");
    
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
    
    // If we got here, storage should have succeeded
    if store_result.is_ok() {
        println!("Store succeeded");
    } else {
        println!("Store failed: {:?}", store_result);
        return;
    }
    
    // Retrieve the item
    let retrieve_result = platform().retrieve_secure_item(key);
    println!("Retrieve result: {:?}", retrieve_result);
    
    // Check if retrieve succeeded
    if retrieve_result.is_ok() {
        println!("Retrieve succeeded");
        
        match retrieve_result {
            Ok(Some(retrieved)) => {
                println!("Retrieved value: {:?}", retrieved);
                println!("Expected value: {:?}", value);
                println!("Values match: {}", retrieved == value);
            },
            Ok(None) => {
                println!("Retrieved None, expected Some with value");
            },
            Err(e) => {
                println!("Error retrieving item: {}", e);
            }
        }
    } else {
        println!("Retrieve failed: {:?}", retrieve_result);
    }
    
    // Delete the item
    let delete_result = platform().delete_secure_item(key);
    println!("Delete result: {:?}", delete_result);
    
    // Clean up after the test
    reset_platform_provider();
    println!("Finished test_mock_secure_storage_debug");
} 