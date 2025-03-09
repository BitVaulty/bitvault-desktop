# BitVault Implementation Plan

This document outlines a concentrated implementation plan for delivering a working BitVault prototype ASAP. The plan focuses on establishing core security architecture and essential Bitcoin functionality while designing for future Lightning and Liquid integration.

## 1. Prototype Scope Definition

### Included Features (MVP)

- Linux desktop platform implementation only
- Basic 2-of-3 threshold signature wallet functionality
- Process isolation security boundary
- Core key management and storage
- Basic transaction construction and signing
- Simplified user authentication
- Essential UI for wallet setup and transactions
- Minimal backup and verification workflow
- **Extension Architecture**: Core abstractions and interfaces for future chain support

### Deferred Features (Post-Prototype)

- Additional platforms (Android priority, then iOS, macOS, Windows)
- Advanced security policies
- Hardware wallet integration
- Lightning Network integration
- Liquid Network support
- Advanced recovery mechanisms
- Enhanced privacy features
- Hardware signing device integration

## 2. Milestone Breakdown

### Week 1: Core Security Architecture
- **Security Boundary Implementation**
  - Basic process isolation on Linux only
  - Simple IPC channel with minimal authentication
  - Validate process separation works correctly
- **Project Structure**
  - Establish workspace with crates structure
  - Configure build system for Linux target
  - Set up basic testing framework

### Week 2: Key Management & Bitcoin Core
- **Basic Key Management**
  - Device key share generation and storage
  - Simple encryption for secure storage
  - Minimal key derivation for testing
- **Bitcoin Integration Foundation**
  - BDK integration for basic wallet operations
  - Address generation from public keys
  - Simple transaction construction (unsigned)

### Week 3: Transaction Flow & UI Basics
- **Transaction Signing**
  - Basic threshold signing with device key
  - Transaction validation within secure boundary
  - Signed transaction return to UI process
- **Minimal UI Implementation**
  - Wallet creation flows
  - Basic send/receive interface
  - Transaction history view

### Week 4: Essential User Workflows & Testing
- **Complete Critical Workflows**
  - End-to-end transaction testing
  - Key backup workflow
  - Basic recovery process
- **Security Validation**
  - Process isolation testing
  - IPC security verification
  - Penetration testing of security boundary
- **Documentation & Demo Preparation**
  - Usage documentation
  - Security model verification
  - Demo script and presentation

## 3. Feature Prioritization Matrix

### Must-Have (Critical Path)
These features represent the absolute minimum viable product:

1. **Security Boundary Core**
   - Process isolation on Linux
   - Basic IPC with authentication
   - Secure process for key operations
   
2. **Bitcoin Fundamentals**
   - Key generation and storage
   - Address derivation (P2WPKH only)
   - Transaction construction and signing
   - Basic transaction broadcasting

3. **Essential User Flows**
   - Wallet creation
   - Send Bitcoin transaction
   - Receive Bitcoin (address generation)
   - View transaction history

4. **Minimum Security Controls**
   - Password authentication
   - Basic key backup process
   - Transaction approval by user
   
### Should-Have (Important but not blocking)
Features that add significant value but could be simplified for MVP:

1. **Enhanced Security Policy**
   - Basic spending limits
   - Simple whitelisting capability
   
2. **Backup Verification**
   - Key share validation
   - Recovery testing mechanisms
   
3. **Transaction Features**
   - Fee estimation and selection
   - UTXO management controls
   
4. **UI Enhancements**
   - Security status indicators
   - Transaction details view
   - Address book functionality

### Could-Have (Desirable but deferrable)
Features that would be nice but can be deferred post-MVP:

1. **Advanced Security Features**
   - Rate limiting for operations
   - Advanced authentication options
   
2. **Extended Bitcoin Features**
   - Custom fee controls
   - Transaction labeling
   - Advanced coin selection
   
3. **UX Refinements**
   - Guided backup workflows
   - Enhanced recovery assistance
   - Transaction categorization

### Dependencies Map
Critical dependencies that must be addressed in sequence:

