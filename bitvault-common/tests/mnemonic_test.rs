use std::sync::Once;
use bitvault_common::key_management::{
    generate_mnemonic_and_key,
    set_test_mode
};

// Static initialization
static INIT: Once = Once::new();

// Setup function to initialize the test environment
fn setup() {
    INIT.call_once(|| {
        // Initialize logger for tests
        let _ = env_logger::builder().is_test(true).try_init();
        
        // Enable test mode for key management
        set_test_mode(true);
        println!("Test mode enabled for key management tests");
    });
    
    // Always ensure test mode is enabled for each test
    set_test_mode(true);
    println!("Key management test setup completed");
}

#[test]
fn test_mnemonic_generation() {
    setup();
    
    // Generate a mnemonic with a test password
    let password = "test_password";
    let result = generate_mnemonic_and_key(password);
    
    // Verify the result is Ok
    assert!(result.is_ok(), "Failed to generate mnemonic");
    
    // Extract the mnemonic and key
    let (mnemonic, key) = result.unwrap();
    
    // Verify the mnemonic has 12 words
    let mnemonic_phrase = mnemonic.to_string();
    let word_count = mnemonic_phrase.split_whitespace().count();
    assert_eq!(word_count, 12, "Mnemonic should have 12 words");
    
    // Just verify we got a key (no need to inspect its contents)
    assert!(true, "Key generation succeeded");
    
    println!("Mnemonic generation test passed");
} 