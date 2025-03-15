//! Input validation and sanitization for secure operations
//!
//! # Security Model
//!
//! This module provides validation functions for all security-sensitive inputs in the BitVault wallet.
//! It serves as a critical security boundary for external data entering the system.
//!
//! ## Security Boundaries
//!
//! This module sits at several critical security boundaries:
//! - Between user input and internal processing
//! - Between external data sources (RPC, APIs) and wallet operations
//! - Between serialized formats and their parsed representations
//! - Between UI inputs and cryptographic operations
//!
//! ## Threat Model Assumptions
//!
//! 1. All external input is potentially malicious and must be validated
//! 2. Input validation occurs before sensitive operations are performed
//! 3. Special attention is needed for data crossing trust boundaries
//! 4. Validation errors must be descriptive without leaking sensitive information
//!
//! ## Security Considerations
//!
//! - All address validation must verify the correct network type
//! - String inputs must be checked for injection attacks (SQL, script)
//! - Amount validation must enforce bounds to prevent overflow/underflow
//! - Transaction validation ensures no unexpected behaviors in transaction structure
//! - URL validation prevents open redirect and SSRF vulnerabilities
//! - Derivation path validation prevents potential key leakage through unusual paths
//!
//! # Usage
//!
//! This module should be used at all trust boundaries in the application:
//! - When accepting user input from the UI
//! - When receiving data from external APIs
//! - When parsing data from storage
//! - Before performing any security-sensitive operation

use bitcoin::{Address, Network, Transaction, TxIn, TxOut, Script, OutPoint};
use bitcoin::address::Payload;
use bitcoin::hashes::Hash;
use regex::Regex;
use thiserror::Error;
use crate::types::WalletError;
use std::str::FromStr;
use std::collections::HashSet;
use url::Url;
use serde_json::Value;

