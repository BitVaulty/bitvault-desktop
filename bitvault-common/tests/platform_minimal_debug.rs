// Minimal debug test for the platform module

use bitvault_common::platform;

#[test]
fn test_get_platform_type() {
    let platform_type = platform::get_platform_type();
    println!("Platform type: {:?}", platform_type);
    assert!(true);
} 