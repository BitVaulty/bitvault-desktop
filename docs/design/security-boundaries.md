# BitVault Security Boundaries Specification

This document defines the security boundaries for BitVault's MVP implementation, detailing how security-critical operations are isolated from the general application. The architecture is designed to be extensible for future Lightning Network and Liquid integration.

## 1. Core Security Domain

### Critical Operations
The following operations MUST execute within the secure boundary:
- Private key share generation, storage, and access
- Threshold signature operations
- Seed phrase handling and recovery
- Key derivation (BIP32/39/44)
- Security policy enforcement
- **Chain-specific cryptographic operations**: Architecture supports future chain-specific operations within appropriate security contexts

### Non-Critical Operations
These operations run in the main application domain:
- UI rendering and interaction
- Network communication
- Transaction construction (pre-signing)
- Address generation (using public keys only)
- UTXO tracking and management
- Fee estimation
- **Chain synchronization**: All blockchain and network state monitoring

### Protected Data
The following data must NEVER leave the security boundary:
- Private key shares (in any form)
- Seed phrases / mnemonic words
- Key derivation paths with private components
- Signing operations internal state
- Security policy private parameters
- **Chain-specific secrets**: Future provisions for channel states or blinding keys

## 2. Security Isolation Model

### Desktop Implementation
- **Mechanism**: Security boundary implemented via process isolation
- **Secure Process**: Minimal-privilege process containing security-critical operations
- **UI Process**: Main application process with network access and UI rendering
- **Permissions**: Secure process runs with reduced permissions (no network access)
- **Domain Architecture**: Security model designed with separate capability domains
- **Extensible Boundaries**: Process model can accommodate future hot/cold wallet separation
- **Startup**: UI process launches secure process with appropriate isolation
- **Memory Protection**: 
  - mlock() to prevent memory from being swapped
  - Guard pages around sensitive data
  - Memory encryption for sensitive data with session keys
  - Automatic zeroization using RAII pattern in Rust
  - **Security Tiers**: Memory protection strategies tiered by sensitivity level

### Browser/WASM Implementation
- **Mechanism**: Web Worker isolation with message passing
- **Limitations**: Accept reduced security guarantees in browser environment
- **Mitigations**: Clear user warnings about browser security limitations
- **Storage**: Use browser IndexedDB with encryption for persistence
- **Key Protection**: Memory encryption for in-use keys
- **Thread Isolation**: Dedicated Web Worker for secure operations
- **Domain Separation**: Clear logical separation between security domains, even with limited physical isolation

### Mobile Implementation
- **iOS**: 
  - Leverage Secure Enclave for key operations where available
  - XPC Services for more robust isolation when appropriate
  - Memory encryption for threshold signature components
  - **Capability Detection**: Runtime discovery of available security features
- **Android**: 
  - Use Android Keystore system for key protection
  - Tiered security approach based on device capabilities
  - StrongBox Keymaster when available, with hardware attestation
  - **Security Classification**: Clear security tier communication to users
- **Common**: Application sandbox provides basic isolation
- **Thread Isolation**: When process isolation is unavailable, use thread isolation with:
  - Dedicated thread with minimal permissions for secure operations
  - Memory barriers between threads handling sensitive data
  - Thread local storage clearing after sensitive operations
  - Thread sanitization before and after key operations
  - **Capability-Based Architecture**: Thread isolation designed with capability-based security model

## 3. Inter-Process Communication

### IPC Mechanism
- **Desktop**: 
  - Unix domain sockets with peer credential verification (Linux/macOS)
  - Socket files with 0600 permissions in /run/user/UID/
  - Named pipes with strict ACLs (Windows)
- **Browser**: PostMessage API between main thread and Web Worker
- **Mobile**: Platform-specific IPC (internal message passing)
- **Protocol Design**: Messaging protocol includes chain-type identifiers and operation classifiers

### Message Structure
- Unique request identifier
- Operation name/method
- Chain type identifier
- Operation parameters
- Authorization metadata
- Security domain classifier
- **Extensible Format**: Message serialization designed for extensibility

### Security Measures
- Request authentication using HMAC-SHA256 with session key established at startup
  - Key derivation using HKDF from master session key
  - Message-specific authentication covering all critical fields
  - Unique nonce per message to prevent replay attacks
  - Time-based message expiration to limit attack window
  - Constant-time HMAC verification to prevent timing attacks
- All sensitive data serialized/deserialized using explicit type schemas
- Validation of all parameters before processing in secure context
- Rate limiting of sensitive operations
- Timeout and automatic shutdown of idle secure process
- **Message Typing**: Strong typing of all messages with chain-specific validations

## 4. Authorization Model

### Request Validation
1. UI process constructs and displays operation details to user
2. User explicitly approves sensitive operations
3. UI passes approved request to secure process with authorization metadata
4. Secure process verifies authorization before execution
5. Secure process enforces security policies independently of UI
6. **Operation Classification**: Security operations classified by sensitivity and chain type

### Security Policies (MVP)
- Transaction amount thresholds requiring additional confirmation
- Simple spending limits (daily/weekly maximum)
- Explicit user confirmation for all transactions
- Basic recipient address whitelisting
- **Policy Framework**: Security policy engine designed with extensible rule system
- **Capability-Based Authorization**: Access control based on explicit capabilities rather than general permissions

## 5. Key Management

### Storage Approach
- **Desktop**: 
  - **Linux**: Secret Service API or GNOME Keyring with application-specific encryption
  - **macOS**: Keychain Services with hardware-assisted encryption
  - **Windows**: DPAPI or Credential Manager with user-account bound encryption
