# BitVault Platform-Specific Security Capabilities

This document analyzes security capabilities across BitVault's target platforms, defining implementation strategies for consistent security boundaries.

## 1. Security Model Fundamentals

### Core Security Requirements

- Secure key share storage and management
- Protected threshold signature operations
- Isolated cryptographic operations
- User-authenticated sensitive actions
- Protection against malware and key theft

### Implementation Approach

Each platform requires different mechanisms to implement the same security model:

- **Security Boundaries**: How process or thread isolation is implemented
- **Secure Storage**: Where and how key shares are protected
- **Authentication**: How user intent is verified
- **Hardware Security**: Utilization of hardware security features
- **Attack Mitigations**: Platform-specific threat countermeasures

## 2. Platform Security Capability Overview

### Desktop (Linux/macOS/Windows)
- **Process Isolation**: Separate OS processes with clear boundary
- **Secure Storage**: OS-specific secure storage APIs with additional encryption
- **Memory Protection**: Address space isolation between processes plus in-memory encryption
- **Hardware Security**: Limited (external devices only)
- **Authentication**: Password-based with optional 2FA
- **Key Generation**: Software-based within secure process
- **Side-channel Protection**: Memory access pattern obfuscation, constant-time operations
- **Cold-boot Attack Resistance**: Memory encryption, aggressive session timeouts, key zeroization
- **Malware Resistance**: Moderate, vulnerable to privileged malware

### Android
- **Process Isolation**: App sandbox with potential service isolation
- **Secure Storage**: Keystore (hardware-backed when available)
- **Memory Protection**: Variable, dependent on device capabilities
- **Hardware Security**: StrongBox/TEE (highly device-dependent)
- **Authentication**: Biometric (fingerprint/face) with fallbacks
- **Key Generation**: Hardware-generated when available
- **Side-channel Protection**: Variable by device
- **Cold-boot Attack Resistance**: Variable by device
- **Malware Resistance**: Moderate, improved on newer devices

### iOS
- **Process Isolation**: App sandbox with XPC services
- **Secure Storage**: Secure Enclave plus Keychain
- **Memory Protection**: System-provided memory encryption
- **Hardware Security**: Secure Enclave (built into all modern devices)
- **Authentication**: Strong biometric (Face/TouchID)
- **Key Generation**: Hardware-generated in Secure Enclave
- **Side-channel Protection**: Strong hardware-based protection
- **Cold-boot Attack Resistance**: Strong
- **Malware Resistance**: Strong due to app review and platform security

## 3. Desktop Implementation (Primary MVP Focus)

### Security Architecture

- **Process Model**: Two separate OS processes
  - UI process: Full permissions, handles network and display
  - Secure process: Restricted permissions, handles keys and signing
- **Process Communications**: Platform-specific IPC
  - Linux: Unix domain sockets with peer credential verification
  - macOS: XPC services or Unix domain sockets with access controls
  - Windows: Named pipes with strict ACLs

### Linux-Specific Security Measures

- **Process Isolation Enhancement**:
  - **Seccomp-BPF Filtering**: Restrict system calls available to secure process
    - Implement allowlist approach for system calls
    - Only permit memory management, IPC, and essential operations
    - Block network, file creation, and external process operations
    - Apply different filter profiles based on operation context
    - Filter applied early in secure process initialization
    - Terminate process on filter violation for maximum security
  - **Namespaces**: Isolate the secure process using Linux namespaces
    - User namespace for privilege isolation
    - Mount namespace for filesystem view restrictions
    - Network namespace (with no interfaces) to prevent network access

- **IPC Authentication**:
  - **HMAC Authentication Protocol**: Secure all IPC communications
    - Session establishment with secure key exchange
    - All messages authenticated with HMAC-SHA256
    - Critical message components included in authentication
    - Nonce-based replay protection
    - Constant-time verification to prevent timing attacks
  - **Credential Verification**: Validate process identity using SO_PEERCRED

### Secure Storage Implementation

- **Linux**: 
  - Secret Service API or GNOME Keyring/KDE Wallet
  - Application-specific encryption layer with Argon2id key derivation
  - Access gated by user authentication
  - Storage verification through integrity checking
  - Fallback to encrypted file with integrity protection if secure storage APIs unavailable

- **macOS**: 
  - Keychain Services with kSecAttrAccessibleWhenUnlockedThisDeviceOnly protection
  - Additional application-level encryption
  - Touch ID integration where available
  - Keychain item ACLs with application restrictions