1. Security boundary must be implemented before key operations
2. Key generation must precede address derivation
3. Address generation must precede transaction construction
4. Basic UI framework must be in place before workflow implementation
5. Transaction construction must precede signing implementation

## 4. Platform Implementation Timeline

### Phase 1: Linux Desktop MVP (Month 1-2)
- **Security Features**: 
  - Process isolation with seccomp-BPF filters
  - HMAC authentication for IPC
  - Basic memory protection
  - Password-based authentication
- **Security Level**: Comprehensive with software-based protections
- **Limitations**: Reliance on OS security without hardware-backed features

### Phase 2: macOS and Windows Desktop (Month 3-4)
- **macOS Security Features**:
  - XPC Services for process isolation
  - Keychain integration for secure storage
  - Touch ID integration where available
  - Sandbox profiles
- **Windows Security Features**:
  - Named pipes with strict ACLs
  - DPAPI integration
  - Windows Hello integration where available
  - Job objects for process restriction
- **Security Parity**: Equivalent security guarantees across desktop platforms with platform-specific optimizations

### Phase 3: Android Platform (Month 5-6)
- **Security Tiers**:
  - Tier 1: StrongBox Keymaster with hardware security
  - Tier 2: TEE-backed keystore
  - Tier 3: Software-only implementation with enhanced protections
- **Security Features**:
  - Hardware-backed key storage where available
  - Biometric authentication integration
  - Security capability detection and adaptation
  - Enhanced software protection for lower-tier devices
- **Security Communication**: Clear indication of device security capabilities

### Phase 4: iOS Platform (Month 7-8)
- **Security Features**:
  - Secure Enclave integration for key protection
  - Keychain secure storage
  - Biometric (Face ID/Touch ID) authentication
  - App sandbox with security extensions
- **Security Level**: Consistent high security due to hardware standardization
- **Security Adaptations**: Optimization for Apple security ecosystem

### Phase 5: Web/WASM Implementation (Month 9+, Post-MVP)
- **Security Approach**:
  - Web Worker isolation for secure operations
  - Memory encryption for in-memory keys
  - Clear security limitations communicated to users
  - Enhanced client-side encryption
- **Security Level**: Basic with transparent limitations
- **Use Cases**: Focused on convenience and limited value storage

## 3. Cross-Platform Security Consistency

### Security Capability Framework
- **Capability Detection System**: Runtime detection of available security features
- **Security Classification Model**: Standardized tiering across all platforms
- **Security Status Communication**: Consistent indicators across platforms
- **Value Threshold Recommendations**: Platform-specific guidance based on security level

### Security Equivalence Mapping
- **Level A (Highest)**: Hardware-isolated key storage with biometric binding
  - iOS: Secure Enclave with Face ID/Touch ID
  - Android: StrongBox Keymaster with biometric
  - Desktop: External hardware security device
- **Level B (Strong)**: Hardware-backed security without full isolation
  - Android: Standard Keystore with TEE
  - macOS: Secure Enclave Macs with Touch ID
  - Windows: TPM-backed DPAPI with Windows Hello
- **Level C (Basic)**: Software-based security with enhanced protections
  - All platforms: Software implementation with memory protection
  - Linux/macOS/Windows: Process isolation with seccomp-BPF/sandbox
  - Mobile: OS sandbox with additional application protections
- **Level D (Limited)**: Environments with inherent security limitations
  - Web/WASM platform
  - Jailbroken/rooted devices (with explicit warnings)
  - Legacy systems without modern security features

### Feature Parity Timeline
- **Core Wallet Functionality**: Available on all supported platforms from their release
- **Key Management**: Platform-optimized but functionally equivalent
- **Transaction Operations**: Identical across all platforms
- **Security Policies**: Adaptive based on platform security level
- **Backup & Recovery**: Consistent with platform-specific optimizations
- **Network Capabilities**: Uniform across platforms
- **UI/UX**: Platform-appropriate but consistent mental model

## 4. Prototype Implementation Focus

### Development Environment Setup

### Required Development Tools

