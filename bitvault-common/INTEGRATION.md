# BitVault Common: Integration Points

This document outlines how the `bitvault-common` library integrates with other components of the BitVault wallet. Understanding these integration points is essential for effective development across the codebase.

## Overview of Component Relationships

```
bitvault-app
    ├── bitvault-ui
    │       ├── bitvault-common
    │       
    ├── bitvault-common
    │
    └── bitvault-core
            └── bitvault-common
```

## 1. Integration with bitvault-core (Security-Critical Module)

The `bitvault-core` component contains security-critical operations and handles private keys. It uses `bitvault-common` primarily for:

### Key Management

- **Mnemonic Generation**: Uses `key_management::generate_mnemonic_and_key` for creating new wallets
- **Secure Storage**: Uses `key_management::encrypt_and_store_key` for protecting keys at rest
- **Key Retrieval**: Uses `key_management::decrypt_and_retrieve_key` for accessing keys securely
- **Key Rotation**: Uses `key_management::rotate_key` for credential updates
- **Password Verification**: Uses `key_management::verify_password` for authentication

### Types and Serialization

- **Public Types**: Uses `bitvault-common::types` for public, non-sensitive data structures that can cross security boundaries
- **Serialization Format**: Uses common serialization/deserialization patterns for secure IPC communication
- **Error Types**: Uses `WalletError` and other error types for consistent error handling
- **Sensitive Data Types**: Uses `SensitiveString` and `SensitiveBytes` for memory-secure handling of sensitive data

### Mathematical Operations

- **Fee Calculation**: Uses `calculate_fee` and related functions to determine appropriate transaction fees
- **Amount Handling**: Uses `is_dust_amount` to enforce minimum output values
- **Economic Constraints**: Uses `min_economical_change` to determine when to create change outputs

### Secure Logging

- **Sanitized Logging**: Uses `logging` module to ensure sensitive data is not inadvertently logged
- **Structured Logging**: Follows common logging patterns for consistent log formats
- **Security Context**: Uses `log_security` functions for security-sensitive operations

### Implementation Example

```rust
// Inside bitvault-core
use bitvault_common::{
    types::{WalletError, SensitiveString, AddressInfo},
    calculate_fee, estimate_tx_size,
    logging,
    key_management
};

// Generating a new wallet
pub fn create_new_wallet(password: &str) -> Result<(), WalletError> {
    // Generate a new mnemonic and master key
    let (mnemonic, key) = key_management::generate_mnemonic_and_key(password)?;
    
    // Store the encrypted key securely
    key_management::encrypt_and_store_key(&key, &mnemonic, password, "wallet.dat")?;
    
    logging::log_security(
        LogLevel::Info,
        "Created new wallet",
        None
    );
    
    Ok(())
}

pub fn build_transaction(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Result<Transaction, WalletError> {
    // Estimate the size of the transaction
    let estimated_size = estimate_tx_size(inputs.len(), outputs.len());
    
    // Calculate the fee based on the estimated size
    let fee = calculate_fee(estimated_size, self.fee_rate);
    
    // Log the operation (without sensitive details)
    logging::log_transaction(
        LogLevel::Info,
        "Building transaction",
        Some(json!({
            "inputs_count": inputs.len(),
            "outputs_count": outputs.len(),
            "estimated_size": estimated_size,
            "fee": fee.to_sat()
        }))
    );
    
    // ... perform secure transaction building operations ...
}
```

## 2. Integration with bitvault-ui (User Interface)

The UI component never handles private keys but needs to present Bitcoin data to users in a meaningful way. It uses `bitvault-common` for:

### Fee Estimation

- **Dynamic Fee Rates**: Uses `fee_estimation` module to provide up-to-date fee options
- **Multiple Strategies**: Presents different fee options based on user priorities
- **Fee History**: Shows fee trends using historical data from `fee_estimation`
- **Network Congestion**: Displays network congestion levels from `network_status`

### UTXO Management and Selection

- **Coin Selection Visualization**: Shows which UTXOs would be selected with different strategies
- **Privacy Analysis**: Uses `utxo_selection` to evaluate privacy implications of transactions
- **Change Outputs**: Explains change output creation to users
- **Advanced Coin Control**: Enables user-controlled UTXO selection for advanced users

### Bitcoin Formatting and Validation

- **Address Validation**: Uses `bitcoin_utils::is_valid_bitcoin_address` to validate user input
- **Amount Formatting**: Uses `bitcoin_utils::format_bitcoin_amount` for consistent display of amounts
- **Amount Parsing**: Uses `bitcoin_utils::parse_bitcoin_amount` to safely parse user input

### Transaction Fee Estimation

