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
    let _ = address_book.add_entry_simple(
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        "Bitcoin Foundation",
        None,
        AddressCategory::Donation
    );
    
    let _ = address_book.add_entry_simple(
        "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S",
        "Bitcoin Core Donation",
        None,
        AddressCategory::Donation
    );
    
    // Add entries with Personal category
    let _ = address_book.add_entry_simple(
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
        "Personal Wallet",
        None,
        AddressCategory::Personal
    );
    
    // Find by category
    let donation_entries = address_book.find_by_category(&AddressCategory::Donation);
    assert_eq!(donation_entries.len(), 2);
    
    let personal_entries = address_book.find_by_category(&AddressCategory::Personal);
    assert_eq!(personal_entries.len(), 1);
    
    let business_entries = address_book.find_by_category(&AddressCategory::Business);
    assert_eq!(business_entries.len(), 0);
}

#[test]
fn test_find_by_label() {
    setup_category_test();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add multiple entries
    let _ = address_book.add_entry_simple(
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        "Bitcoin Foundation",
        None,
        AddressCategory::Donation
    );
    
    let _ = address_book.add_entry_simple(
        "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S",
        "Bitcoin Core Donation",
        None,
        AddressCategory::Donation
    );
    
    // Find by label (partial, case-insensitive)
    let results = address_book.find_by_label("donation");
    assert_eq!(results.len(), 1); // Only one entry contains "donation" in its label
    
    // Find by another label part
    let results = address_book.find_by_label("foundation");
    assert_eq!(results.len(), 1);
    
    // Find by non-existent label
    let results = address_book.find_by_label("nonexistent");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_custom_category() {
    setup_category_test();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add an entry with custom category
    let _ = address_book.add_entry_simple(
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        "My Custom Category Entry",
        None,
        AddressCategory::Custom("MyCategory".to_string())
    );
    
    // Find by custom category
    let results = address_book.find_by_category(&AddressCategory::Custom("MyCategory".to_string()));
    assert_eq!(results.len(), 1);
    
    // Different custom category should not match
    let results = address_book.find_by_category(&AddressCategory::Custom("OtherCategory".to_string()));
    assert_eq!(results.len(), 0);
} 