- Rust toolchain (stable channel)
- Cargo and essential development plugins:
  - `cargo-watch` for development workflow
  - `cargo-audit` for dependency security
  - `cargo-expand` for macro debugging
  - `cargo-flamegraph` for performance analysis
- Git for version control
- Linux development environment (Ubuntu/Debian recommended)
- Visual Studio Code or other Rust-friendly IDE
- Android SDK and NDK (for post-MVP Android development)
- **Bitcoin Core**: For local node integration testing 
- **Testing Libraries**: For security boundary validation

### Initial Repository Structure

```
bitvault/
├── Cargo.toml                # Workspace definition
├── README.md                 # Project overview
├── docs/                     # Documentation
├── bitvault-core/            # Security-critical module
│   ├── Cargo.toml
│   └── src/
├── bitvault-common/          # Shared components
│   ├── Cargo.toml
│   └── src/
├── bitvault-ui/              # User interface
│   ├── Cargo.toml
│   └── src/
├── bitvault-app/             # Platform integration
│   ├── Cargo.toml
│   └── src/
└── tests/                    # Integration tests
```

### Core Dependencies

- **Bitcoin Functionality**:
  - `bdk = "0.28.0"` - Bitcoin Development Kit
  - `bitcoin = "0.30.0"` - Bitcoin primitives
  
- **Cryptography**:
  - `ring = "0.16.20"` - Cryptographic primitives
  - `zeroize = "1.5.7"` - Secure memory wiping
  - `rand = "0.8.5"` - Secure random number generation
  - `getrandom = "0.2.8"` - Platform entropy gathering

- **Serialization**:
  - `serde = "1.0.152"` - Serialization framework
  - `serde_json = "1.0.93"` - JSON support
  - `bincode = "1.3.3"` - Binary encoding
  
- **UI Framework**:
  - `egui = "0.22.0"` - Immediate mode GUI
  - `eframe = "0.22.0"` - egui framework

- **Process Management**:
  - `tokio = { version = "1.25.0", features = ["full"] }` - Async runtime
  - Unix domain sockets for IPC (Linux)

## 3. Weekly Implementation Plan

### Week 1: Foundation and Security Boundary

**Goals**: Establish repository structure, implement basic process isolation, and create minimal IPC.

**Tasks**:

1. **Project Setup**:
   - Initialize Git repository
   - Configure Rust workspace
   - Set up CI pipeline (GitHub Actions)
   - Configure linting and formatting rules

2. **Security Boundary Implementation**:
   - Create process isolation mechanism
   - Implement basic IPC using Unix domain sockets
   - Define message serialization format with chain type identifiers
   - Build simple message passing examples
   - Design capability-based security model

3. **Common Types**:
   - Define core API interface types
   - Implement message validation
   - Create error handling patterns
   - Define security capability interfaces
   - Design extensible trait system for chain operations

**Deliverable**: Repository with process isolation and IPC working, demonstrating secure message passing between processes.

### Week 2: Core Bitcoin Functionality

**Goals**: Implement basic wallet operations and key management within secure boundary.

**Tasks**:

1. **Key Management**:
   - Implement secure key generation with robust entropy sources
   - Create key storage with encryption
   - Integrate with platform secure storage
   - Implement BIP39 seed phrase generation
   - Design key purpose abstraction for future extension

2. **BDK Integration**:
   - Implement custom signer for BDK
   - Create multisig descriptor handling
   - Set up address derivation
   - Implement basic transaction construction
   - Create abstraction layers around BDK interfaces

3. **Authentication**:
   - Create basic authentication system
   - Implement session management
   - Set up secure password handling
   - Basic policy enforcement framework
   - Design tiered authentication for future extension

**Deliverable**: Functional core module that can create keys, derive addresses, and handle basic wallet operations within the secure boundary.

### Week 3: UI and Application Integration

**Goals**: Build basic UI, connect it to core functionality, and implement transaction flows.

**Tasks**:

1. **User Interface**:
   - Set up egui application framework
   - Create wallet creation workflow
   - Implement transaction construction interface
   - Design basic navigation and layouts
   - Create security status visualization

