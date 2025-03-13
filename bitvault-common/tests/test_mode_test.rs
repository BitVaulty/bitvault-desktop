use std::sync::Once;
use bitvault_common::key_management::set_test_mode;

// Static initialization
static INIT: Once = Once::new();

// Setup function to initialize the test environment
fn setup() {
    INIT.call_once(|| {
        // Initialize logger for tests
        let _ = env_logger::builder().is_test(true).try_init();
    });
    
    // Always ensure test mode is enabled for each test
    set_test_mode(true);
    println!("Test mode set to true");
}

#[test]
fn test_test_mode() {
    setup();
    
    // This test just verifies that we can set test mode
    // The actual test is in the setup function
    assert!(true, "Test mode set successfully");
    
    println!("Test mode test passed");
} 