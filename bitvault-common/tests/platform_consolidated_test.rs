// Consolidated platform tests
//
// This test file combines functionality from:
// - platform_minimal_test.rs
// - platform_minimal_debug.rs
// - platform_simple_test.rs
//
// It provides basic verification of platform functionality with file-based output
// utilities for better diagnostics.

use std::sync::Once;
use std::io::Write;
use bitvault_common::platform;
use bitvault_common::PlatformType;

// Import test helpers
mod test_helpers;
use test_helpers::{
    log_test_start, log_test_end, log_info, log_error,
    write_test_output, create_test_logger
};

// Static initialization
static INIT: Once = Once::new();

// Setup function to initialize the test environment
fn setup() {
    INIT.call_once(|| {
        // Initialize logger for tests
        let _ = env_logger::builder().is_test(true).try_init();
        log_info("Platform tests initialized");
    });
}

#[test]
fn test_platform_type_basic() {
    setup();
    log_test_start("platform_type_basic");
    let mut logger = create_test_logger("platform_type");
    
    // Get platform type
    let platform_type = platform::get_platform_type();
    writeln!(logger, "Platform type: {:?}", platform_type).unwrap();
    
    // Convert to string for display
    let platform_string = match platform_type {
        PlatformType::Linux => "Linux".to_string(),
        PlatformType::MacOS => "MacOS".to_string(),
        PlatformType::Windows => "Windows".to_string(),
        PlatformType::Android => "Android".to_string(),
        PlatformType::IOS => "iOS".to_string(),
        PlatformType::Other => "Other".to_string(),
    };
    writeln!(logger, "Platform string: {}", platform_string).unwrap();
    
    // Verify that platform type formatting works correctly
    let formatted = format!("{:?}", platform_type);
    writeln!(logger, "Formatted platform type: {}", formatted).unwrap();
    
    // Simple verification that platform type exists and is implemented
    assert!(match platform_type {
        PlatformType::Linux | PlatformType::MacOS | PlatformType::Windows | 
        PlatformType::Android | PlatformType::IOS | PlatformType::Other => true,
    }, "Platform type should be one of the known types");
    
    writeln!(logger, "Platform type verification passed").unwrap();
    log_test_end("platform_type_basic", true);
}

#[test]
fn test_platform_capabilities_basic() {
    setup();
    log_test_start("platform_capabilities_basic");
    
    // Create a vector of strings for output lines
    let mut output_lines = Vec::new();
    output_lines.push("Platform Capabilities Test Results".to_string());
    output_lines.push("=================================".to_string());
    
    // Get platform capabilities
    let capabilities = platform::get_platform_capabilities();
    output_lines.push(format!("Platform capabilities: {:?}", capabilities));
    
    // Check secure enclave support
    output_lines.push(format!("Secure enclave supported: {}", capabilities.has_secure_enclave));
    
    // Check memory locking support
    output_lines.push(format!("Memory locking supported: {}", capabilities.supports_memory_locking));
    
    // Check secure storage
    output_lines.push(format!("Secure storage available: {}", capabilities.has_secure_storage));
    
    // Check biometric authentication
    output_lines.push(format!("Biometric authentication: {}", capabilities.has_biometric_auth));
    
    // Verify some basic expectations (adjust based on your platform)
    // Most desktop platforms should support memory locking
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        assert!(capabilities.supports_memory_locking, "Memory locking should be supported on desktop platforms");
        output_lines.push("✓ Verified memory locking capability".to_string());
    }
    
    // Write the test output to a file
    let output = output_lines.join("\n");
    if let Err(e) = write_test_output("platform_capabilities", &output) {
        log_error(&format!("Failed to write test output: {}", e));
    }
    
    log_test_end("platform_capabilities_basic", true);
}