2. **Core Integration**:
   - Connect UI to core process via IPC
   - Implement request/response handling
   - Create error handling and status indicators
   - Add security status visualization
   - Design chain type awareness in UI infrastructure

3. **Transaction Flow**:
   - Implement address generation and display
   - Create transaction builder interface
   - Set up PSBT creation and signing
   - Implement minimal transaction broadcasting
   - Design abstracted transaction interfaces

**Deliverable**: Working UI application that communicates with the secure core process, supporting wallet creation and basic transactions.

### Week 4: Testing, Refinement, and Documentation

**Goals**: Improve reliability, implement backup/recovery, and prepare for prototype demonstration.

**Tasks**:

1. **Testing and Hardening**:
   - Implement critical unit tests
   - Test security boundaries
   - Fix identified issues
   - Performance optimization where needed
   - Verify security boundary integrity

2. **Backup and Recovery**:
   - Implement basic backup procedure
   - Create minimal recovery mechanism
   - Test recovery scenarios
   - Add verification steps
   - Design extensible backup format

3. **Documentation and Polish**:
   - Update implementation documentation
   - Create user guide for prototype
   - Add developer onboarding instructions
   - Polish UI for demonstration
   - Prepare Android development foundation
   - Document extension points for future L2 integration

**Deliverable**: Working prototype suitable for demonstration, with basic testing, documentation, and essential functionality implemented.

## 4. Development Workflow

### Daily Development Cycle

1. Morning team sync (15 minutes)
2. Focus development blocks (2-3 hours)
3. Afternoon integration tests
4. End-of-day commit and push
5. Issue tracking updates

### Code Review Requirements

- All security-critical code requires review
- IPC and secure process code requires thorough review
- Test coverage for critical functionality
- Documentation updates with code changes
- Platform abstraction code receives extra attention
- Abstractions for future chain support require explicit review

### Testing Approach

- Critical security boundary tests
- Basic functionality tests for wallet operations
- IPC communication tests
- Mock tests for external dependencies
- Platform abstraction layer tests
- Capability-based security tests

## 5. Module-Specific Implementation Priorities

### bitvault-common Implementation Focus

1. **API Interfaces**:
   - Request/response structures
   - Error types and handling
   - Capability interfaces
   - Platform-agnostic abstractions
   - Chain-type identification system

2. **Serialization**:
   - Message format definitions
   - Binary serialization for IPC
   - Validation schemas
   - Extensible type system for future chain support

3. **Bitcoin Types**:
   - Address representations
   - Transaction formats
   - UTXO structures
   - Abstracted chain interfaces

### bitvault-core Implementation Focus

1. **Process Management**:
   - Child process model
   - IPC server implementation
   - Message handling loop
   - Platform security abstraction layer
   - Security domain architecture

2. **Key Operations**:
   - Key generation and storage
   - Secure key handling
   - BDK integration for signing
   - Key purpose system
   - Entropy validation and collection

3. **Security Features**:
   - Authentication verification
   - Basic policy enforcement
   - Secure memory handling
   - Capability-based security enforcement

### bitvault-ui Implementation Focus

1. **State Management**:
   - Wallet representation
   - Transaction state
   - User preferences
   - Chain-aware data models

2. **UI Components**:
   - Wallet setup wizard
   - Transaction builder
   - Address display
   - Security status
   - Responsive layouts (for eventual Android adaptation)
   - Chain-specific UI elements (hidden behind feature flags)

3. **Core API Client**:
   - IPC communication
   - Request formatting
   - Response handling
   - Error visualization
   - Chain type routing

### bitvault-app Implementation Focus

1. **Application Lifecycle**:
   - Process management
   - IPC channel setup
   - Shutdown handling
   - Platform detection
   - Security capability discovery

2. **Platform Integration**:
   - Linux-specific features
   - Secure storage integration
   - Permission management
   - Platform abstraction design (considering Android next)
   - Node connection framework