/// Errors that can occur during validation
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid Bitcoin address: {0}")]
    InvalidAddress(String),
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Invalid input value: {0}")]
    InvalidInput(String),
    
    #[error("Security risk detected: {0}")]
    SecurityRisk(String),
    
    #[error("Network mismatch: {0}")]
    NetworkMismatch(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl From<ValidationError> for WalletError {
    fn from(err: ValidationError) -> Self {
        match err {
            ValidationError::InvalidAddress(msg) => WalletError::InvalidArgument(format!("Invalid address: {}", msg)),
            ValidationError::InvalidTransaction(msg) => WalletError::InvalidArgument(format!("Invalid transaction: {}", msg)),
            ValidationError::InvalidInput(msg) => WalletError::InvalidArgument(format!("Invalid input: {}", msg)),
            ValidationError::SecurityRisk(msg) => WalletError::Security(format!("Security risk: {}", msg)),
            ValidationError::NetworkMismatch(msg) => WalletError::Configuration(format!("Network mismatch: {}", msg)),
            ValidationError::InvalidUrl(msg) => WalletError::InvalidArgument(format!("Invalid URL: {}", msg)),
            ValidationError::InvalidAmount(msg) => WalletError::InvalidArgument(format!("Invalid amount: {}", msg)),
            ValidationError::InvalidFormat(msg) => WalletError::InvalidArgument(format!("Invalid format: {}", msg)),
        }
    }
}

// Bitcoin address validation functions
//-------------------------------------

/// Validates if a string is a valid Bitcoin address for a specific network
pub fn validate_bitcoin_address(address: &str, network: Network) -> Result<Address, ValidationError> {
    // Try to parse the address
    let addr = Address::from_str(address)
        .map_err(|e| ValidationError::InvalidAddress(format!("Failed to parse address: {}", e)))?;
    
    // Check if the address is for the expected network
    if addr.network != network {
        return Err(ValidationError::NetworkMismatch(
            format!("Address {} is for network {:?}, but expected {:?}", 
                    address, addr.network, network)
        ));
    }
    
    // Check if the address is of a supported type
    match addr.payload {
        Payload::PubkeyHash(_) => (),
        Payload::ScriptHash(_) => (),
        Payload::WitnessProgram { version, program } => {
            // Check for v0 (P2WPKH, P2WSH) or v1 (Taproot)
            if version > 1 {
                return Err(ValidationError::InvalidAddress(
                    format!("Unsupported witness version: {}", version)
                ));
            }
            
            // For v0, program must be 20 bytes (P2WPKH) or 32 bytes (P2WSH)
            if version == 0 && program.len() != 20 && program.len() != 32 {
                return Err(ValidationError::InvalidAddress(
                    format!("Invalid program length {} for witness v0", program.len())
                ));
            }
            
            // For v1 (Taproot), program must be 32 bytes (public key)
            if version == 1 && program.len() != 32 {
                return Err(ValidationError::InvalidAddress(
                    format!("Invalid program length {} for Taproot (v1)", program.len())
                ));
            }
        }
        _ => return Err(ValidationError::InvalidAddress(
            format!("Unsupported address type: {:?}", addr.payload)
        )),
    }
    
    Ok(addr)
}

/// Check if an address is compatible with the network and has a secure type
/// This function is useful for recipient address validation
pub fn validate_recipient_address(address: &str, network: Network) -> Result<Address, ValidationError> {
    let addr = validate_bitcoin_address(address, network)?;
    
    // Additional security checks for recipient addresses
    // Log warning for p2sh addresses (could be multisig or other complex scripts)
    if let Payload::ScriptHash(_) = addr.payload {
        log::warn!("P2SH address used as recipient: {}. Verify carefully.", address);
    }
    
    Ok(addr)
}

/// Validate multiple addresses
pub fn validate_addresses(addresses: &[String], network: Network) -> Result<Vec<Address>, ValidationError> {
    let mut valid_addresses = Vec::with_capacity(addresses.len());
    
    for address in addresses {
        valid_addresses.push(validate_bitcoin_address(address, network)?);
    }
    
    Ok(valid_addresses)
}

// Transaction validation functions
//--------------------------------

/// Validates a Bitcoin transaction with comprehensive checks
pub fn validate_transaction(tx: &Transaction, network: Network) -> Result<(), ValidationError> {
    // Empty transactions are invalid
    if tx.input.is_empty() {
        return Err(ValidationError::InvalidTransaction("Transaction has no inputs".to_string()));
    }
    
    if tx.output.is_empty() {
        return Err(ValidationError::InvalidTransaction("Transaction has no outputs".to_string()));
    }
    
    // Check for duplicate inputs (possible double-spend attempt)
    let mut input_outpoints = HashSet::new();
    for input in &tx.input {
        if !input_outpoints.insert(input.previous_output) {
            return Err(ValidationError::SecurityRisk(
                format!("Duplicate input detected: {}", input.previous_output)
            ));
        }
    }
    
    // Check output values
    for (i, output) in tx.output.iter().enumerate() {
        // Check for dust outputs (typically 546 satoshis, but can be network dependent)
        // This could vary based on fee rates and script types, so this is a simplified check
        if output.value < 546 && !output.script_pubkey.is_op_return() {
            log::warn!("Output {} may be dust: {} sats", i, output.value);
        }
        
        // Check for suspicious OP_RETURN outputs
        if output.script_pubkey.is_op_return() && output.value > 0 {
            return Err(ValidationError::InvalidTransaction(
                format!("OP_RETURN output {} has non-zero value: {} sats", i, output.value)
            ));
        }
        
        // Validate that output addresses are valid for the network
        if let Some(address) = extract_address_from_script(&output.script_pubkey, network) {
            // This mainly checks that the address matches the expected network
            validate_bitcoin_address(&address.to_string(), network)?;
        }
    }
    
    // Check transaction size and weight for reasonableness
    let size = tx.size();
    let weight = tx.weight();
    
    // Extremely large transactions are suspicious
    if size > 100_000 {
        return Err(ValidationError::SecurityRisk(
            format!("Transaction is unusually large: {} bytes", size)
        ));
    }
    
    // Check for abnormally high fees (potential mistake)
    // This would typically be done at a higher level with input value information
    
    // Check for sequence-based time locks
    for input in &tx.input {
        if input.sequence & 0x80000000 == 0 {
            log::info!("Transaction uses sequence-based time locks");
            break;
        }
    }
    
    // Check for locktime
    if tx.lock_time > 0 {
        log::info!("Transaction has non-zero locktime: {}", tx.lock_time);
    }
    
    Ok(())
}

/// Validate transaction outputs for security and correctness
pub fn validate_outputs(outputs: &[TxOut], network: Network) -> Result<(), ValidationError> {
    if outputs.is_empty() {
        return Err(ValidationError::InvalidTransaction("No outputs provided".to_string()));
    }
    
    let mut total_value = 0;
    for (i, output) in outputs.iter().enumerate() {
        // Check for dust outputs
        if output.value < 546 && !output.script_pubkey.is_op_return() {
            log::warn!("Output {} may be dust: {} sats", i, output.value);
        }
        
        // Check for extremely large output values (might be a mistake)
        if output.value > 1_000_000_000_000 { // 10,000 BTC
            return Err(ValidationError::InvalidAmount(
                format!("Output {} has suspiciously large value: {} sats", i, output.value)
            ));
        }
        
        // Validate script type and address
        if let Some(address) = extract_address_from_script(&output.script_pubkey, network) {
            validate_bitcoin_address(&address.to_string(), network)?;
        } else if !output.script_pubkey.is_op_return() {
            // Non-standard output that's not OP_RETURN
            log::warn!("Output {} has non-standard script: {}", i, output.script_pubkey);
        }
        
        total_value += output.value;
    }
    
    // Check for unreasonably high total value
    if total_value > 2_100_000_000_000_000 { // More than 21 million BTC
        return Err(ValidationError::InvalidAmount(
            format!("Total output value exceeds Bitcoin supply: {} sats", total_value)
        ));
    }
    
    Ok(())
}

/// Helper function to extract an address from a script, if possible
fn extract_address_from_script(script: &Script, network: Network) -> Option<Address> {
    Address::from_script(script, network).ok()
}

// Input string validation functions
//--------------------------------

/// Validate an amount string (e.g., "0.1" or "0.1 BTC")
pub fn validate_amount_string(amount: &str) -> Result<f64, ValidationError> {
    // Strip 'BTC' suffix if present
    let amount_str = amount.trim().to_lowercase();
    let cleaned_amount = amount_str
        .strip_suffix(" btc")
        .or_else(|| amount_str.strip_suffix("btc"))
        .unwrap_or(&amount_str);
    
    // Parse the amount as a float
    match cleaned_amount.parse::<f64>() {
        Ok(value) => {
            if value < 0.0 {
                Err(ValidationError::InvalidAmount(
                    format!("Amount cannot be negative: {}", amount)
                ))
            } else if value > 21_000_000.0 {
                Err(ValidationError::InvalidAmount(
                    format!("Amount exceeds total Bitcoin supply: {}", amount)
                ))
            } else {
                Ok(value)
            }
        },
        Err(_) => Err(ValidationError::InvalidAmount(
            format!("Failed to parse amount: {}", amount)
        )),
    }
}

/// Validate a Bitcoin Core compatible fee rate string (e.g., "0.00001" or "10 sat/vB")
pub fn validate_fee_rate_string(fee_rate: &str) -> Result<f64, ValidationError> {
    let fee_str = fee_rate.trim().to_lowercase();
    
    // Handle different fee rate formats
    let cleaned_fee = if fee_str.contains("sat/v") || fee_str.contains("sat/b") {
        // Extract number portion from "X sat/vB" format
        let parts: Vec<&str> = fee_str.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ValidationError::InvalidFormat(
                format!("Invalid fee rate format: {}", fee_rate)
            ));
        }
        parts[0]
    } else {
        // Assume plain number in BTC/kB
        fee_str.as_str()
    };
    
    // Parse the fee rate as a float
    match cleaned_fee.parse::<f64>() {
        Ok(value) => {
            if value < 0.0 {
                Err(ValidationError::InvalidAmount(
                    format!("Fee rate cannot be negative: {}", fee_rate)
                ))
            } else if value > 10_000.0 {
                // 10,000 sat/vB is an extremely high fee rate
                Err(ValidationError::InvalidAmount(
                    format!("Fee rate is unreasonably high: {}", fee_rate)
                ))
            } else {
                Ok(value)
            }
        },
        Err(_) => Err(ValidationError::InvalidFormat(
            format!("Failed to parse fee rate: {}", fee_rate)
        )),
    }
}

