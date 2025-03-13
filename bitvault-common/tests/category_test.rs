use bitvault_common::address_book::{AddressBook, AddressCategory};
use bitcoin::Network;
use std::sync::Once;
use bitvault_common::logging::{self, LogConfig, LogLevel};

// Initialize once for category tests
static CATEGORY_TEST_INIT: Once = Once::new();

fn setup_category_test() {
    // Initialize global test logging without using external utils
    static GLOBAL_TEST_INIT: Once = Once::new();
    
    GLOBAL_TEST_INIT.call_once(|| {
        // Configure minimal logging for tests
        let config = LogConfig {
            level: LogLevel::Error, // Use Error level to minimize output
            log_file: None,         // No file logging in tests
            include_timestamps: false,
            include_source_location: false,
            max_file_size: 1024 * 1024,
            console_logging: false, // Disable console logging for tests
            json_format: false,
        };

        // Initialize logging with test configuration
        // Ignore any errors - tests should work even if logging fails
        let _ = logging::init(&config);
    });
    
    CATEGORY_TEST_INIT.call_once(|| {
        // Additional test-specific initialization if needed
    });
}

#[test]
fn test_find_by_category() {
    setup_category_test();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add entries with Donation category
    let _ = address_book.add_entry(
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        "Bitcoin Foundation",
        None,
        AddressCategory::Donation
    );
    
    let _ = address_book.add_entry(
        "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S",
        "Bitcoin Core Donation",
        None,
        AddressCategory::Donation
    );
    
    // Find by Donation category
    let results = address_book.find_by_category(&AddressCategory::Donation);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_find_by_label() {
    setup_category_test();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add multiple entries
    let _ = address_book.add_entry(
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        "Bitcoin Foundation",
        None,
        AddressCategory::Donation
    );
    
    let _ = address_book.add_entry(
        "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S",
        "Bitcoin Core Donation",
        None,
        AddressCategory::Donation
    );
    
    // Find by label (partial, case-insensitive)
    let results = address_book.find_by_label("donation");
    assert_eq!(results.len(), 1); // Only one entry contains "donation" in its label
}

#[test]
fn test_custom_category() {
    setup_category_test();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Use the exact same string instance for the custom category
    let custom_category = "My Category".to_string();
    let _ = address_book.add_entry(
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        "Custom Category",
        None,
        AddressCategory::Custom(custom_category.clone())
    );
    
    // Find by custom category - use the same string instance
    let results = address_book.find_by_category(&AddressCategory::Custom(custom_category));
    assert_eq!(results.len(), 1);
    
    // Find by custom category with a different string instance but same content
    let results = address_book.find_by_category(&AddressCategory::Custom("My Category".to_string()));
    assert_eq!(results.len(), 1);
} 