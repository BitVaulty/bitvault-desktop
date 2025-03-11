use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Network};
use bitvault_common::bitcoin_utils;
use bitvault_common::WalletError;
use std::str::FromStr;
mod test_utils;
use test_utils::test_with_logging;

#[test]
fn test_basic_address_validation() {
    // Valid address
    let valid_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    assert!(bitcoin_utils::is_valid_bitcoin_address(
        valid_address,
        Network::Bitcoin
    ));

    // Invalid address
    assert!(!bitcoin_utils::is_valid_bitcoin_address(
        "invalid",
        Network::Bitcoin
    ));
}

#[test]
fn test_address_parsing() {
    // Valid address
    let valid_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    let result = bitcoin_utils::parse_address(valid_address, Network::Bitcoin);
    assert!(result.is_ok());

    // Invalid address
    let invalid_result = bitcoin_utils::parse_address("invalid-addr", Network::Bitcoin);
    assert!(matches!(
        invalid_result,
        Err(WalletError::InvalidAddress(_))
    ));
}

#[test]
fn test_address_network_handling() {
    // Network mismatch
    let testnet_address = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
    let network_mismatch = bitcoin_utils::parse_address(testnet_address, Network::Bitcoin);
    assert!(matches!(
        network_mismatch,
        Err(WalletError::InvalidNetworkType(_))
    ));
}

#[test]
fn test_address_validation_with_logging() {
    let _ = test_with_logging("test_address_validation", || {
        // Valid addresses
        let res1 = bitcoin_utils::is_valid_bitcoin_address(
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
            Network::Bitcoin,
        );
        assert!(res1);

        let res2 = bitcoin_utils::is_valid_bitcoin_address(
            "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
            Network::Bitcoin,
        );
        assert!(res2);

        let res3 = bitcoin_utils::is_valid_bitcoin_address(
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
            Network::Bitcoin,
        );
        assert!(res3);

        // Invalid addresses
        let res4 = bitcoin_utils::is_valid_bitcoin_address("invalid", Network::Bitcoin);
        assert!(!res4);

        let res5 = bitcoin_utils::is_valid_bitcoin_address("", Network::Bitcoin);
        assert!(!res5);

        // Wrong network
        let res6 = bitcoin_utils::is_valid_bitcoin_address(
            "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            Network::Bitcoin,
        );
        assert!(!res6);
    });
}

#[test]
fn test_address_display_with_logging() {
    let _ = test_with_logging("test_address_display", || {
        // Parse the address directly with the expected network
        let address_str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";

        let unchecked_addr =
            Address::<NetworkUnchecked>::from_str(address_str).expect("Failed to parse address");

        // Check that the network is correct
        assert_eq!(unchecked_addr.network, Network::Bitcoin);

        // Convert to checked address for display
        let addr = unchecked_addr.assume_checked();

        // Check the string representation
        let addr_str = addr.to_string();
        assert_eq!(addr_str, address_str);
    });
}

#[test]
fn test_address_validation_errors() {
    let _ = test_with_logging("test_address_validation_errors", || {
        // Test invalid address format
        let invalid_addr = "invalid-addr";
        let invalid_result = bitcoin_utils::parse_address(invalid_addr, Network::Bitcoin);

        assert!(matches!(
            invalid_result,
            Err(WalletError::InvalidAddress(_))
        ));

        if let Err(WalletError::InvalidAddress(err_str)) = invalid_result {
            // Ensure error contains context
            assert!(err_str.contains("Invalid address format"));
        }

        // Test network mismatch (testnet address with mainnet network)
        let testnet_address = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
        let network_mismatch = bitcoin_utils::parse_address(testnet_address, Network::Bitcoin);

        assert!(matches!(
            network_mismatch,
            Err(WalletError::InvalidNetworkType(_))
        ));

        // Test address validity check
        let res1 = bitcoin_utils::is_valid_bitcoin_address(invalid_addr, Network::Bitcoin);
        assert!(!res1);

        let res2 = bitcoin_utils::is_valid_bitcoin_address(testnet_address, Network::Bitcoin);
        assert!(!res2);

        let res3 = bitcoin_utils::is_valid_bitcoin_address(
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
            Network::Bitcoin,
        );
        assert!(res3);
    });
}