/// Validate a URL string for security and format correctness
pub fn validate_url(url_str: &str) -> Result<Url, ValidationError> {
    // Parse the URL
    let url = Url::parse(url_str)
        .map_err(|e| ValidationError::InvalidUrl(format!("Failed to parse URL: {}", e)))?;
    
    // Ensure scheme is https (for security)
    if url.scheme() != "https" {
        return Err(ValidationError::SecurityRisk(
            format!("Non-HTTPS URL is insecure: {}", url_str)
        ));
    }
    
    // Check for valid host
    if url.host_str().is_none() {
        return Err(ValidationError::InvalidUrl(
            format!("URL has no host: {}", url_str)
        ));
    }
    
    // Check for suspicious hosts/TLDs
    // This is a simplistic check - in practice you'd use a reputation database or more sophisticated checks
    let host = url.host_str().unwrap().to_lowercase();
    
    // Check for IP addresses as hosts (often suspicious)
    if host.chars().all(|c| c.is_digit(10) || c == '.') {
        log::warn!("URL uses IP address as host: {}", host);
    }
    
    // Check for unusual TLDs (simplified check)
    let suspicious_tlds = ["tk", "ml", "ga", "cf", "gq"];
    for tld in suspicious_tlds {
        if host.ends_with(&format!(".{}", tld)) {
            log::warn!("URL uses potentially suspicious TLD: {}", host);
            break;
        }
    }
    
    Ok(url)
}