- **Fee Estimation**: Uses `calculate_fee` with different fee rates to show fee options
- **Transaction Size Estimation**: Uses `estimate_tx_size` to provide feedback on transaction complexity
- **Balance Handling**: Uses amount utilities to calculate available balance after fees

### Localization Support

- **Translations**: Uses localization module for consistent formatting across languages
- **Amount Display**: Uses locale-aware formatting of amounts

### Implementation Example

```rust
// Inside bitvault-ui
use bitvault_common::{
    bitcoin_utils::{format_bitcoin_amount, is_valid_bitcoin_address, parse_bitcoin_amount},
    calculate_fee, estimate_tx_size,
    localization::{format_amount, tr, BitcoinUnit},
    fee_estimation::{FeeRecommendations, create_recommendations},
    utxo_selection::{Utxo, UtxoSelector, SelectionStrategy},
    network_status::{NetworkStatus, NetworkStatusProvider, CongestionLevel}
};

fn render_send_form(
    utxos: &[Utxo],
    amount: Amount,
    network_status: &NetworkStatus
) {
    // Get current fee recommendations
    let fee_recommendations = create_recommendations(
        network_status.network,
        network_status.congestion
    );
    
    // Calculate fees for different priority levels
    let low_fee = fee_recommendations.get_fee_for_priority(FeePriority::Low);
    let medium_fee = fee_recommendations.get_fee_for_priority(FeePriority::Medium);
    let high_fee = fee_recommendations.get_fee_for_priority(FeePriority::High);
    
    // Display fee options to user
    display_fee_option("Low Priority", low_fee, "May take several hours");
    display_fee_option("Medium Priority", medium_fee, "Usually within an hour");
    display_fee_option("High Priority", high_fee, "Usually within 10 minutes");
    
    // Preview coin selection with default strategy
    let selector = UtxoSelector::new();
    let selection_result = selector.select_utxos(
        utxos,
        amount,
        SelectionStrategy::MinimizeFee
    );
    
    match selection_result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Display selected UTXOs and change information
            display_utxo_selection(&selected, fee_amount, change_amount);
        },
        SelectionResult::InsufficientFunds { available, required } => {
            show_error(tr("insufficient_funds", Some(&HashMap::from([
                ("available", &format_amount(available).to_string()),
                ("required", &format_amount(required).to_string())
            ]))));
        }
    }
    
    // Show privacy recommendations
    if network_status.congestion == CongestionLevel::Low {
        display_privacy_recommendation(
            "Consider using 'Maximize Privacy' coin selection during low congestion periods."
        );
    }
}
```

## 3. Integration with bitvault-ipc (Inter-Process Communication)

The IPC layer handles communication between secure and non-secure components. It uses `bitvault-common` for:

### Message Types

- **API Types**: Uses common types for request/response structures
- **Serialization**: Uses consistent serialization formats
- **Event System**: Uses `events` module for communication between components

### Error Handling

- **Error Types**: Propagates errors between components with consistent error types
- **Error Formatting**: Uses standardized error formatting for user presentation

### Implementation Example

```rust
// Inside bitvault-ipc
use bitvault_common::{
    types::{WalletError, AddressInfo, WalletTransaction},
    bitcoin_utils::parse_address,
    events::{MessageBus, EventType, MessagePriority, IpcMessage}
};

pub fn handle_create_transaction_request(
    request: CreateTransactionRequest,
    message_bus: &MessageBus
) -> Result<CreateTransactionResponse, WalletError> {
    // Validate the recipient address
    let recipient_address = parse_address(&request.address, request.network)?;
    
    // Forward the request to the core process via message bus
    let request_id = message_bus.publish(
        EventType::CoreRequest,
        &serde_json::to_string(&CoreRequest::CreateTransaction {
            address: recipient_address,
            amount: request.amount,
            fee_rate: request.fee_rate,
        }).unwrap(),
        MessagePriority::High
    );
    
    // Wait for response...
    let response = wait_for_response(request_id);
    
    // ... process response ...
    
    Ok(CreateTransactionResponse {
        transaction: response.transaction,
        fee: response.fee,
    })
}
```

## 4. Integration with Testing Infrastructure

The testing infrastructure uses `bitvault-common` extensively to test wallet functionality:

### Test Utilities

- **Address Test Vectors**: Uses `test_utils::TestAddresses` for standard Bitcoin address test data
- **Transaction Test Vectors**: Uses `test_utils::TestTransactions` for standard transaction test data
- **Deterministic Testing**: Uses `key_management::set_test_mode` to enable deterministic testing of crypto functions
- **Test UTXO Generation**: Creates test UTXOs with `utxo_selection::Utxo` for selection algorithm testing

