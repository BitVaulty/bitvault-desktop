# BitVault Platform Module

This document provides an overview of the platform abstraction layer in BitVault.

## Purpose

The platform module provides a consistent interface for platform-specific functionality:

- File system access and paths
- Memory protection operations
- Secure storage mechanisms
- Platform capabilities detection
- Directory management

## Architecture

The platform module uses a provider-based architecture:

- **PlatformProvider Trait**: Defines the interface for all platform-specific operations
- **Platform-Specific Implementations**: Separate implementations for each supported platform
- **Global Provider Access**: Single entry point through the `platform()` function

## Supported Platforms

BitVault currently supports the following platforms:

- Linux (`linux.rs`)
- macOS (`macos.rs`)
- Windows (`windows.rs`)
- Android (`android.rs`)
- iOS (`ios.rs`)

Additionally, the platform module includes:

- Mock implementation for testing (`mock.rs`)
- Generic fallback implementation (`generic.rs`)

## Core API

The platform module exposes the following key functionality:

### Directory Management

```rust
// Get application directories
pub fn get_data_dir() -> Result<PathBuf, PlatformError>;
pub fn get_config_dir() -> Result<PathBuf, PlatformError>;
pub fn get_logs_dir() -> Result<PathBuf, PlatformError>;
pub fn get_temp_dir() -> Result<PathBuf, PlatformError>;
```

### Memory Security

```rust
// Memory protection operations
pub fn mem_lock(data: &mut [u8]) -> Result<(), PlatformError>;
pub fn mem_unlock(data: &mut [u8]) -> Result<(), PlatformError>;
pub fn secure_memzero(data: &mut [u8]);
```

### Platform Information

```rust
// Platform type and capabilities
pub fn get_platform_type() -> PlatformType;
pub fn get_platform_capabilities() -> PlatformCapabilities;
```

## Security Considerations

The platform module handles several security-sensitive operations:

1. **Secure Memory Management**
   - Memory locking to prevent swapping of sensitive data
   - Secure zeroing of memory to prevent data leakage
   - Platform-specific memory protection features

2. **Secure Storage Access**
   - Platform-specific secure storage (keychain, keyring, etc.)
   - Configuration file security
   - Wallet data protection

3. **Directory Management**
   - Secure creation of wallet directories
   - Proper permissions for sensitive files
   - Isolation of wallet data from other applications

## Implementation Details

### PlatformProvider Trait

Each platform implementation must provide the `PlatformProvider` trait defined in `provider.rs`:

```rust
pub trait PlatformProvider: Send + Sync {
    fn get_platform_type(&self) -> PlatformType;
    fn get_platform_capabilities(&self) -> PlatformCapabilities;
    fn get_data_dir(&self) -> Result<PathBuf, PlatformError>;
    fn get_config_dir(&self) -> Result<PathBuf, PlatformError>;
    fn get_logs_dir(&self) -> Result<PathBuf, PlatformError>;
    fn get_temp_dir(&self) -> Result<PathBuf, PlatformError>;
    fn mem_lock(&self, data: &mut [u8]) -> Result<(), PlatformError>;
    fn mem_unlock(&self, data: &mut [u8]) -> Result<(), PlatformError>;
    fn secure_memzero(&self, data: &mut [u8]);
    // Additional methods...
}
```

### Platform Selection

The appropriate platform implementation is selected at compile time based on target platform detection:

```rust
// Platform-specific implementations
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "ios")]
mod ios;
```

## Usage Examples

### Basic Usage

```rust
use bitvault_common::platform;

// Get platform information
let platform_type = platform::get_platform_type();
let capabilities = platform::get_platform_capabilities();

// Get directories
let data_dir = platform::get_data_dir().expect("Failed to get data directory");
let config_dir = platform::get_config_dir().expect("Failed to get config directory");

// Use secure memory operations
let mut sensitive_data = vec![1, 2, 3, 4, 5];
platform::mem_lock(&mut sensitive_data).expect("Failed to lock memory");
// Use the data...
platform::mem_unlock(&mut sensitive_data).expect("Failed to unlock memory");
platform::secure_memzero(&mut sensitive_data);
```

### Provider-Based Usage

```rust
use bitvault_common::platform;

// Get the global platform provider
let provider = platform::platform();

// Use the provider directly
let data_dir = provider.get_data_dir().expect("Failed to get data directory");
```

## Related Components

- [Security Boundaries](../security/security_boundaries.md) - Security considerations for platform integration
- [Architecture Overview](../architecture/updated_architecture.md) - Platform's role in the overall architecture 