- **Windows**: 
  - DPAPI with additional application-specific encryption
  - Credential Manager for user-visible credentials
  - User account binding for key material
  - TPM integration where available

### Key Handling Model

- Keys generated within secure process
- Keys never exposed to UI process
- Keys encrypted at rest with user-provided secret
- Keys encrypted in memory when not in active use
- Session key derived at runtime for memory encryption
- Memory wiping using secure zeroization patterns

### Cold Boot Attack Mitigations

- **Memory Encryption**: All sensitive data encrypted in memory
- **Key Fragmentation**: Key material split across memory locations
- **Limited Exposure**: Decrypt key material only when needed
- **Aggressive Timeouts**: Short session validity with automatic wiping
- **Page Locking**: Prevent memory pages from being swapped to disk
- **Memory Guards**: Guard pages around sensitive memory regions

### Critical Security Limitations

1. **Limited Hardware Protection**: No built-in hardware security module
2. **OS-level Malware Vulnerability**: Susceptible to privileged malware
3. **Memory Exposure**: Keys potentially visible in process memory
4. **Cold Boot Risk**: Physical memory attacks possible
5. **Screen Capture Risk**: Transaction details visible on screen

### Essential Mitigations

1. **Memory Encryption**: Encrypt keys in memory with session key
2. **Key Zeroization**: Wipe keys from memory after use
3. **Short Timeouts**: Aggressive session expiration
4. **Process Monitoring**: Detect integrity violations
5. **Input Validation**: Strict validation at process boundaries
6. **Secure IPC**: Message authentication and encryption
7. **Minimal Dependencies**: Reduce attack surface in secure process

## 4. Android Implementation

### Security Architecture

- **Security Tiering**:
  - **Tier 1**: StrongBox Keymaster (hardware-isolated environment)
    - Generate keys with `.setIsStrongBoxBacked(true)`
    - Full hardware protection for key material
    - Hardware attestation verification
  - **Tier 2**: Trusted Execution Environment (separate secure processor)
    - Standard hardware-backed keystore
    - Key generation within TEE
  - **Tier 3**: Software-only implementation (for older devices)
    - Enhanced software protection with obfuscation
    - More frequent key rotation
    - Reduced transaction limits
- **Runtime Security Detection**: 
  - Detect available security features during initialization
  - Security level indicators for users
  - Adapt security parameters to device capabilities

### Secure Storage Implementation

- **Android Keystore**: 
  - Generate keys with `.setUserAuthenticationRequired(true)`
  - Apply `.setUnlockedDeviceRequired(true)` for added security
  - Set `.setInvalidatedByBiometricEnrollment(true)` for biometric binding
  - Use `.setUserConfirmationRequired(true)` for transaction signing
  - Hardware-backed when available

- **EncryptedSharedPreferences**: 
  - Store configuration and non-key sensitive data
  - Key derived from Keystore master key
  - Automatic encryption/decryption with data integrity

- **EncryptedFile**: 
  - For larger sensitive datasets
  - Keystore-backed encryption keys
  - Strong integrity protection

### Thread Isolation (when process isolation unavailable)

- Dedicated thread with minimal permissions for secure operations
- Use `SecureRandom` seeded from hardware RNG when available
- Memory barriers between threads handling sensitive data
- Clear thread local storage after sensitive operations
- Thread sanitization before and after key operations

### Threshold Signature Adaptation

- **Key Storage Tiering**:
  - Primary: Hardware-backed when available
  - Fallback: Software implementation with additional protections
- **Cross-device Signing**: Secure protocol between user devices
- **Security Status Indicators**: Clear display of current security level
- **Biometric Binding**: Direct biometric verification for signing operations

### Critical Security Limitations

1. **Device Fragmentation**: Widely variable security capabilities
2. **Hardware Security Inconsistency**: Not all devices offer hardware protection
3. **Root/Custom ROM Risks**: Compromised platform security
4. **OEM Customizations**: Unpredictable security behavior
5. **Malware Prevalence**: Higher exposure to malicious applications

### Essential Mitigations

1. **Tiered Security Model**: Adapt to device capabilities
2. **Hardware Attestation**: Verify genuine Android security components
3. **Root Detection**: Multiple detection methods with clear warnings
4. **API Targeting**: Use latest security APIs with fallbacks
5. **Enhanced Software Protection**: Additional measures on lower-security devices
6. **Screen Security**: FLAG_SECURE to prevent screenshots of sensitive data
7. **Overlay Detection**: Check for screen overlays during sensitive operations
8. **Accessibility Service Detection**: Warn when accessibility services active