### Mock Components

- **Mock Network Status**: Uses `network_status::MockNetworkStatusProvider` for offline testing
- **Mock Platform**: Uses `platform::MockPlatformProvider` for cross-platform testing

### Implementation Example

```rust
// In test file
use bitvault_common::{
    test_utils::{TestAddresses, TestTransactions, test_with_logging},
    key_management::{set_test_mode, generate_mnemonic_and_key},
    utxo_selection::{Utxo, UtxoSelector, SelectionStrategy}
};

#[test]
fn test_utxo_selection_with_fee_estimation() {
    // Set up test environment
    set_test_mode(true);
    
    // Create test UTXOs
    let utxos = vec![
        Utxo::new(
            OutPoint::new(
                TestTransactions::sample_txid(),
                0
            ),
            Amount::from_sat(10_000),
            0,
            false
        ),
        Utxo::new(
            OutPoint::new(
                TestTransactions::sample_txid(),
                1
            ),
            Amount::from_sat(50_000),
            2,
            true
        )
    ];
    
    // Use the selector
    let selector = UtxoSelector::new();
    let target_amount = Amount::from_sat(30_000);
    let result = selector.select_utxos(&utxos, target_amount, SelectionStrategy::MinimizeFee);
    
    // Verify results...
}
```

## 5. Maintaining Appropriate Security Boundaries

When using `bitvault-common` across security boundaries, always remember:

1. **No Sensitive Data**: Types in `bitvault-common` should NEVER contain private keys, seeds, or sensitive credentials
2. **Validation at Boundaries**: Always validate data when crossing security boundaries
3. **Minimal Dependencies**: Reduce attack surface by limiting external dependencies
4. **Sanitized Logging**: Be careful not to log sensitive data, even inadvertently
5. **Security-First Design**: Prioritize security over convenience

### Security Considerations for Specific Modules

#### Key Management

The key management module (`key_management.rs`) is particularly security-sensitive:

- **Security Boundary**: This module sits at a critical security boundary between volatile memory and persistent storage
- **Threat Model**: Assumes the encrypted key file might be accessible to attackers
- **Memory Safety**: Uses `SensitiveString` and `SensitiveBytes` for automatic memory zeroing
- **Cryptographic Primitives**: Uses AES-256-GCM for encryption and PBKDF2-HMAC-SHA256 for key derivation
- **Test Mode**: Includes a strict separation between test code and production code

Example of proper usage across security boundaries:

```rust
// Security-critical side (bitvault-core)
// Never expose the key or mnemonic directly to non-security-critical components
fn authenticate_user(password: &str) -> Result<AuthToken, WalletError> {
    // Verify the password without exposing keys
    match key_management::verify_password(password, "wallet.dat")? {
        true => {
            // Password correct, generate a limited-time auth token
            let token = generate_auth_token();
            Ok(token)
        },
        false => {
            // Password incorrect
            Err(WalletError::InvalidArgument("Invalid password".to_string()))
        }
    }
}

// Non-security-critical side (bitvault-ui)
// Only receives the auth token, never handles keys
fn login_user(password: &str) -> Result<(), Error> {
    match request_authentication(password) {
        Ok(token) => {
            store_auth_token(token);
            navigate_to_main_screen();
            Ok(())
        },
        Err(e) => {
            show_error_message(e);
            Err(e)
        }
    }
}
```

#### UTXO Selection and Management

The UTXO selection and management modules (`utxo_selection.rs` and `utxo_management.rs`) handle transaction input selection but do NOT process private keys or signatures. However, they contain important security considerations:

- **Privacy Impact**: UTXO selection strategies directly impact user privacy
- **Fee Accuracy**: Inaccurate fee calculations could lead to stuck transactions or excessive fees
- **Input Validation**: Input data must be validated, especially when coming from untrusted sources
- **Performance with Large Sets**: Algorithms should perform efficiently even with large UTXO sets
- **Advanced Algorithms**: Branch and Bound implementation provides optimal fee efficiency

Example of proper usage across security boundaries:

```rust
// In bitvault-ui (non-security-critical component)
let utxo_list = get_utxos_from_core(); // Data comes from secure core

// Use selection algorithm (non-sensitive operation)
let selector = UtxoSelector::new();
let result = selector.select_utxos(&utxo_list, amount, SelectionStrategy::MinimizeFee);

// For advanced selection, use specialized algorithms
if user_prefers_privacy() {
    let privacy_result = selector.select_utxos(&utxo_list, amount, SelectionStrategy::MaximizePrivacy);
    show_privacy_comparison(result, privacy_result);
}

// Send selected UTXOs back to core for signing (crossing back to secure zone)
send_to_core(CoreRequest::SignTransaction {
    utxos: result.selected,
    recipient: validated_address,
    amount: validated_amount,
});
```