3. **Development Utilities**:
   - Logging framework
   - Debugging helpers
   - Development mode features
   - Security testing tools

## 6. Security Implementation Specifics

### Process Isolation Details

```rust
// Process spawning with restricted permissions
pub fn spawn_secure_process() -> Result<Child, Error> {
    Command::new(env::current_exe()?)
        .arg("--secure-mode")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Additional permission restrictions will be added
        .spawn()
        .map_err(Error::ProcessSpawn)
}
```

### IPC Communication Implementation

```rust
// Unix domain socket server (core process)
pub async fn start_ipc_server(socket_path: &Path) -> Result<(), Error> {
    let listener = UnixListener::bind(socket_path)?;
    
    while let Ok((stream, _addr)) = listener.accept().await {
        tokio::spawn(handle_client(stream));
    }
    
    Ok(())
}

// Client connection (UI process)
pub async fn connect_to_core() -> Result<UnixStream, Error> {
    let socket_path = get_socket_path()?;
    UnixStream::connect(socket_path).await.map_err(Error::IpcConnection)
}
```

### Key Management Implementation

```rust
// Key generation in secure context with robust entropy
pub fn generate_wallet_keys(network: Network) -> Result<WalletKeys, Error> {
    // Validate entropy source
    ensure_secure_entropy()?;
    
    // Generate three keys for 2-of-3 multisig
    let device_key = generate_key(KeyRole::Device)?;
    let backup_key = generate_key(KeyRole::Backup)?;
    let recovery_key = generate_key(KeyRole::Recovery)?;
    
    // Create wallet structure
    let wallet_keys = WalletKeys {
        device_key,
        backup_key,
        recovery_key,
        network,
        chain_type: ChainType::Bitcoin,
    };
    
    // Store keys securely
    store_keys(&wallet_keys)?;
    
    Ok(wallet_keys)
}
```

### Chain-Agnostic Interface Example

```rust
// Chain type enum for future extension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    Bitcoin,
    #[cfg(feature = "lightning")]
    Lightning,
    #[cfg(feature = "liquid")]
    Liquid,
}

// Transaction trait for abstraction
pub trait Transaction {
    fn chain_type(&self) -> ChainType;
    fn amount(&self) -> Amount;
    fn fee(&self) -> Option<Amount>;
    fn recipients(&self) -> Vec<Recipient>;
    fn serialize_for_signing(&self) -> Result<Vec<u8>, Error>;
    fn validate(&self) -> Result<(), ValidationError>;
}
```

## 7. Development Resources

### Key Documentation References

