use bitvault_common::address_book::{AddressBook, AddressCategory};
use bitcoin::Network;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use std::time::Duration;
use std::sync::Once;
use bitvault_common::logging::{self, LogConfig, LogLevel};

// Initialize once for address book tests
static ADDRESS_BOOK_TESTS_INIT: Once = Once::new();

fn setup_address_book_tests() {
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
    
    ADDRESS_BOOK_TESTS_INIT.call_once(|| {
        // Additional test-specific initialization if needed
    });
}

// Test addresses for various networks
const MAINNET_ADDRESS: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
const TESTNET_ADDRESS: &str = "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn";
const REGTEST_ADDRESS: &str = "bcrt1qnght4cs07uh2z3v9pr0xly7n7jnx2kl2p0q38w";
const INVALID_ADDRESS: &str = "not-a-bitcoin-address";

#[test]
fn test_address_book_creation() {
    setup_address_book_tests();
    
    // Create a new address book
    let address_book = AddressBook::new(Network::Bitcoin);
    
    // Check initial state
    assert_eq!(address_book.network(), Network::Bitcoin);
    assert!(address_book.is_empty());
    assert_eq!(address_book.len(), 0);
    assert!(address_book.get_all_entries().is_empty());
}

#[test]
fn test_add_and_get_entry() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add an entry - using add_entry_simple instead of add_entry
    let result = address_book.add_entry_simple(MAINNET_ADDRESS, "Satoshi", Some("First Bitcoin address"), AddressCategory::Personal);
    
    // Check success
    assert!(result.is_ok());
    assert_eq!(address_book.len(), 1);
    
    // Get the entry
    let entry = address_book.get_entry(MAINNET_ADDRESS);
    assert!(entry.is_some());
    
    let entry = entry.unwrap();
    assert_eq!(entry.address, MAINNET_ADDRESS);
    assert_eq!(entry.label, "Satoshi");
    assert_eq!(entry.notes, Some("First Bitcoin address".to_string()));
    assert_eq!(entry.category, AddressCategory::Personal);
    assert!(entry.created_at > 0);
    assert_eq!(entry.last_used, None);
    
    // Parse the address
    let address = entry.parse_address(Network::Bitcoin);
    assert!(address.is_ok());
}

#[test]
fn test_add_invalid_address() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Try adding an invalid address
    let result = address_book.add_entry_simple(
        INVALID_ADDRESS,
        "Invalid",
        None,
        AddressCategory::Personal
    );
    
    // Should fail
    assert!(result.is_err());
    assert_eq!(address_book.len(), 0);
    
    // Try adding a testnet address to a mainnet book
    let result = address_book.add_entry_simple(
        TESTNET_ADDRESS,
        "Testnet",
        None,
        AddressCategory::Personal
    );
    
    // Should fail
    assert!(result.is_err());
    assert_eq!(address_book.len(), 0);
}

#[test]
fn test_remove_entry() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add an entry
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
        "Satoshi",
        None,
        AddressCategory::Personal
    );
    
    assert_eq!(address_book.len(), 1);
    
    // Remove the entry
    let removed = address_book.remove_entry(MAINNET_ADDRESS);
    assert!(removed);
    assert_eq!(address_book.len(), 0);
    
    // Try removing again
    let removed = address_book.remove_entry(MAINNET_ADDRESS);
    assert!(!removed); // Should return false for non-existent entry
}

#[test]
fn test_update_entry() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add an entry
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
        "Satoshi",
        None,
        AddressCategory::Personal
    );
    
    // Update the entry
    let update_result = address_book.update_entry(
        MAINNET_ADDRESS,
        Some("Updated Name"),
        Some("Added notes"),
        Some(AddressCategory::Business)
    );
    
    assert!(update_result.is_ok());
    
    // Verify the update
    let entry = address_book.get_entry(MAINNET_ADDRESS).unwrap();
    assert_eq!(entry.label, "Updated Name");
    assert_eq!(entry.notes, Some("Added notes".to_string()));
    assert_eq!(entry.category, AddressCategory::Business);
    
    // Try to update non-existent entry
    let invalid_result = address_book.update_entry(
        "not-a-valid-address",
        Some("Invalid"),
        Some("This is not a valid Bitcoin address"),
        None
    );
    
    assert!(invalid_result.is_err());
}

