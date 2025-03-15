// New test for platform module
//
// This test verifies the platform module functionality.

use bitvault_common::platform;
use bitvault_common::platform::PlatformType;

#[test]
fn test_platform_functions() {
    // Test platform type
    let platform_type = platform::get_platform_type();
    println!("Platform type: {}", platform_type);
    
    // Test platform capabilities
    let capabilities = platform::get_platform_capabilities();
    println!("Platform capabilities: {:?}", capabilities);
    
    // Test directories
    let data_dir = platform::get_data_dir().expect("Failed to get data directory");
    println!("Data directory: {}", data_dir.display());
    
    let config_dir = platform::get_config_dir().expect("Failed to get config directory");
    println!("Config directory: {}", config_dir.display());
    
    let logs_dir = platform::get_logs_dir().expect("Failed to get logs directory");
    println!("Logs directory: {}", logs_dir.display());
    
    let temp_dir = platform::get_temp_dir().expect("Failed to get temp directory");
    println!("Temp directory: {}", temp_dir.display());
    
    // Test memory functions
    let buffer_size = 1024;
    let mut buffer = platform::secure_alloc(buffer_size);
    println!("Allocated secure buffer of size {}", buffer.len());
    
    // Fill with test data
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    
    // Test secure erase
    platform::secure_erase(&mut buffer);
    println!("Buffer erased");
    
    // Test memory locking if available
    if platform::has_secure_memory() {
        println!("Secure memory available");
        let lock_result = platform::lock_memory(buffer.as_ptr(), buffer.len());
        println!("Memory lock result: {:?}", lock_result);
        
        if lock_result.is_ok() {
            let unlock_result = platform::unlock_memory(buffer.as_ptr(), buffer.len());
            println!("Memory unlock result: {:?}", unlock_result);
        }
    } else {
        println!("Secure memory not available");
    }
    
    // Test check_dir_writable
    let writable_result = platform::check_dir_writable(&temp_dir);
    println!("Temp directory writable: {:?}", writable_result);
} 