#[test]
fn test_platform_directories_basic() {
    setup();
    log_test_start("platform_directories_basic");
    
    // Create a vector of strings for output lines
    let mut output_lines = Vec::new();
    output_lines.push("Platform Directories Test Results".to_string());
    output_lines.push("================================".to_string());
    
    // Get and check data directory
    match platform::get_data_dir() {
        Ok(dir) => {
            let dir_str = dir.display().to_string();
            output_lines.push(format!("Data directory: {}", dir_str));
            assert!(!dir_str.is_empty(), "Data directory should not be empty");
        },
        Err(e) => {
            output_lines.push(format!("Failed to get data directory: {}", e));
            panic!("Failed to get data directory: {}", e);
        }
    }
    
    // Get and check config directory
    match platform::get_config_dir() {
        Ok(dir) => {
            let dir_str = dir.display().to_string();
            output_lines.push(format!("Config directory: {}", dir_str));
            assert!(!dir_str.is_empty(), "Config directory should not be empty");
        },
        Err(e) => {
            output_lines.push(format!("Failed to get config directory: {}", e));
            panic!("Failed to get config directory: {}", e);
        }
    }
    
    // Get and check logs directory
    match platform::get_logs_dir() {
        Ok(dir) => {
            let dir_str = dir.display().to_string();
            output_lines.push(format!("Logs directory: {}", dir_str));
            assert!(!dir_str.is_empty(), "Logs directory should not be empty");
        },
        Err(e) => {
            output_lines.push(format!("Failed to get logs directory: {}", e));
            panic!("Failed to get logs directory: {}", e);
        }
    }
    
    // Get and check temp directory
    match platform::get_temp_dir() {
        Ok(dir) => {
            let dir_str = dir.display().to_string();
            output_lines.push(format!("Temp directory: {}", dir_str));
            assert!(!dir_str.is_empty(), "Temp directory should not be empty");
        },
        Err(e) => {
            output_lines.push(format!("Failed to get temp directory: {}", e));
            panic!("Failed to get temp directory: {}", e);
        }
    }
    
    // Verify directories are different
    let data_dir = platform::get_data_dir().unwrap().display().to_string();
    let logs_dir = platform::get_logs_dir().unwrap().display().to_string();
    let config_dir = platform::get_config_dir().unwrap().display().to_string();
    let temp_dir = platform::get_temp_dir().unwrap().display().to_string();
    
    assert_ne!(data_dir, logs_dir, "Data and logs directories should be different");
    assert_ne!(config_dir, temp_dir, "Config and temp directories should be different");
    output_lines.push("✓ Verified that directories are distinct".to_string());
    
    // Write the test output to a file
    let output = output_lines.join("\n");
    if let Err(e) = write_test_output("platform_directories", &output) {
        log_error(&format!("Failed to write test output: {}", e));
    }
    
    log_test_end("platform_directories_basic", true);
}

#[test]
fn test_memory_operations_basic() {
    setup();
    log_test_start("memory_operations_basic");
    
    // Create a vector of strings for output lines
    let mut output_lines = Vec::new();
    output_lines.push("Memory Operations Test Results".to_string());
    output_lines.push("============================".to_string());
    
    // Check if secure memory is available
    let has_secure_memory = platform::has_secure_memory();
    output_lines.push(format!("Secure memory available: {}", has_secure_memory));
    
    // Test memory operations with a secure buffer
    let size = 32;
    let mut secure_buffer = platform::secure_alloc(size);
    output_lines.push(format!("Allocated secure buffer of size {}", secure_buffer.len()));
    
    // Fill with test data
    for (i, byte) in secure_buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    output_lines.push(format!("Filled buffer with test data: {:?}", &secure_buffer[..8]));
    
    // Test memory locking if supported
    if has_secure_memory {
        match platform::lock_memory(secure_buffer.as_ptr(), secure_buffer.len()) {
            Ok(_) => {
                output_lines.push("✓ Memory locked successfully".to_string());
                
                // Try to unlock
                match platform::unlock_memory(secure_buffer.as_ptr(), secure_buffer.len()) {
                    Ok(_) => output_lines.push("✓ Memory unlocked successfully".to_string()),
                    Err(e) => output_lines.push(format!("✗ Failed to unlock memory: {}", e)),
                }
            },
            Err(e) => {
                output_lines.push(format!("✗ Failed to lock memory: {}", e));
                output_lines.push("Note: Memory locking might require elevated privileges".to_string());
            }
        }
    } else {
        output_lines.push("Secure memory operations not available on this platform".to_string());
    }
    
    // Test secure erase
    platform::secure_erase(&mut secure_buffer);
    output_lines.push(format!("Buffer after secure erase: {:?}", &secure_buffer[..8]));
    
    // Verify all bytes are zero
    let all_zeros = secure_buffer.iter().all(|&x| x == 0);
    assert!(all_zeros, "All bytes should be zero after secure_erase");
    output_lines.push(if all_zeros {
        "✓ Verified all bytes are zero after secure_erase".to_string()
    } else {
        "✗ secure_erase did not zero all bytes".to_string()
    });
    
    // Write the test output to a file
    let output = output_lines.join("\n");
    if let Err(e) = write_test_output("platform_memory_operations", &output) {
        log_error(&format!("Failed to write test output: {}", e));
    }
    
    log_test_end("memory_operations_basic", true);
} 