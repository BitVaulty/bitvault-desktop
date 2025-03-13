use bitvault_common::address_book::{AddressBook, AddressCategory};
use bitcoin::Network;
use std::sync::Once;

// Initialize logging once for address book tests
static ADDRESS_BOOK_TEST_INIT: Once = Once::new();

fn setup_address_book_test() {
    ADDRESS_BOOK_TEST_INIT.call_once(|| {
        // No need to initialize logging here, just ensure we have test isolation
        // by using our own Once guard
    });
}

const MAINNET_ADDRESS: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";

#[test]
fn test_address_book_simple() {
    setup_address_book_test();
    
    // Create a new address book
    let address_book = AddressBook::new(Network::Bitcoin);
    
    // Check initial state
    assert_eq!(address_book.network(), Network::Bitcoin);
    assert!(address_book.is_empty());
    assert_eq!(address_book.len(), 0);
}

#[test]
fn test_add_and_get_entry() {
    setup_address_book_test();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add an entry
    let result = address_book.add_entry(
        MAINNET_ADDRESS,
        "Satoshi",
        Some("First Bitcoin address"),
        AddressCategory::Personal
    );
    
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
}

#[test]
fn test_find_by_category_simple() {
    setup_address_book_test();
    
    let mut address_book = AddressBook::new(Network::Bitcoin);
    
    // Add multiple entries with different categories
    let _ = address_book.add_entry(
        MAINNET_ADDRESS,
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
    
    let _ = address_book.add_entry(
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",  // Valid P2SH address instead of the invalid one
        "Personal Wallet",
        None,
        AddressCategory::Personal
    );
    
    // Find by category - Donation should have 2 entries
    let donation_results = address_book.find_by_category(&AddressCategory::Donation);
    assert_eq!(donation_results.len(), 2);
    
    // Find by category - Personal should have 1 entry
    let personal_results = address_book.find_by_category(&AddressCategory::Personal);
    assert_eq!(personal_results.len(), 1);
    
    // Find by non-existent category
    let exchange_results = address_book.find_by_category(&AddressCategory::Exchange);
    assert_eq!(exchange_results.len(), 0);
} 