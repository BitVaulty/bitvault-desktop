// Test for importing the platform module and MockPlatformProvider

#[test]
fn test_can_import_platform() {
    // Just check that we can import the platform module
    use bitvault_common::platform;
    
    let platform_type = platform::get_platform_type();
    println!("Platform type: {}", platform_type);
    
    assert!(true);
}

#[test]
fn test_can_import_mock() {
    // Just check that we can import the mock module
    use bitvault_common::platform::mock::MockPlatformProvider;
    
    let mock = MockPlatformProvider::new();
    println!("Mock provider created: {:?}", mock);
    
    assert!(true);
}

#[test]
fn test_can_import_set_platform_provider() {
    // Just check that we can import the set_platform_provider function
    use bitvault_common::platform::set_platform_provider;
    use bitvault_common::platform::mock::MockPlatformProvider;
    
    // Just reference the function to verify it can be imported
    println!("Can import set_platform_provider: {}", 
             std::any::type_name_of_val(&set_platform_provider));
    
    assert!(true);
} 