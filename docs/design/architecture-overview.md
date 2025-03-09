# BitVault Architecture Overview

This document defines BitVault's codebase architecture, component relationships, and implementation strategy to achieve our security model and Bitcoin functionality requirements.

## 1. Architectural Principles

### Core Design Values

- **Security Boundaries**: Strict isolation between security-critical and non-critical components
- **Minimal Trust Surface**: Least privilege principle applied throughout the architecture
- **Platform Adaptability**: Common core with platform-specific security adaptations
- **Bitcoin-First Design**: Optimized for Bitcoin operations, especially 2-of-3 multisignature workflows
- **Developer Ergonomics**: Clear module boundaries and responsibilities

### Architecture Style

- **Rust Workspace Structure**: Multi-crate organization with clear dependencies
- **Interface-Driven Design**: Abstract interfaces with platform-specific implementations
- **Capability-Based Security**: Components access only explicitly granted capabilities
- **Message-Passing Communication**: Well-defined communication between security domains
- **Event-Driven UI**: Reactive UI pattern with state management

## 2. Codebase Organization

### Primary Modules

1. **bitvault-core**: Security-critical operations
   - Runs in isolated security context
   - Handles all key operations and transaction signing
   - Implements security policy enforcement
   - Provides minimal, well-defined API surface

2. **bitvault-common**: Shared components
   - Type definitions and traits
   - Serialization formats for cross-boundary communication
   - Error types and handling patterns
   - Logging infrastructure (security-aware)
   - NO security-critical operations

3. **bitvault-ui**: User interface
   - Transaction construction and display
   - Wallet management interface
   - User workflow implementation
   - State management
   - NO access to private keys

4. **bitvault-app**: Platform integration
   - Platform-specific entry points and process management
   - Security feature detection and adaptation
   - IPC channel establishment and maintenance
   - Native platform integration

### Module Dependencies

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

### Security Boundary Placement

- Security boundary exists between bitvault-core and all other modules
- bitvault-common crosses the boundary but contains no security-critical code
- All private key operations confined to bitvault-core
- All communication across boundary occurs through well-defined APIs

## 3. bitvault-core (Security-Critical Module)

### Responsibilities

- Secure key generation, storage, and management
- Transaction signing and validation
- Security policy enforcement
- Cryptographic operations
- Authentication validation

### Key Components

1. **Key Management Subsystem**
   - Key generation and encryption (BIP32/39/44)
   - Key storage with platform-specific security
   - Key derivation for addresses
   - Hardware security integration (when available)

2. **Transaction Signing Service**
   - PSBT validation and policy compliance
   - Transaction script verification
   - Signature creation with appropriate keys
   - Multisignature coordination

3. **Security Policy Engine**
   - Policy storage and enforcement
   - Transaction rule verification
   - Authentication requirement validation
   - Spending limit tracking and enforcement

4. **Secure IPC Handler**
   - Message authentication and validation
   - Request processing with authorization
   - Response filtering to prevent data leakage
   - Session management and timeouts

### Bitcoin-Specific Components

1. **BDK Integration Layer**
   - Custom signer implementation that never exposes keys
   - Key store adapter for secure storage
   - Descriptor management for multisig
   - PSBT handling for partial signatures

2. **Multisignature Implementation**
   - 2-of-3 P2WSH scripts as primary wallet type
   - Key role assignment (device, backup, recovery)
   - Script path spending validation
   - Threshold verification

3. **Wallet State Management**
   - Public key and address derivation
   - Derivation path tracking
   - Wallet structure persistence (descriptors)
   - Recovery data management

### Security Measures

- All operations inside isolated process boundary
- Memory protection including zeroing sensitive data
- Explicit wiping of key material after use
- Minimal dependencies to reduce attack surface
- No direct network access from secure process

## 4. bitvault-common (Shared Components)

### Responsibilities

- Shared type definitions that cross security boundary
- Cross-boundary communication protocol definitions
- Serialization and validation schemes
- Error handling patterns
- Logging (security-aware)

### Key Components

1. **API Definitions**
   - Core API interface traits and types
   - Request/response structures
   - Error types and handling patterns
   - Capability interfaces