/// Validate a JSON string for correct format
pub fn validate_json(json_str: &str) -> Result<Value, ValidationError> {
    serde_json::from_str(json_str)
        .map_err(|e| ValidationError::InvalidFormat(format!("Invalid JSON: {}", e)))
}

/// Validate a hexadecimal string
pub fn validate_hex(hex_str: &str) -> Result<(), ValidationError> {
    // Check if string is valid hex
    if !hex_str.chars().all(|c| c.is_digit(16)) {
        return Err(ValidationError::InvalidFormat(
            format!("Invalid hexadecimal string: {}", hex_str)
        ));
    }
    
    // Check length is even (hex strings representing byte arrays should have even length)
    if hex_str.len() % 2 != 0 {
        return Err(ValidationError::InvalidFormat(
            format!("Hexadecimal string has odd length: {}", hex_str.len())
        ));
    }
    
    Ok(())
}

/// Validate input against SQL injection patterns
pub fn validate_no_sql_injection(input: &str) -> Result<(), ValidationError> {
    let sql_patterns = [
        r"(?i)'\s*OR\s*", // 'OR 
        r"(?i);\s*DROP", // ;DROP
        r"(?i);\s*SELECT", // ;SELECT
        r"(?i)UNION\s+SELECT", // UNION SELECT
        r"(?i)--", // SQL comment
        r"(?i)/\*.*\*/", // /* comment */
    ];
    
    for pattern in sql_patterns {
        if Regex::new(pattern).unwrap().is_match(input) {
            return Err(ValidationError::SecurityRisk(
                format!("Potential SQL injection attempt detected in input")
            ));
        }
    }
    
    Ok(())
}

/// Validate input against common script injection patterns
pub fn validate_no_script_injection(input: &str) -> Result<(), ValidationError> {
    let script_patterns = [
        r"(?i)<script", // <script tag
        r"(?i)javascript:", // javascript: protocol
        r"(?i)on\w+\s*=", // event handlers like onclick=
        r"(?i)<iframe", // <iframe tag
        r"(?i)data:text/html", // data: URL with HTML content
    ];
    
    for pattern in script_patterns {
        if Regex::new(pattern).unwrap().is_match(input) {
            return Err(ValidationError::SecurityRisk(
                format!("Potential script injection attempt detected in input")
            ));
        }
    }
    
    Ok(())
}

/// Validate a derivation path string (m/purpose'/coin_type'/account'/change/address_index)
pub fn validate_derivation_path(path: &str) -> Result<(), ValidationError> {
    // BIP32/BIP44 derivation path format
    let path_regex = Regex::new(r"^m(/\d+'?)*$").unwrap();
    
    if !path_regex.is_match(path) {
        return Err(ValidationError::InvalidFormat(
            format!("Invalid derivation path format: {}", path)
        ));
    }
    
    // Split the path and check each component
    let components: Vec<&str> = path.split('/').collect();
    
    // Should start with 'm'
    if components.is_empty() || components[0] != "m" {
        return Err(ValidationError::InvalidFormat(
            format!("Derivation path must start with 'm': {}", path)
        ));
    }
    
    // Check depth (shouldn't be excessively deep)
    if components.len() > 10 {
        return Err(ValidationError::InvalidFormat(
            format!("Derivation path is too deep: {}", path)
        ));
    }
    
    // For each component (skipping 'm')
    for component in &components[1..] {
        // Check if it has the hardened marker
        let is_hardened = component.ends_with('\'');
        
        // Extract the numeric part
        let num_part = if is_hardened {
            component.trim_end_matches('\'')
        } else {
            component
        };
        
        // Validate the numeric part
        match num_part.parse::<u32>() {
            Ok(index) => {
                // BIP32 indices must be less than 2^31
                if index >= 0x80000000 {
                    return Err(ValidationError::InvalidFormat(
                        format!("Index too large in derivation path: {}", index)
                    ));
                }
            },
            Err(_) => {
                return Err(ValidationError::InvalidFormat(
                    format!("Invalid index in derivation path: {}", num_part)
                ));
            }
        }
    }
    
    Ok(())
}

