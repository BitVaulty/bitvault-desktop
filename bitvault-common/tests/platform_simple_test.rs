// Simple test for platform module
//
// This test verifies basic functionality of the platform module.

use bitvault_common::platform;

#[test]
fn test_platform_type() {
    // Test that we can get the platform type
    let platform_type = platform::get_platform_type();
    println!("Platform type: {}", platform_type);
    
    // Test that we can get the platform capabilities
    let capabilities = match platform::get_platform_capabilities() {
        cap => {
            println!("Platform capabilities: has_secure_enclave={}, supports_memory_locking={}, has_secure_storage={}, has_biometric_auth={}", 
                     cap.has_secure_enclave, 
                     cap.supports_memory_locking, 
                     cap.has_secure_storage, 
                     cap.has_biometric_auth);
            cap
        }
    };
    
    assert_eq!(capabilities.platform_type, platform_type);
}

#[test]
fn test_platform_directories() {
    // Test that we can get the data directory
    match platform::get_data_dir() {
        Ok(dir) => println!("Data directory: {}", dir.display()),
        Err(e) => println!("Data directory error: {}", e)
    }
    
    // Test that we can get the config directory
    match platform::get_config_dir() {
        Ok(dir) => println!("Config directory: {}", dir.display()),
        Err(e) => println!("Config directory error: {}", e)
    }
    
    // Test that we can get the logs directory
    match platform::get_logs_dir() {
        Ok(dir) => println!("Logs directory: {}", dir.display()),
        Err(e) => println!("Logs directory error: {}", e)
    }
    
    // Test that we can get the temp directory
    match platform::get_temp_dir() {
        Ok(dir) => println!("Temp directory: {}", dir.display()),
        Err(e) => println!("Temp directory error: {}", e)
    }
}

#[test]
fn test_memory_operations() {
    // Test secure allocation
    let size = 1024;
    let mut buffer = platform::secure_alloc(size);
    println!("Allocated secure buffer of size {}", buffer.len());
    assert_eq!(buffer.len(), size);
    
    // Fill with test data
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    
    // Test secure erase
    platform::secure_erase(&mut buffer);
    println!("Buffer erased, checking zeros");
    assert!(buffer.iter().all(|&b| b == 0));
    
    // Test memory locking if available
    if platform::has_secure_memory() {
        println!("Secure memory available, testing locking");
        match platform::lock_memory(buffer.as_ptr(), buffer.len()) {
            Ok(_) => {
                println!("Memory locked successfully");
                // Try to unlock
                match platform::unlock_memory(buffer.as_ptr(), buffer.len()) {
                    Ok(_) => println!("Memory unlocked successfully"),
                    Err(e) => println!("Memory unlock error: {}", e)
                }
            },
            Err(e) => println!("Memory lock error: {}", e)
        }
    } else {
        println!("Secure memory not available on this platform");
    }
} 