2. **Bitcoin Types**
   - Address types and validation utilities
   - Transaction representations (without private data)
   - UTXO management structures
   - Fee models and estimation

3. **Serialization Framework**
   - Message formats for cross-boundary IPC
   - Binary serialization for efficient transfer
   - Schema validation for security
   - Type safety enforcement

4. **Security Utilities**
   - Authentication data structures
   - Secure random number generation
   - Platform capability detection traits
   - Security level definitions and requirements

### Design Considerations

- No security-critical operations permitted
- Pure data types and utilities only
- Cross-platform compatibility
- Minimal external dependencies
- Comprehensive testing with security focus

## 5. bitvault-ui (User Interface Module)

### Responsibilities

- User interface rendering and interaction
- Wallet data presentation
- Transaction construction (unsigned)
- User workflow implementation
- State management

### Key Components

1. **State Management**
   - Wallet state representation (public data only)
   - Transaction history and status
   - User preferences
   - Session state tracking

2. **Transaction Builder**
   - Address validation and formatting
   - Amount calculation with unit conversion
   - Fee estimation and selection
   - UTXO selection algorithms
   - Unsigned transaction creation

3. **UI Component Library**
   - Wallet dashboard with balance and activity
   - Transaction construction forms
   - Address book and management
   - Security policy configuration interface
   - Key backup and recovery interfaces

4. **Core API Client**
   - Communication with secure process
   - Request formatting and validation
   - Response handling and parsing
   - Error management and retry logic
   - Session maintenance

### User Workflow Implementation

- Wallet creation and setup wizards
- Transaction construction and approval flow
- Address generation and management
- Backup and recovery procedures
- Security policy configuration

### Design Considerations

- No access to private keys or signing operations
- Clear security boundary indicators for users
- Graceful handling of secure process failures
- Comprehensive input validation before boundary crossing
- State recovery mechanisms after interruptions

## 6. bitvault-app (Platform Integration)

### Responsibilities

- Native application initialization
- Secure process management
- Platform capability detection
- Platform-specific security integration
- Environment-specific optimizations

### Key Components

1. **Process Management**
   - Secure process spawning with restricted permissions
     - On Linux: Process isolation with namespace separation and seccomp-BPF filters
       - System call filtering restricts allowed operations
       - Only necessary syscalls for cryptographic operations permitted
       - Prevents exploitation of vulnerabilities through system call restrictions
     - On macOS: Sandbox profiles and entitlement restrictions
     - On Windows: Job objects and restricted tokens
   - IPC channel establishment and maintenance
   - Process monitoring and heartbeat
   - Crash recovery and secure restart

2. **Platform Security Bridge**
   - Platform security feature detection
   - Secure storage integration
   - Authentication mechanism adaptation
   - Permission management and validation

3. **Application Lifecycle**
   - Initialization sequence with security checks
   - Graceful shutdown with secure cleanup
   - Update management and verification
   - Error recovery with security preservation

4. **Platform-Specific Adaptations**
   - Desktop: Process isolation with platform-specific IPC
   - Android: Keystore integration and security tier detection
   - iOS: Secure Enclave integration when available
   - Web: Worker isolation with security limitations (future)

### Platform-Specific Implementations

1. **Desktop-Specific**
   - Process isolation via separate OS processes
   - Platform-specific IPC (Unix sockets/Named pipes)
   - OS-specific secure storage integration
   - Permission restriction for secure process

2. **Android-Specific** (Future)
   - Hardware security module detection and use
   - Keystore integration with security tiers
   - Biometric authentication integration
   - Platform-specific IPC mechanisms

3. **iOS-Specific** (Future)
   - Secure Enclave integration
   - Keychain secure storage
   - Biometric (Face ID/Touch ID) authentication
   - App sandbox security model

## 7. Security Boundary Enforcement

### Process Isolation Strategy

1. **Desktop Implementation**
   - Separate OS processes with restricted permissions
   - IPC using platform-specific mechanisms:
     - Unix domain sockets (Linux/macOS)
     - Named pipes (Windows)
   - Minimal permissions for secure process
   - Strict message validation at boundary