- **Browser**: Encrypted in IndexedDB with key derived from user password
- **Mobile**: 
  - **iOS**: Secure Enclave plus Keychain with appropriate protection classes
  - **Android**: Keystore with hardware backing when available
- **Key Purpose Architecture**: Keys organized by purpose with clear security classifications

### Key Share Access
- Private key shares NEVER exposed to UI process
- Key shares remain encrypted in memory when not in active use
- For 2-of-3 TSS: key shares distributed (1 on device, 1 on separate device/backup, 1 recovery)
- Encryption keys derived from user password with strong KDF (Argon2id)
- **Security Domains**: Key access governed by domain-specific security policies
- **Purpose Restrictions**: Keys restricted to specific operations by type

## 6. Bitcoin Transaction Flow

### Transaction Construction
1. UI process constructs unsigned transaction using UTXO data and BDK
2. UI displays transaction details to user for approval
3. Approved unsigned transaction passed to secure process
4. Secure process validates transaction against security policies
5. Secure process performs threshold signing operation and returns signed transaction to UI
6. UI broadcasts signed transaction to network
7. **Chain-Type Handling**: Transaction flow includes chain type identification for future extensions

### Data Passed to Secure Process
- Unsigned transaction (with outputs, amounts, fees)
- Required derivation paths (public paths only)
- Transaction metadata (purpose, user approval status)
- Chain type identifier
- Authorization proof (user confirmation hash)
- **Validation Context**: Complete data needed for policy validation

### Data Returned from Secure Process
- Signed transaction ready for broadcast
- Success/failure status
- Non-sensitive error details if applicable
- Security policy compliance status
- **Chain-Specific Results**: Result format adapts to transaction type

## 7. Error Handling

### Error Categories
- **Security Violations**: Minimal details, logged securely
- **User Errors**: Clear guidance without exposing implementation details
- **System Errors**: Generic errors with internal logging
- **Chain-Specific Errors**: Specialized error types for different operations

### Error Response Structure
- Reference to original request
- Success/failure indicator
- Error code and user-friendly message
- Recoverability status
- Suggested user action
- **Error Registry**: Centralized error type system with clear security classifications

## 8. Platform-Specific Security Implementations

### Linux Implementation
- **Process Isolation**: 
  - Use `clone()` with namespace isolation for the secure process
  - Apply seccomp-BPF filters to restrict syscalls in secure process
    - Whitelist only necessary syscalls for cryptographic operations and IPC
    - Default deny policy for all non-whitelisted syscalls
    - Prevent network access, file creation, and other risky operations
    - Separate filter configurations for different operational modes
  - Capability dropping to limit privileges (drop all but CAP_IPC_LOCK)
  - Mount private /tmp directory for secure process
  - **Domain Architecture**: Security domains mapped to process capabilities

- **IPC Security**:
  - Unix domain sockets with peer credential verification
  - Message authentication using HMAC with session-specific keys
    - HMAC-SHA256 for message integrity and authentication
    - Message-specific elements included in HMAC calculation (request ID, timestamp, operation)
    - Unique client nonce with each message to prevent replay attacks
    - Session keys never exposed outside secure boundary
  - Binary message format with strict schema validation
  - **Protocol Versioning**: IPC protocol designed for versioned extensions

### Android Implementation
- **Security Tiering**:
  - Runtime detection of security capabilities
  - Clear user notification of device security level
  - Enhanced software protections when hardware security unavailable
  - **Security Classification**: Device capabilities mapped to security tiers

- **Key Protection**:
  - Generate keys with `.setIsStrongBoxBacked(true)` when available
  - Apply `.setUserAuthenticationRequired(true)` for key operations
  - Implement `.setUnlockedDeviceRequired(true)` for added security
  - **Purpose Binding**: Keys bound to specific operations by purpose

### iOS Implementation
- **Secure Enclave Integration**:
  - Generate keys with kSecAttrTokenIDSecureEnclave attribute
  - Apply access control with kSecAccessControlUserPresence
  - Implement biometric policy with kSecAccessControlBiometryCurrentSet
  - **Capability Detection**: Security features detected and utilized at runtime

- **Screen Security**:
  - Implement screenshot prevention for sensitive screens
  - Secure UI paths to prevent overlay attacks
  - Biometric authentication directly linked to transactions
  - **Security Indicators**: Clear security status communication to users

## 9. Implementation Priority for MVP

1. Basic security isolation on desktop platforms
2. Simple IPC with request authentication
3. Core key management and threshold signing operations
4. Basic authorization model with user confirmation
5. Transaction construction and signing flow
6. Platform-specific secure storage adapters
7. **Extensible Architecture**: Core security model with defined extension points
8. **Capability Framework**: Security operations organized by capability with clear boundaries

## 10. Security Limitations in MVP

- Full HSM/hardware wallet integration deferred to post-MVP
- Complex multi-signature policies deferred to post-MVP
- Advanced threat mitigations (side-channel protections) minimized for MVP
- Browser platform has inherent security limitations that will be clearly communicated
- Duress wallet features postponed to post-MVP
- **Future L2 Integration**: Lightning and Liquid integration planned with appropriate security models

## 11. Development Workflow

- Implement and test core security process in isolation first
- Build mock interfaces for development/testing
- Create comprehensive test suite for boundary-crossing operations
- Develop security policy enforcement independently from UI logic
- **Capability Testing**: Validate security boundaries through capability-based testing
- **Security Verification**: Continuous testing of security isolation effectiveness