## 5. iOS Implementation

### Security Architecture

- **Process Model**: App sandbox with optional extensions
- **Privilege Separation**: 
  - Main app: UI, networking, transaction construction
  - Extensions (optional): Isolated functionality
- **Secure Enclave Integration**:
  - Generate keys with kSecAttrTokenIDSecureEnclave attribute
  - Apply access control with kSecAccessControlUserPresence
  - Implement biometric policy with kSecAccessControlBiometryCurrentSet
  - Set kSecAccessControlPrivateKeyUsage for signing operations

### Secure Storage Implementation

- **Keychain Security Classes**:
  - High-security keys: `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`
  - Session data: `kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`
- **Secure Enclave Keys**: 
  - Generated and stored in hardware
  - Never exposed to application memory
  - Direct signing operations when compatible with threshold scheme
- **Access Controls**: 
  - Biometric (Face ID/Touch ID) authentication for key operations
  - `LAPolicy.deviceOwnerAuthenticationWithBiometrics` for security operations

### Memory Protection

- Automatic memory management with Swift/Rust interaction
- Implementation of secure value types for sensitive data
- Explicit memory wiping after cryptographic operations
- Apply SecureEnclave operations directly where supported
- Memory encryption for threshold signature components

### Threshold Signature Implementation

- **Device Key Share**: Generated and stored in Secure Enclave
- **Backup/Recovery Key Shares**: Options for:
  - Secondary iOS device with Secure Enclave
  - iCloud Keychain (with appropriate security warnings)
  - Manual backup (with clear security instructions)
- **Signing Protocol**:
  - Secure multi-party computation for signature generation
  - Biometric verification for signing authorization
  - Direct Secure Enclave operations where possible

### Critical Security Limitations

1. **Limited Cryptography in Secure Enclave**: Not all operations supported
2. **Apple Ecosystem Lock-in**: Security tied to Apple infrastructure
3. **iOS Vulnerability Exposure**: Subject to platform-level issues
4. **Jailbreak Risks**: Compromised on jailbroken devices
5. **Backup Security**: iCloud backup security considerations

### Essential Mitigations

1. **Capability Detection**: Verify Secure Enclave functionality
2. **Security Downgrade Warnings**: Clear notification if reduced security
3. **Jailbreak Detection**: Multiple detection methods with clear warnings
4. **Secure UI**: Protection against overlay attacks
5. **Local Authentication Context**: Tie operations to recent authentication
6. **Memory Protection**: Sensitive data handling techniques
7. **Code Protection**: Anti-debugging and code integrity validation

## 6. Cross-Platform Security Implementation

### Security Capability Abstraction

- **Core Interface**: Define abstract security interfaces for all critical operations
- **Platform Adapters**: Implement platform-specific security mechanisms
- **Capability Registry**: Runtime detection and registration of available security features
- **Graceful Degradation**: Fallback mechanisms with appropriate security warnings

### Authentication Framework

- **Wallet Access**:
  - Desktop: Password with optional 2FA
  - Android: Password plus biometric when available
  - iOS: Password plus biometric (Face ID/Touch ID)

- **Transaction Signing**:
  - Desktop: Explicit confirmation with password
  - Android: Biometric confirmation where available
  - iOS: Biometric confirmation

- **Key Export**:
  - Desktop: Password plus secondary factor
  - Android: Password plus biometric
  - iOS: Password plus biometric

- **Policy Changes**:
  - Desktop: Password plus cooling period
  - Android: Password, biometric, and cooling period
  - iOS: Password, biometric, and cooling period

### Threshold Signature Implementation Strategy

- **Key Types and Distribution**:
  - **Device Key Share**: Platform-secured on primary device
  - **Backup Key Share**: Secondary device or secured backup medium
  - **Recovery Key Share**: Cold storage or trusted third party

- **Cross-Platform Signing Coordination**:
  - Secure protocol for threshold signature generation
  - QR code transmission for airgapped operations
  - Secure file transfer between user devices
  - Clear signing status indicators

### Unified Security Policy Engine

- Platform-agnostic rules engine
- Platform-specific enforcement mechanisms
- Consistent security guarantees regardless of platform
- Transparent security status for users

## 7. Implementation Priorities and Roadmap

### Phase 1: Desktop MVP (Q1)

1. **Security Boundary Implementation**: 
   - Process isolation architecture
   - IPC mechanisms with authentication
   - Memory protection for sensitive data

2. **Core Key Operations**:
   - Secure key generation and storage
   - Threshold signature implementation
   - Basic authentication framework