2. **Android Implementation** (Future)
   - Process isolation where available
   - Hardware-backed keystore when available
   - Tiered security based on device capabilities
   - Clear security status indicators

3. **iOS Implementation** (Future)
   - Secure Enclave for key operations when available
   - App sandbox with extension isolation
   - Keychain for secure storage
   - Biometric authentication binding

### Cross-Boundary Communication

1. **Message Protocol**
   - Strictly typed request/response pairs
   - Explicit schema validation on both sides
   - Authentication tokens for request validation
   - Minimal data transfer with need-to-know principle

2. **Authentication Mechanism**
   - Session-based authentication with timeout
   - Request signing for validation using HMAC-SHA256
     - Hash-based Message Authentication Code provides integrity and authenticity
     - Session-specific key material never crosses security boundary
     - Authentication covers critical message fields including operation, request ID, and payload
     - Each message contains unique nonce to prevent replay attacks
     - Constant-time verification prevents timing side-channel attacks
   - Capability-based access control for operations
   - Rate limiting and anomaly detection

3. **Session Management**
   - Explicit session establishment with authentication
   - Regular session validation
   - Automatic timeout after inactivity
   - Explicit termination capabilities

## 8. Technology Selection

### Core Technologies

1. **Programming Language**: Rust
   - Memory safety guarantees for security
   - Strong type system to prevent errors
   - Cross-compilation for all target platforms
   - Performance suitable for cryptographic operations
   - Robust ecosystem with security-focused libraries

2. **Bitcoin Implementation**: Bitcoin Development Kit (BDK)
   - Well-maintained Bitcoin operations library
   - Descriptor-based wallet management
   - PSBT support for multisignature workflows
   - Extensible design for security customization

3. **UI Framework**: egui
   - Cross-platform immediate mode GUI
   - Rust-native implementation (security benefit)
   - Minimal dependencies
   - Good performance on all platforms

4. **Serialization**: serde
   - Type-safe serialization/deserialization
   - Multiple format support (bincode for binary efficiency)
   - Custom serializer support for security
   - Schema validation capabilities

### Security-Specific Libraries

1. **Cryptography**: ring
   - Well-audited cryptographic implementations
   - Side-channel resistance for key operations
   - Modern algorithms with regular updates
   - Constant-time operations for security

2. **Secure Storage**:
   - Platform keychain/keyring integration:
     - Secret Service API (Linux)
     - Keychain Services (macOS)
     - DPAPI (Windows)
     - Keystore (Android)
     - Secure Enclave/Keychain (iOS)
   - Encrypted file fallback with Argon2id KDF

3. **Memory Protection**: zeroize
   - Secure memory wiping to prevent key extraction
   - Protection against compiler optimization
   - Cross-platform consistent behavior
   - Integration with cryptographic types

## 9. Cross-Platform Strategy

### Platform Adaptation Approach

1. **Core Functionality**: Platform-agnostic implementation
   - Common Bitcoin operations and validation
   - Core security policy engine
   - Key derivation and management logic
   - Transaction construction and signing

2. **Platform Bridge**: Capability-based adaptation
   - Runtime security capability detection
   - Selection of optimal security mechanisms
   - Fallback paths for missing features
   - Clear security status indicators

3. **Compilation Targets** (Prioritized):
   - Phase 1: Linux (x86_64, aarch64)
   - Phase 1: macOS (x86_64, aarch64)
   - Phase 1: Windows (x86_64)
   - Phase 2: Android (aarch64, armv7)
   - Phase 3: iOS (aarch64)
   - Phase 4: Web (wasm32) with security limitations

### Feature Flag Strategy

- `feature = "desktop"`: Desktop-specific process isolation
- `feature = "android"`: Android-specific security features
- `feature = "ios"`: iOS/Secure Enclave features
- `feature = "web"`: Web/WASM with security limitations
- `feature = "hardware-security"`: Hardware wallet support
- `feature = "development"`: Development tools and logging

## 10. Testing Strategy

### Security-Focused Testing

1. **Boundary Testing**
   - IPC security validation with adversarial inputs
   - Message format fuzzing for parser vulnerabilities
   - Authentication bypass attempt simulation
   - Privilege escalation testing

