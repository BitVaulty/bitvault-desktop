use std::panic;
use std::str::FromStr;
use bitcoin::{Address, Amount, Network, OutPoint, Script, Transaction, TxIn, TxOut, Txid};
use bdk::FeeRate;
use bitcoin::address::NetworkUnchecked;
use bitvault_common::types::*;
use bitvault_common::bitcoin_utils;
use bitvault_common::config::Config;
use anyhow::Result;

// Setup function to run at the beginning of each test to capture panics and log them
pub fn test_with_logging<T, F: FnOnce() -> T + panic::UnwindSafe>(
    name: &str,
    test_fn: F,
) -> Result<T, String> {
    eprintln!("===== STARTING TEST: {} =====", name);

    let result = panic::catch_unwind(|| test_fn());

    match result {
        Ok(value) => {
            eprintln!("===== TEST PASSED: {} =====", name);
            Ok(value)
        }
        Err(e) => {
            let panic_msg = if let Some(msg) = e.downcast_ref::<String>() {
                format!("{}", msg)
            } else if let Some(msg) = e.downcast_ref::<&str>() {
                format!("{}", msg)
            } else {
                "Unknown panic".to_string()
            };

            eprintln!("===== TEST FAILED: {} - PANIC: {} =====", name, panic_msg);
            Err(panic_msg)
        }
    }
}

/// Sample addresses for different networks and formats
pub struct TestAddresses;

impl TestAddresses {
    /// Returns a valid P2PKH address for the given network
    pub fn p2pkh(network: Network) -> Address {
        let addr_str = match network {
            Network::Bitcoin => "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
            Network::Testnet => "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn",
            Network::Regtest => "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn",
            Network::Signet => "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            _ => panic!("Unsupported network for test address"),
        };
        
        bitcoin_utils::parse_address(addr_str, network).unwrap()
    }

    /// Returns a valid P2WPKH (bech32) address for the given network
    pub fn p2wpkh(network: Network) -> Address {
        let addr_str = match network {
            Network::Bitcoin => "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
            Network::Testnet => "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            Network::Regtest => "bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080",
            Network::Signet => "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            _ => panic!("Unsupported network for test address"),
        };
        
        bitcoin_utils::parse_address(addr_str, network).unwrap()
    }

    /// Returns a valid P2WSH (bech32) address for the given network
    pub fn p2wsh(network: Network) -> Address {
        let addr_str = match network {
            Network::Bitcoin => "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3",
            Network::Testnet => "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7",
            Network::Regtest => "bcrt1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qzf4jry",
            Network::Signet => "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7",
            _ => panic!("Unsupported network for test address"),
        };
        
        bitcoin_utils::parse_address(addr_str, network).unwrap()
    }

    /// Returns an invalid address string that has correct format but wrong checksum
    pub fn invalid_checksum() -> String {
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t5".to_string() // Changed last char
    }

    /// Returns an address string with completely invalid format
    pub fn invalid_format() -> String {
        "invalid-bitcoin-address-format".to_string()
    }

    /// Returns an address from the wrong network (testnet address when Bitcoin network is expected)
    pub fn wrong_network(expected: Network) -> String {
        match expected {
            Network::Bitcoin => "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            _ => "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        }.to_string()
    }
}

/// Test transaction utilities
pub struct TestTransactions;

impl TestTransactions {
    /// Returns a sample transaction ID string
    pub fn sample_txid_str() -> &'static str {
        "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16"
    }

    /// Returns a sample Txid
    pub fn sample_txid() -> Txid {
        Txid::from_str(Self::sample_txid_str()).unwrap()
    }
}

/// Utility for creating wallet settings for testing
pub fn create_test_wallet_settings(network: Network) -> WalletSettings {
    WalletSettings {
        network,
        use_tor: false,
        default_fee_level: FeePriority::Medium,
        custom_fee_rate: Some(FeeRate::from_sat_per_vb(5.0)),
        display_as_bitcoin: true,
        preferred_fiat: "USD".to_string(),
        show_fiat_amounts: true,
        custom_settings: std::collections::HashMap::new(),
    }
}

/// Utility to create a test AddressInfo
pub fn create_test_address_info(network: Network, is_owned: bool) -> AddressInfo {
    let address = TestAddresses::p2wpkh(network);
    let info = AddressInfo::new(address, is_owned);
    
    if is_owned {
        return info.with_label("Test address".to_string());
    }
    
    info
}

/// Utility to create sample WalletTransaction objects for testing
pub fn create_test_wallet_transaction(is_sent: bool) -> WalletTransaction {
    let txid = TestTransactions::sample_txid();
    let timestamp = 1617184224; // Some fixed timestamp
    
    let amount = if is_sent {
        Amount::from_sat(150000) // 0.0015 BTC
    } else {
        Amount::from_sat(250000) // 0.0025 BTC
    };
    
    // Create the BDK transaction details
    let details = bdk::TransactionDetails {
        transaction: None, // We don't need the full transaction for most tests
        txid,
        received: if is_sent { 0 } else { amount.to_sat() },
        sent: if is_sent { amount.to_sat() } else { 0 },
        fee: if is_sent { Some(10000) } else { None },
        confirmation_time: None, // We'll avoid using ConfirmationTime directly
    };
    
    // Create a sample address info for the transaction
    let address_info = create_test_address_info(
        Network::Bitcoin,
        !is_sent, // If receiving, it's our address
    );
    
    WalletTransaction {
        details,
        memo: None,
        addresses: vec![address_info],
        timestamp: timestamp as u64, // We still use the timestamp in our struct
    }
}

pub fn load_default_config() -> Result<Config> {
    Config::load("path/to/default/config.toml")
}

pub fn load_custom_config(network: &str, fiat: &str) -> Result<Config> {
    let mut config = load_default_config()?;
    config.wallet.network = network.to_string();
    config.wallet.fiat_currency = fiat.to_string();
    Ok(config)
}