3. **Threshold Signature Foundation**:
   - 2-of-3 key share management structure
   - Secure signing protocol
   - Cross-device signing preparation

### Phase 2: Android Implementation (Q2)

1. **Security Capability Detection**:
   - Hardware security module identification
   - Security level determination
   - Adaptive security implementation

2. **Keystore Integration**:
   - Optimized key storage per device
   - Biometric binding where available
   - Security status transparency

3. **Android-Specific Hardening**:
   - Root detection measures
   - Enhanced software protections
   - Security guidance for varied devices

### Phase 3: iOS Implementation (Q3)

1. **Secure Enclave Integration**:
   - Hardware key generation and storage
   - Biometric authentication
   - Signing operation protection

2. **iOS-Specific Security**:
   - Keychain security configuration
   - Local Authentication framework
   - App security hardening

3. **Extended Threshold Signature Support**:
   - Device-to-device key coordination
   - Backup key management
   - Recovery procedures

## 8. Security Validation Requirements

### Cross-Platform Testing Framework

- Standardized security test suite across platforms
- Boundary penetration testing
- Authentication bypass attempts
- Memory analysis for key exposure
- IPC message integrity and confidentiality

### Platform-Specific Testing

- **Desktop**: Process isolation effectiveness, memory protection
- **Android**: Security level verification, hardware attestation testing
- **iOS**: Secure Enclave operation verification, jailbreak resistance

### User-Facing Security Verification

- Security status indicators
- Key backup verification
- Recovery procedure testing
- Security policy effectiveness verification

## 9. Trade-Offs and Decisions

### Security vs. Usability

- Aggressive timeouts enhance security but impact usability
- Hardware security improves protection but limits flexibility
- Multiple authentication factors increase security but add friction

### Platform-Specific Considerations

- Desktop prioritizes flexibility but requires more software security
- Android provides adaptability but needs rigorous security verification
- iOS offers strong hardware security but less customization

### Implementation Guidance

- Begin with highest available security on each platform
- Provide clear opt-out warnings when users choose convenience over security
- Document security model and limitations transparently
- Evolve security implementation as platform capabilities advance

## 10. Security Capability Detection and Adaptation Framework

### Capability Detection Architecture

#### Core Principles
- **Runtime Detection**: Detect available security features during application initialization
- **Progressive Enhancement**: Utilize best available security capabilities on each platform
- **Graceful Degradation**: Provide enhanced software protection when hardware features unavailable
- **Transparent Communication**: Clearly communicate actual security level to users
- **Consistent Security Interface**: Present uniform security interfaces regardless of platform

#### Implementation Strategy

1. **Platform Security Registry**
   - Catalog of security capabilities across supported platforms
   - Standardized capability definitions and requirements
   - Versioned capability specifications for platform evolution
   - Test verification methods for each capability
   - Fallback implementation guidance

2. **Detection Mechanism**
   - Secure boot-time capability probing with verification
   - Multi-phase detection with increasing specificity
   - Detection result caching with integrity protection
   - Attestation verification where available
   - Tamper-resistant capability reporting

3. **Security Classification Engine**
   - Standard security level determination across platforms
   - Mapping platform-specific features to standard security tiers
   - Multi-dimensional security assessment (key storage, authentication, isolation)
   - Overall security rating with component-specific details
   - Clear capability publication to UI

### Security Level Taxonomy

#### Level A (Hardware-Secured)
- **Requirements**:
  - Hardware-isolated key storage (SE/TEE/HSM)
  - Hardware-backed attestation
  - Strong biometric or multi-factor authentication
  - Physical tamper resistance
  - Side-channel attack mitigations

- **Platform Examples**:
  - iOS: Secure Enclave with Face ID/Touch ID
  - Android: StrongBox Keymaster with biometric
  - Desktop: External hardware security device
  - Linux: TPM with hardware token authentication
  - macOS: T2/M1 Secure Enclave with Touch ID

- **Security Guarantees**:
  - Keys never exposed to application processor
  - Hardware-enforced rate limiting for authentication
  - Physical attack resistance
  - Strong protection against malware

#### Level B (Hardware-Backed)
- **Requirements**:
  - Hardware-backed key storage without full isolation
  - Some form of attestation or verification
  - Biometric or strong authentication
  - Software-assisted tamper resistance
  - Basic side-channel protections

- **Platform Examples**:
  - Android: Standard Keystore with TEE
  - Windows: TPM-backed DPAPI with Windows Hello
  - macOS: Keychain with secure storage
  - Linux: TPM-backed storage