#[test]
fn test_mark_as_used() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add an entry
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
        "Satoshi",
        None,
        AddressCategory::Personal
    );
    
    // Verify initial state
    {
        let entry = address_book.get_entry(MAINNET_ADDRESS).unwrap();
        assert_eq!(entry.last_used, None);
    }
    
    // Record the current timestamp before marking as used
    let before_mark = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Mark as used
    let result = address_book.mark_as_used(MAINNET_ADDRESS);
    assert!(result.is_ok());
    
    // Get the timestamp after marking as used
    let first_timestamp_opt = {
        let updated_entry = address_book.get_entry(MAINNET_ADDRESS).unwrap();
        updated_entry.last_used
    };
    
    // Ensure we got a timestamp
    assert!(first_timestamp_opt.is_some(), "Expected a timestamp after marking as used");
    let first_timestamp = first_timestamp_opt.unwrap();
    
    // Verify the timestamp is at least as recent as our before_mark timestamp
    assert!(first_timestamp >= before_mark, 
        "Expected timestamp ({}) to be >= before_mark ({})", 
        first_timestamp, before_mark);
    
    // Let some time pass - use a longer sleep to ensure timestamp changes
    sleep(Duration::from_secs(1));
    
    // Mark as used again
    let _ = address_book.mark_as_used(MAINNET_ADDRESS);
    
    // Get the timestamp after marking as used again
    let second_timestamp_opt = {
        let entry_again = address_book.get_entry(MAINNET_ADDRESS).unwrap();
        entry_again.last_used
    };
    
    // Ensure we got a timestamp
    assert!(second_timestamp_opt.is_some(), "Expected a timestamp after marking as used again");
    let second_timestamp = second_timestamp_opt.unwrap();
    
    // Verify the second timestamp is greater than the first
    assert!(second_timestamp > first_timestamp, 
        "Expected second timestamp ({}) to be > first timestamp ({})", 
        second_timestamp, first_timestamp);
}

#[test]
fn test_find_by_label() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add multiple entries
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
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
    
    let _ = address_book.add_entry_simple(
        "38Segwittt9kLMmFUvn17osR58MzuoiJz9",
        "Personal Wallet",
        None,
        AddressCategory::Personal
    );
    
    // Find by label (full)
    let results = address_book.find_by_label("Bitcoin Foundation");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].address, MAINNET_ADDRESS);
    
    // Find by label (partial, case-insensitive)
    let results = address_book.find_by_label("donation");
    assert_eq!(results.len(), 1); // Only one entry contains "donation" in its label
    
    // Find by non-existent label
    let results = address_book.find_by_label("Exchange");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_find_by_category() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add multiple entries with different categories
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
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
    
    let _ = address_book.add_entry_simple(
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
        "Personal Wallet",
        None,
        AddressCategory::Personal
    );
    
    // Find entries by category
    let donation_entries = address_book.find_by_category(&AddressCategory::Donation);
    assert_eq!(donation_entries.len(), 2);
    
    let personal_entries = address_book.find_by_category(&AddressCategory::Personal);
    assert_eq!(personal_entries.len(), 1);
    
    let business_entries = address_book.find_by_category(&AddressCategory::Business);
    assert_eq!(business_entries.len(), 0);
}

#[test]
fn test_json_serialization() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add an entry
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
        "Satoshi",
        Some("First Bitcoin address"),
        AddressCategory::Personal
    );
    
    // Serialize to JSON
    let json = address_book.to_json();
    assert!(json.is_ok());
    
    let json_str = json.unwrap();
    
    // Deserialize back
    let deserialized = AddressBook::from_json(&json_str);
    assert!(deserialized.is_ok());
    
    let deserialized_book = deserialized.unwrap();
    assert_eq!(deserialized_book.network(), Network::Bitcoin);
    assert_eq!(deserialized_book.len(), 1);
    
    let entry = deserialized_book.get_entry(MAINNET_ADDRESS).unwrap();
    assert_eq!(entry.label, "Satoshi");
    assert_eq!(entry.notes, Some("First Bitcoin address".to_string()));
}

