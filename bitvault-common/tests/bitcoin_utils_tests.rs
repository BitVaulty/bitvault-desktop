use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount, Network};
use bitvault_common::bitcoin_utils;
use bitvault_common::WalletError;
use std::str::FromStr;

#[test]
fn test_address_parsing() {
    // Test parsing a valid address - should succeed
    let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";

    // Test is_valid_bitcoin_address
    let is_valid = bitcoin_utils::is_valid_bitcoin_address(address, Network::Bitcoin);
    assert!(is_valid);

    // Test parse_address
    match bitcoin_utils::parse_address(address, Network::Bitcoin) {
        Ok(addr) => {
            assert_eq!(addr.network, Network::Bitcoin);
        }
        Err(e) => {
            panic!("Failed to parse valid address: {:?}", e);
        }
    }

    // Test with invalid address
    let invalid_address = "invalid_address";

    // Test is_valid_bitcoin_address
    let is_valid = bitcoin_utils::is_valid_bitcoin_address(invalid_address, Network::Bitcoin);
    assert!(!is_valid);

    // Test parse_address
    match bitcoin_utils::parse_address(invalid_address, Network::Bitcoin) {
        Ok(_) => {
            panic!("Successfully parsed invalid address!");
        }
        Err(e) => {
            assert!(matches!(e, WalletError::InvalidAddress(_)));
        }
    }
}

#[test]
fn test_amount_formatting() {
    // Test amount creation and formatting
    let amount = Amount::from_sat(123456789);

    // Format as BTC
    let btc_format = bitcoin_utils::format_bitcoin_amount(amount, true);
    assert_eq!(btc_format, "1.23456789 BTC");

    // Format as satoshis
    let sat_format = bitcoin_utils::format_bitcoin_amount(amount, false);
    assert_eq!(sat_format, "123456789 sats");
}

#[test]
fn test_address_types() {
    // Test different address types
    let addresses = [
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",         // P2PKH
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",         // P2SH
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", // P2WPKH
    ];

    for &addr_str in &addresses {
        // Ensure the address is valid
        assert!(bitcoin_utils::is_valid_bitcoin_address(
            addr_str,
            Network::Bitcoin
        ));

        // Parse the address
        let unchecked = Address::<NetworkUnchecked>::from_str(addr_str).unwrap();
        assert_eq!(unchecked.network, Network::Bitcoin);

        // Check our parse function
        let parsed = bitcoin_utils::parse_address(addr_str, Network::Bitcoin).unwrap();
        assert_eq!(parsed.to_string(), addr_str);
    }
}