- **Security Guarantees**:
  - Keys protected by hardware for many operations
  - Some protection against sophisticated malware
  - Moderate protection against physical attacks
  - Strong protection against remote attacks

#### Level C (Software-Enhanced)
- **Requirements**:
  - Enhanced software protection for key material
  - Process isolation with security boundaries
  - Strong authentication mechanisms
  - Memory encryption and protection
  - Active integrity monitoring

- **Platform Examples**:
  - All desktop platforms with process isolation
  - Android without hardware security modules
  - Older iOS devices without Secure Enclave
  - Linux without TPM but with seccomp-BPF

- **Security Guarantees**:
  - Protection against user-level malware
  - Moderate protection against privileged attackers
  - Basic resistance to memory examination
  - Protection against casual physical access

#### Level D (Basic)
- **Requirements**:
  - Standard software security practices
  - Basic key encryption at rest
  - User authentication with reasonable strength
  - Standard platform protections
  - Clear security limitations

- **Platform Examples**:
  - Web/WASM implementation
  - Legacy operating systems
  - Development/testing environments
  - Jailbroken/rooted devices (with warnings)

- **Security Guarantees**:
  - Basic protection against casual attacks
  - Limited resistance to targeted attacks
  - Protection primarily through obscurity and encryption
  - Clearly communicated security limitations

### Adaptation Strategies

#### Hardware-Secured Implementation (Level A)
- Direct use of hardware security features
- Minimal software security layers
- Attestation verification for critical operations
- Hardware-bound authentication
- Platform-specific optimizations:
  - iOS: Direct Secure Enclave operations
  - Android: StrongBox direct operations
  - Desktop: HSM/Smart card integration

#### Hardware-Backed Implementation (Level B)
- Hardware-backed key storage with software protection layers
- Additional memory protection for sensitive operations
- Enhanced authentication with hardware binding
- Active integrity verification
- Platform-specific adaptations:
  - Android: Keystore with additional protections
  - Windows: TPM + additional software security
  - macOS: Keychain with enhanced access controls

#### Software-Enhanced Implementation (Level C)
- Comprehensive software security mitigations:
  - Memory encryption for all sensitive data
  - Process isolation with strict boundaries
  - Constant-time cryptographic implementations
  - Anti-debugging and integrity verification
  - Enhanced key derivation (Argon2id with high parameters)
  - Multiple layers of encryption for key material
  - Aggressive session timeouts and memory wiping

#### Basic Implementation (Level D)
- Maximum software protection given limitations:
  - Multi-layered encryption for sensitive data
  - Obfuscation of key handling operations
  - Reduced functionality for high-value operations
  - Strict value limits with clear user warnings
  - Enhanced user verification for sensitive actions
  - Minimal trust assumptions about environment

### Security Level Communication

#### User Interface Elements
- Security indicator in main wallet interface
- Level-specific security badges with clear visualization
- Capability-specific detail views on demand
- Educational content on security level meaning
- Contextual security guidance based on detected level

#### Operation-Specific Communication
- Per-operation security assessment based on capability requirements
- Clear warning when operation exceeds security level recommendations
- Suggested alternatives for higher-security operations
- Immediate feedback on authentication strength
- Transparent communication of actual vs. desired security

#### Value-Based Recommendations
- Recommended maximum wallet balances based on security level
- Transaction limit recommendations tied to security capabilities
- Clear guidance on appropriate usage for each security tier
- Balance between security communication and user experience
- Non-alarmist but transparent security guidance

### Implementation Priority

1. **MVP Implementation (Linux Desktop)**:
   - Basic capability detection framework
   - Process isolation with seccomp-BPF detection
   - Memory protection capability validation
   - Simple security level indicator
   - Essential security guidance

2. **Cross-Platform Expansion**:
   - Platform-specific capability detection adapters
   - Standardized security level determination
   - Enhanced user communication system
   - Comprehensive security guidance
   - Automated testing of security capabilities

## 11. Trade-Offs and Decisions

### Security vs. Usability

- Aggressive timeouts enhance security but impact usability
- Hardware security improves protection but limits flexibility
- Multiple authentication factors increase security but add friction

### Platform-Specific Considerations

- Desktop prioritizes flexibility but requires more software security
- Android provides adaptability but needs rigorous security verification
- iOS offers strong hardware security but less customization

### Implementation Guidance

- Begin with highest available security on each platform
- Provide clear opt-out warnings when users choose convenience over security
- Document security model and limitations transparently
- Evolve security implementation as platform capabilities advance