#[test]
fn test_import_entries() {
    setup_address_book_tests();
    
    // Create first address book
    let mut book1 = AddressBook::new(Network::Bitcoin);
    let _ = book1.add_entry_simple(
        MAINNET_ADDRESS,
        "First Book Entry",
        None,
        AddressCategory::Personal
    );
    
    // Create second address book
    let mut book2 = AddressBook::new(Network::Bitcoin);
    let _ = book2.add_entry_simple(
        "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S",
        "Second Book Entry",
        None,
        AddressCategory::Donation
    );
    
    // Common entry with different label
    let _ = book2.add_entry_simple(
        MAINNET_ADDRESS,
        "Different Label",
        None,
        AddressCategory::Business
    );
    
    // Import without overwrite
    let imported = book1.import_entries(&book2, false);
    assert_eq!(imported, 1); // Only the non-duplicate entry
    assert_eq!(book1.len(), 2);
    
    // Original entry should be preserved
    let entry = book1.get_entry(MAINNET_ADDRESS).unwrap();
    assert_eq!(entry.label, "First Book Entry");
    
    // Import with overwrite
    let imported = book1.import_entries(&book2, true);
    assert_eq!(imported, 2); // Both entries
    assert_eq!(book1.len(), 2);
    
    // Entry should be overwritten
    let entry = book1.get_entry(MAINNET_ADDRESS).unwrap();
    assert_eq!(entry.label, "Different Label");
    
    // Test import from different network
    let book3 = AddressBook::new(Network::Testnet);
    let imported = book1.import_entries(&book3, true);
    assert_eq!(imported, 0); // Nothing should be imported
}

#[test]
fn test_address_category_display() {
    setup_address_book_tests();
    
    // Test Display implementation for AddressCategory
    assert_eq!(format!("{}", AddressCategory::Personal), "Personal");
    assert_eq!(format!("{}", AddressCategory::Business), "Business");
    assert_eq!(format!("{}", AddressCategory::Donation), "Donation");
    assert_eq!(format!("{}", AddressCategory::Exchange), "Exchange");
    assert_eq!(
        format!("{}", AddressCategory::Custom("Test".to_string())), 
        "Test"
    );
}

#[test]
fn test_address_entry_creation_timestamp() {
    setup_address_book_tests();
    
    // Get current time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    // Create a new address book and entry
    let mut address_book = AddressBook::new(Network::Bitcoin);
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
        "Test Entry",
        None,
        AddressCategory::Personal
    );
    
    // Get the entry
    let entry = address_book.get_entry(MAINNET_ADDRESS).unwrap();
    
    // Check that created_at is close to current time
    assert!(entry.created_at >= now); // Should be at least the time we captured
    assert!(entry.created_at <= now + 2); // Allow for a small delay
}

#[test]
fn test_address_entry_last_used_update() {
    // Create a new address book and entry
    let mut address_book = AddressBook::new(Network::Bitcoin);
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
        "Test Entry",
        None,
        AddressCategory::Personal
    );
    
    // Get initial timestamp
    let before_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    // Sleep briefly to ensure time difference
    sleep(Duration::from_millis(100));
    
    // Mark as used
    let _ = address_book.mark_as_used(MAINNET_ADDRESS);
    
    // Get the entry
    let entry = address_book.get_entry(MAINNET_ADDRESS).unwrap();
    
    // Check that last_used was updated
    assert!(entry.last_used.is_some());
    let last_used = entry.last_used.unwrap();
    assert!(last_used >= before_time);
}

#[test]
fn test_add_multiple_entries() {
    setup_address_book_tests();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add several entries
    let _ = address_book.add_entry_simple(
        MAINNET_ADDRESS,
        "Satoshi 1",
        Some("First entry"),
        AddressCategory::Personal
    );
    
    let _ = address_book.add_entry_simple(
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        "Satoshi 2",
        Some("Second entry"),
        AddressCategory::Business
    );
    
    // Verify entries were added
    assert_eq!(address_book.len(), 2);
    
    // Get all entries
    let entries = address_book.get_all_entries();
    assert_eq!(entries.len(), 2);
    
    // Verify labels
    let labels: Vec<&str> = entries.iter().map(|e| e.label.as_str()).collect();
    assert!(labels.contains(&"Satoshi 1"));
    assert!(labels.contains(&"Satoshi 2"));
} 