# BitVault Common Library

This library provides common types, utilities, and interfaces for the BitVault Bitcoin wallet. It's designed as a security-conscious foundation for cross-process communication and shared functionality.

## Purpose

The BitVault Common library serves as a bridge between the security-isolated components of the Bitcoin wallet. It provides:

- Core Bitcoin-related types that can safely cross security boundaries
- Mathematical utilities for Bitcoin operations with safeguards against common errors
- Configuration management for application settings
- Security-aware logging that protects sensitive information

## Security Architecture

This library adheres to the security architecture defined in BitVault's design documents, particularly:

- No security-critical operations are performed
- Types are designed to be serializable for IPC transport
- Input validation is thorough and explicit
- Error types are descriptive but don't leak sensitive information

## Modules

### Types (`types.rs`)

Core Bitcoin data structures with validation and safety features:

- `Address` - Bitcoin address wrapper with validation
- `TransactionId` - Transaction ID with validation
- `BitcoinAmount` - Safe representation of Bitcoin amounts
- `WalletPath` - BIP32 derivation paths
- Error types for wallet operations

### Math (`math.rs`)

Mathematical utilities for Bitcoin operations:

- BTC/satoshi conversions with overflow protection
- Fee calculations
- Transaction size estimation
- Validation helpers for Bitcoin amounts
- Constants for Bitcoin transaction sizes

### Configuration (`config.rs`)

Application settings management:

- TOML-based configuration
- Default values for all settings
- Validation to ensure settings meet security requirements
- Platform-specific paths and settings

### Logging (`logging.rs`)

Security-aware logging infrastructure:

- Context-based logging for different parts of the application
- Sanitization of sensitive information
- Structured logging (plain text or JSON)
- Cross-boundary logging with security domain awareness

## Usage Guidelines

When using this library, keep the following in mind:

1. **Security Boundaries**: Types in this library can cross security boundaries, but operations should respect those boundaries
2. **No Secret Data**: Never store secrets (private keys, seeds, etc.) in any type from this library
3. **Validation**: Always validate inputs when crossing security boundaries
4. **Error Handling**: Use the provided error types for clear, secure error reporting

## Development Guidelines

When contributing to this library:

1. **No Direct Network Access**: This library should never directly access the network
2. **Minimal Dependencies**: Keep dependencies to a minimum to reduce attack surface
3. **Security First**: Always prioritize security over convenience or performance
4. **Cross-Platform**: Code should work consistently across all target platforms
5. **Test Thoroughly**: Include comprehensive tests for all functionality

## Examples

The `examples/` directory contains usage examples for the library components:

- Configuration management
- Bitcoin amount conversions and formatting
- Logging with security considerations

## Testing

Run tests with:

```bash
cargo test
```

Security-critical functionality has extensive test coverage to ensure correctness and safety.

Tests are organized by module and functionality:
- **address_tests.rs**: Address validation, parsing, and error handling
- **bitcoin_utils_tests.rs**: Bitcoin utility functions like formatting
- **bitcoin_amount_tests.rs**: Tests for Bitcoin amount handling
- **transaction_tests.rs**: Transaction ID validation and handling
- **memory_security_tests.rs**: Sensitive data handling and memory security
- **config_tests.rs**: Configuration loading and validation
- **logging_tests.rs**: Logging functionality and sanitization

The test suite leverages a shared `test_utils.rs` module for common testing utilities.

## Code Quality

The codebase follows these quality guidelines:

1. **No Redundant Code**: Duplicate functionality is eliminated
2. **Clean Imports**: Only necessary imports are included
3. **Proper Documentation**: All public APIs are documented with examples
4. **Consistent Naming**: Follows Rust naming conventions
5. **Memory Safety**: Special attention to secure memory handling
6. **Error Handling**: Comprehensive error types and handling

Regular code cleanup ensures the codebase remains maintainable and secure. 