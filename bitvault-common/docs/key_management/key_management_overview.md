# BitVault Key Management

This document describes the key management system in BitVault, which handles the most security-sensitive operations in the wallet.

## Overview

The key management module is responsible for:

- Generation of cryptographic key material (mnemonics, seeds, keys)
- Secure encryption of sensitive key material
- Password-based key derivation
- Key rotation for credential updates
- Secure storage and retrieval of wallet keys
- Memory protection for sensitive data

## Architecture

The key management system follows a layered architecture:

1. **API Layer** - High-level functions for common operations
2. **Encryption Layer** - Handles secure encryption/decryption
3. **Storage Layer** - Manages persistent storage of encrypted keys
4. **Memory Protection Layer** - Ensures secure handling in memory

## Security Model

This module sits at critical security boundaries between:

- **Volatile memory** (where keys are used) and **persistent storage** (encrypted files)
- **User input** (passwords) and **cryptographic material**
- **Application logic** and **cryptographic operations**

### Threat Model Assumptions

1. The underlying operating system's RNG is trustworthy
2. The user's password has sufficient entropy
3. The memory of the process is not directly readable by other processes
4. The encrypted key file might be accessible to attackers

## Cryptographic Primitives

- **AES-256-GCM** for authenticated encryption
- **PBKDF2-HMAC-SHA256** for key derivation, with adaptive iteration count
- **BIP39** for mnemonic generation
- **BIP32** for hierarchical deterministic key derivation
- **Secure random** for entropy generation

## Core API

The module exposes several key functions:

```rust
// Key generation
pub fn generate_mnemonic() -> Result<Mnemonic, WalletError>;
pub fn generate_mnemonic_and_key(password: &str) -> Result<(Mnemonic, ExtendedPrivKey), WalletError>;

// Key storage
pub fn encrypt_and_store_key(mnemonic: &Mnemonic, password: &str, path: &Path) -> Result<(), WalletError>;
pub fn decrypt_and_retrieve_key(password: &str, path: &Path) -> Result<(Mnemonic, ExtendedPrivKey), WalletError>;

// Key rotation
pub fn rotate_key(old_password: &str, new_password: &str, path: &Path) -> Result<(), WalletError>;

// Test utilities
pub fn set_test_mode(enabled: bool);
```

## Implementation Notes

### Memory Security

- Sensitive key material is automatically zeroized when no longer needed
- Memory locking is used to prevent keys from being swapped to disk
- The `secure_memzero` function ensures proper clearing of memory

### Key Derivation

- Adaptive key derivation parameters based on device performance
- Default iteration count balances security and performance
- Parameters are stored with encrypted data for future-proofing

### Test Mode

- Test mode for unit testing (disabled in production)
- Allows predictable keys for testing scenarios
- Automatically disabled in release builds

### Logging

- Detailed security logging that never includes sensitive material
- Operation successes and failures are logged
- Timing information for performance-sensitive operations

## Usage Examples

### Basic Key Generation and Storage

```rust
use bitvault_common::key_management::{
    generate_mnemonic_and_key,
    encrypt_and_store_key
};
use std::path::Path;

// Generate a new mnemonic and master key
let password = "secure_password";
let (mnemonic, master_key) = generate_mnemonic_and_key(password)?;

// Encrypt and store the key
let storage_path = Path::new("/path/to/wallet.dat");
encrypt_and_store_key(&mnemonic, password, storage_path)?;
```

### Key Retrieval

```rust
use bitvault_common::key_management::decrypt_and_retrieve_key;
use std::path::Path;

// Retrieve the key from storage
let password = "secure_password";
let storage_path = Path::new("/path/to/wallet.dat");
let (mnemonic, master_key) = decrypt_and_retrieve_key(password, storage_path)?;

// Use the key for wallet operations
// ...
```

### Key Rotation

```rust
use bitvault_common::key_management::rotate_key;
use std::path::Path;

// Change the wallet password
let old_password = "old_password";
let new_password = "new_secure_password";
let storage_path = Path::new("/path/to/wallet.dat");
rotate_key(old_password, new_password, storage_path)?;
```

## Security Best Practices

When working with the key management module:

1. **Never store unencrypted keys** in memory longer than necessary
2. **Zeroize sensitive data** after use
3. **Use strong, high-entropy passwords**
4. **Validate user input** before passing to cryptographic functions
5. **Handle errors securely** without leaking sensitive information

## Related Components

- [Security Boundaries](../security/security_boundaries.md) - Security model overview
- [Platform Module](../platform/platform_overview.md) - Platform-specific security features 