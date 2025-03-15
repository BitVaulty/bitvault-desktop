// Test for platform functionality
//
// This test verifies basic platform functionality.

use std::path::PathBuf;
use bitvault_common::platform;
use bitvault_common::platform::PlatformType;

#[test]
fn test_platform_type() {
    // Test that we can detect the platform type
    let platform_type = platform::get_platform_type();
    
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
    assert!(!platform_name.is_empty());
}

#[test]
fn test_platform_capabilities() {
    // Test that we can get the platform capabilities
    let capabilities = platform::get_platform_capabilities();
    
    // This should match the platform type
    assert_eq!(capabilities.platform_type, platform::get_platform_type());
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
    
    // Directories should exist
    assert!(data_dir.unwrap().exists());
    assert!(config_dir.unwrap().exists());
    assert!(logs_dir.unwrap().exists());
    assert!(temp_dir.unwrap().exists());
} 