/// Validate a label or description string for wallet entries
pub fn validate_label(label: &str) -> Result<(), ValidationError> {
    // Check length
    if label.len() > 500 {
        return Err(ValidationError::InvalidInput(
            format!("Label is too long (max 500 characters): {} chars", label.len())
        ));
    }
    
    // Check for potentially dangerous content
    validate_no_script_injection(label)?;
    validate_no_sql_injection(label)?;
    
    Ok(())
}

/// Validate a string as a valid Bitcoin Core descriptor string
pub fn validate_descriptor(descriptor: &str) -> Result<(), ValidationError> {
    // Simplified descriptor validation (full validation would parse the descriptor)
    // Basic format checks 
    
    // Should start with a descriptor type
    let descriptor_types = ["pk", "pkh", "wpkh", "sh", "wsh", "combo", "addr", "raw", "tr"];
    let starts_with_type = descriptor_types.iter().any(|&t| descriptor.starts_with(&format!("{}(", t)));
    
    if !starts_with_type {
        return Err(ValidationError::InvalidFormat(
            format!("Invalid descriptor format: {}", descriptor)
        ));
    }
    
    // Check for matching parentheses
    let mut depth = 0;
    for c in descriptor.chars() {
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth < 0 {
                return Err(ValidationError::InvalidFormat(
                    format!("Unmatched parentheses in descriptor: {}", descriptor)
                ));
            }
        }
    }
    
    if depth != 0 {
        return Err(ValidationError::InvalidFormat(
            format!("Unmatched parentheses in descriptor: {}", descriptor)
        ));
    }
    
    // Check for checksum if present
    if descriptor.contains('#') {
        let parts: Vec<&str> = descriptor.split('#').collect();
        if parts.len() != 2 || parts[1].len() != 8 || !parts[1].chars().all(|c| c.is_digit(16)) {
            return Err(ValidationError::InvalidFormat(
                format!("Invalid descriptor checksum: {}", descriptor)
            ));
        }
    }
    
    Ok(())
}

/// Validate RPC input parameters to prevent injection
pub fn validate_rpc_input(method: &str, params: &str) -> Result<(), ValidationError> {
    // Validate the method name
    if method.contains(|c: char| !c.is_alphanumeric() && c != '_') {
        return Err(ValidationError::SecurityRisk(
            format!("Invalid RPC method name: {}", method)
        ));
    }
    
    // Validate the parameters as valid JSON
    validate_json(params)?;
    
    // Check for common injection patterns
    validate_no_sql_injection(params)?;
    validate_no_script_injection(params)?;
    
    Ok(())
}

// High-level validation functions for common wallet operations
//------------------------------------------------------------

/// Validate a transaction creation request with outputs and fee information
pub fn validate_tx_request(
    outputs: &[TxOut], 
    fee_rate: f64,
    network: Network
) -> Result<(), ValidationError> {
    // Validate outputs
    validate_outputs(outputs, network)?;
    
    // Validate fee rate
    if fee_rate < 0.0 {
        return Err(ValidationError::InvalidAmount(
            format!("Fee rate cannot be negative: {}", fee_rate)
        ));
    }
    
    if fee_rate > 5000.0 {
        // Warning for extremely high fee rates (over 5000 sat/vB)
        log::warn!("Unusually high fee rate: {} sat/vB", fee_rate);
    }
    
    // Check total output value for reasonableness
    let total_value: u64 = outputs.iter().map(|o| o.value).sum();
    if total_value > 2_100_000_000_000_000 { // More than 21 million BTC
        return Err(ValidationError::InvalidAmount(
            format!("Total output value exceeds Bitcoin supply: {} sats", total_value)
        ));
    }
    
    Ok(())
}

/// Validate a seed phrase (mnemonic)
pub fn validate_mnemonic(mnemonic: &str) -> Result<(), ValidationError> {
    // Split the mnemonic into words
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    
    // Check word count (BIP39 requires 12, 15, 18, 21, or 24 words)
    let valid_word_counts = [12, 15, 18, 21, 24];
    if !valid_word_counts.contains(&words.len()) {
        return Err(ValidationError::InvalidFormat(
            format!("Invalid mnemonic word count: {}", words.len())
        ));
    }
    
    // In a real implementation, we would check against the BIP39 wordlist
    // and validate the checksum, but that would require additional dependencies
    
    // Check that all words use only lowercase letters
    for word in &words {
        if !word.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(ValidationError::InvalidFormat(
                format!("Mnemonic words must be lowercase: {}", word)
            ));
        }
        
        // BIP39 words are 3-8 characters
        if word.len() < 3 || word.len() > 8 {
            return Err(ValidationError::InvalidFormat(
                format!("Word has invalid length: {}", word)
            ));
        }
    }
    
    Ok(())
} 