- [BDK Documentation](https://bitcoindevkit.org/docs/)
- [Rust Bitcoin Documentation](https://docs.rs/bitcoin/)
- [egui Documentation](https://docs.rs/egui/)
- [Tokio Documentation](https://docs.rs/tokio/)
- [Android NDK Documentation](https://developer.android.com/ndk)
- [Rust on Android Guide](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html)
- [LDK Documentation](https://lightningdevkit.org/)
- [Elements Project Documentation](https://elementsproject.org/elements-code-tutorial/overview)

### Development Environments

- **Local Development**: Linux development environment
- **Testing Environment**: Virtual machines for isolation testing
- **CI Environment**: GitHub Actions for automated testing
- **Android Development**: Android Studio with Rust NDK integration
- **Node Testing**: Bitcoin Core for integration testing

### Review and Support Resources

- Security review checklist
- Bitcoin operation verification guide
- UI component library
- IPC communication testing utilities
- Cross-platform abstraction review guide
- Chain abstraction verification tools

## 8. Post-Prototype Roadmap

### Phase 1: Hardening and Testing (Weeks 5-6)

- Security audit of prototype
- Comprehensive test suite development
- Bug fixes and stability improvements
- Performance optimization
- Prepare for Android development
- Begin framework for Lightning/Liquid integration

### Phase 2: Android Implementation (Weeks 7-10)

- Platform abstraction layer
- Android Keystore integration
- Secure process model for Android
- UI adaptation for mobile
- Android-specific security features
- Testing on various Android devices

### Phase 3: Security Policy Enhancement (Weeks 11-12)

- Advanced security policy implementation
- Transaction limits and controls
- Enhanced authentication options
- Security status monitoring
- Cross-platform security consistency
- Chain-specific policy extensions

### Phase 4: Lightning Network Integration (Weeks 13-16)

- LDK integration
- Channel management security model
- Lightning payment framework
- Extended backup for channel states
- Security boundary adaptation for hot wallet
- Lightning-specific UI components

### Phase 5: Liquid Integration (Weeks 17-20) 

- Liquid network support
- Confidential transaction handling
- Asset management framework
- Liquid-specific security policies
- Integrated multi-asset view
- L-BTC and issued asset support

## 9. Risk Management

### Development Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Security boundary implementation challenges | High | Start with simplified model, iterative enhancement |
| BDK integration complexity | Medium | Focus on core functions first, defer advanced features |
| IPC performance issues | Medium | Optimize message size, benchmark early |
| UI complexity delaying prototype | Medium | Use minimal UI initially, enhance later |
| Testing overhead | Low | Focus on critical security tests only for prototype |
| Android platform complexity | High | Begin research during prototype, prepare abstractions |
| Future chain integration challenges | High | Design clean abstractions and extension points from the start |

### Contingency Plans

1. **Security Boundary Simplification**: Fall back to simplified security model if full implementation exceeds timeframe
2. **Scope Reduction**: Further limit transaction types or wallet features if needed
3. **External Components**: Use existing libraries more extensively if custom implementation takes too long
4. **UI Simplification**: Reduce UI complexity if development time exceeds estimates
5. **Platform Focus**: Concentrate solely on Linux for prototype if cross-platform issues arise
6. **Android Deferral**: If Android complexity is excessive, create detailed plan but defer implementation
7. **Extension Point Documentation**: If implementing extension points adds complexity, focus on documentation of future design

## 10. Prototype Success Criteria

A successful prototype must demonstrate:

1. Functioning process isolation security boundary
2. Basic wallet creation with 2-of-3 multisig
3. Address generation and display
4. Transaction construction and signing
5. Successful transmission of signed transaction
6. Basic backup and recovery capability
7. Clear security status indicators
8. Acceptable performance for core operations
9. Platform abstraction design suitable for Android expansion
10. Clearly defined extension points for future Lightning and Liquid support

These criteria represent the minimum viable product features required to validate the core architecture and security model.

## 11. Android Implementation Planning

Even during the Linux prototype phase, we'll prepare for Android by:

1. **Research**:
   - Android security model and Keystore capabilities
   - Process isolation options on Android
   - UI adaptation requirements
   - Cross-compilation workflow setup

2. **Architecture**:
   - Identify platform abstraction points
   - Design security boundary model for Android
   - Plan IPC alternatives for Android
   - Evaluate performance implications

3. **Technical Spike**:
   - Simple proof-of-concept for Rust on Android
   - Test BDK functionality on Android
   - Evaluate egui performance on Android
   - Verify Keystore integration possibilities

4. **Risk Assessment**:
   - Device compatibility mapping
   - Security capability variance across Android versions
   - Performance benchmarks on target devices
   - Deployment and distribution considerations 

## 12. Lightning and Liquid Preparation

During the Bitcoin-focused MVP development, we'll prepare for future L2 integration by:

1. **Architectural Foundations**:
   - Abstract chain types in core data models
   - Design extensible transaction interfaces
   - Create pluggable validation systems
   - Implement capability-based security model

2. **Documentation**:
   - Document extension points for future integrations
   - Create architectural decision records for future capabilities
   - Define security models for different chain types
   - Map LDK and Elements integration points

3. **Test Framework**:
   - Design tests that validate extensibility
   - Create mock implementations of future chain types
   - Test IPC with chain type identifiers
   - Validate security boundary with capability tests

4. **UI Preparation**:
   - Design UI with expandable navigation
   - Create abstract asset representations
   - Build transaction interfaces that can adapt to different types
   - Implement chain-aware data models 