#### Address Book

The address book module (`address_book.rs`) stores Bitcoin addresses and labels, but not private keys:

- **No Private Data**: Store only public addresses, never private keys
- **Input Validation**: Always validate addresses when adding to address book
- **User Labels**: Be careful with user-provided labels (potential XSS or injection issues)
- **Persistence Security**: When persisting address book data, use appropriate OS security mechanisms
- **Categorization**: Uses categories for better organization of addresses

#### Fee Estimation

The fee estimation module (`fee_estimation.rs`) calculates transaction fees:

- **Network Interaction**: Fee estimation may require network requests, which should be handled securely
- **Default Fallbacks**: Always have safe default values if estimation fails
- **Validation**: Validate fee rates to prevent excessive fees or dust outputs
- **Adaptive Estimation**: Uses network congestion, historical data, and time-of-day for better estimates
- **Multiple Sources**: Combines data from multiple sources for reliability

#### Network Status

The network status module (`network_status.rs`) tracks Bitcoin network status:

- **Network Requests**: Handle network requests securely
- **Untrusted Data**: Always treat responses as untrusted data
- **Failure Handling**: Gracefully handle network failures
- **Minimal Information**: Only track needed information to limit data exposure
- **Congestion Detection**: Provides congestion levels for fee estimation

## Future Development Considerations

When extending `bitvault-common` functionality, consider these cross-component implications:

1. **Core Impact**: Will new functionality be used by security-critical code? If so, ensure it's thoroughly reviewed
2. **UI Requirements**: Does the UI need this functionality for user-facing features?
3. **IPC Transport**: Will new types need to be serialized across security boundaries?
4. **Platform Consistency**: Will it work across all supported platforms?
5. **Performance**: For operations that may be called frequently, consider performance implications
6. **Testing Strategy**: How will the new functionality be tested securely?
7. **Security Boundaries**: Does the functionality cross security boundaries?

## Cross-Platform Compatibility Considerations

BitVault supports multiple platforms, and `bitvault-common` must work consistently across all of them. When modifying or extending the library, consider these cross-platform concerns:

### File System Access

- **Path Separators**: Use the platform module's `path_join` function instead of manually joining paths
- **Storage Locations**: Use platform-specific storage locations via the platform module
- **File Locking**: Be aware that file locking behaves differently across platforms
- **Key Storage**: Use platform-specific secure storage when available via abstraction

Example:
```rust
// Instead of this:
let config_path = format!("{}/config.json", some_dir);

// Do this:
let config_path = platform::path_join(some_dir, "config.json");

// For secure storage, use platform capabilities
if platform::get_platform_capabilities().has_secure_storage {
    // Use platform-specific secure storage
} else {
    // Fall back to file-based encrypted storage
}
```

### Concurrency and Threading

- **Thread Limits**: Mobile platforms may have stricter thread limits than desktop
- **Thread Priorities**: Thread priorities work differently across platforms
- **Background Processing**: Background processing constraints vary significantly
- **Event System**: The event system must work consistently across different threading models

### Network Access

- **Connection Limits**: Mobile platforms often limit simultaneous connections
- **Background Networking**: Background network access may be restricted on mobile
- **Proxy Settings**: Different platforms handle proxy settings differently
- **Fee Estimation Services**: Be aware of platform-specific networking restrictions for fee estimation services

### Memory Constraints

- **Memory Limits**: Mobile devices have much lower memory limits
- **Large UTXO Sets**: Be especially careful with memory usage for large UTXO sets
- **Caching**: Implement adaptive caching based on platform capabilities
- **Key Derivation**: Adapt key derivation parameters based on platform performance

### Platform-Specific Features

- **Secure Storage**: Use the platform module to access platform-specific secure storage
- **Biometrics**: Authentication capabilities vary by platform
- **Background Operation**: Background execution constraints are platform-specific
- **Secure Enclaves**: Take advantage of secure enclaves when available

### Testing Considerations

- **Test Isolation**: Ensure tests run reliably on all platforms
- **Test Vectors**: Use consistent test vectors across platforms
- **Platform-Specific Mocks**: Provide platform-specific mock implementations for testing
- **Performance Testing**: Test performance on low-end devices

When implementing new functionality, test thoroughly on all target platforms before committing, paying special attention to resource-constrained mobile environments.

By understanding these integration points, you can develop functionality in `bitvault-common` that effectively supports all components of the BitVault wallet while maintaining appropriate security boundaries. 