2. **Cryptographic Validation**
   - Key generation verification against test vectors
   - Transaction signing validation with test cases
   - Policy enforcement with boundary testing
   - Side-channel resistance testing where applicable

3. **Process Isolation Verification**
   - Validate process separation effectiveness
   - Memory access control verification
   - Crash recovery with security preservation
   - Resource limitation and denial of service testing

### Bitcoin-Specific Testing

1. **Transaction Testing**
   - Address generation verification against test vectors
   - Transaction construction correctness
   - Fee calculation accuracy
   - Signature verification against test vectors
   - PSBT handling with test cases

2. **Wallet Operation Testing**
   - Multisig script creation verification
   - Key derivation path testing
   - Wallet recovery with various key combinations
   - Backup verification and restoration

3. **Network Interaction**
   - Transaction broadcast security
   - Network fee estimation accuracy
   - UTXO tracking and management
   - Confirmation handling and verification

### Test Suite Organization

1. **Unit Tests**: Within each module
   - Component-level testing with mocks
   - Security property verification
   - Edge case handling
   - Performance benchmarks for critical operations

2. **Integration Tests**: Cross-module testing
   - Security boundary validation
   - End-to-end workflow verification
   - Platform-specific feature testing
   - Recovery from failure states

3. **Security Auditing Integration**:
   - Static analysis in CI pipeline
   - Dependency vulnerability scanning
   - Fuzz testing for critical parsers
   - Memory safety validation

## 11. Implementation Roadmap

### Phase 1: Core Architecture (Q1-Q2)

1. **Security Boundary Implementation**
   - Process isolation on desktop platforms
   - IPC mechanism with authentication
   - Basic session management
   - Memory protection for sensitive data

2. **Key Management Foundation**
   - Basic key generation and secure storage
   - 2-of-3 multisignature structure
   - BDK integration with custom signers
   - Backup and verification workflows

3. **Minimal UI**
   - Wallet creation and setup
   - Basic transaction construction
   - Security status indicators
   - Backup guidance interface

### Phase 2: Bitcoin Operations (Q2-Q3)

1. **Multisignature Workflow**
   - Complete 2-of-3 signing process
   - PSBT handling between devices
   - Key role management (device, backup, recovery)
   - Script and address verification

2. **Transaction Lifecycle**
   - Full transaction construction with UTXO selection
   - Fee management and estimation
   - Change address handling
   - Transaction history and status tracking

3. **Security Policy Engine**
   - Spending limits and thresholds
   - Authentication requirements
   - Address whitelisting/restrictions
   - Time-based constraints

### Phase 3: Platform Expansion (Q3-Q4)

1. **Android Implementation**
   - Keystore integration with security tiers
   - Platform-specific UI adaptations
   - Cross-device signing coordination
   - Security capability detection

2. **iOS Implementation**
   - Secure Enclave integration
   - Keychain secure storage
   - Biometric authentication
   - Platform-specific security features

3. **Enhanced Security Features**
   - Hardware wallet support
   - Advanced recovery options
   - Additional wallet security policies
   - Enhanced backup strategies

## 12. Development Guidelines

### Security-Critical Development

1. **Code Review Requirements**
   - Mandatory security-focused review for core module
   - Secondary review for boundary-crossing code
   - Explicit verification of security properties
   - Regular security audits by specialists

2. **Dependency Management**
   - Minimal dependencies in security-critical code
   - Regular vulnerability scanning in CI
   - Pinned dependency versions with hash verification
   - Vendoring critical dependencies when necessary

3. **Coding Standards**
   - Explicit error handling with no unwrap() in production
   - Memory safety best practices (e.g., zeroize after use)
   - No unsafe code without thorough review and documentation
   - Comprehensive testing of security-critical paths

### Bitcoin-Specific Guidelines

1. **BDK Integration Patterns**
   - Standard descriptor formats for compatibility
   - Custom signer implementations for security boundaries
   - Consistent PSBT handling practices
   - Clear documentation of Bitcoin-specific security considerations

2. **Multisignature Implementation**
   - 2-of-3 P2WSH as primary wallet type
   - Key separation with clear documentation
   - Recovery procedures with verification
   - Testing against known Bitcoin test vectors 