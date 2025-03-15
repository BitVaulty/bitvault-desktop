// A simple test for MockPlatformProvider to diagnose issues

use bitvault_common::platform::mock::MockPlatformProvider;
use bitvault_common::platform::provider::PlatformProvider;

#[test]
fn test_mock_platform_basic() {
    println!("Creating MockPlatformProvider instance...");
    let mock = MockPlatformProvider::new();
    println!("MockPlatformProvider created successfully");
    
    // Get platform type
    let platform_type = mock.get_platform_type();
    println!("Platform type: {:?}", platform_type);
    
    // Get platform capabilities
    let capabilities = mock.get_capabilities();
    println!("Platform capabilities: {:?}", capabilities);
    
    // Test secure storage if enabled
    if capabilities.has_secure_storage {
        println!("Testing secure storage...");
        
        // Store an item
        let key = "test_key";
        let value = b"test_value";
        println!("Storing key '{}' with value {:?}", key, value);
        let store_result = mock.store_secure_item(key, value);
        println!("Store result: {:?}", store_result);
        
        // Retrieve the item
        println!("Retrieving key '{}'", key);
        let retrieve_result = mock.retrieve_secure_item(key);
        println!("Retrieve result: {:?}", retrieve_result);
        
        // Delete the item
        println!("Deleting key '{}'", key);
        let delete_result = mock.delete_secure_item(key);
        println!("Delete result: {:?}", delete_result);
        
        // Verify the item is gone
        println!("Verifying key '{}' is gone", key);
        let retrieve_after_delete = mock.retrieve_secure_item(key);
        println!("Retrieve after delete: {:?}", retrieve_after_delete);
    }
    
    println!("Test completed successfully");
} 