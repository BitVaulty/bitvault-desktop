# BitVault Bitcoin Implementation Strategy

This document outlines BitVault's Bitcoin implementation approach, with specific focus on the 2-of-3 threshold signature scheme (TSS) capabilities for MVP. The design incorporates future-proofing considerations for later Lightning Network and Liquid integration.

## 1. Threshold Signature Architecture

### Core Design Principles

- **True 2-of-3 Security**: Any two key shares from three distinct participants can authorize transactions
- **Distributed Key Control**: Key shares distributed across different security domains
- **Private Key Security**: No single entity possesses the complete private key
- **Transparent Verification**: All threshold signing operations independently verifiable
- **Extensible Architecture**: Core signing architecture designed to support future L2 solutions
- **Chain-Agnostic Foundation**: Security domains and boundaries designed for multiple chain types

### Key Share Distribution Strategy

- **Device Key Share**: Stored on user's primary device (within secure boundary)
- **Backup Key Share**: Stored on secondary device or secure backup medium
- **Recovery Key Share**: Cold storage or trusted third-party (service, trusted contact)
- **Future L2 Adaptation**: Key architecture allows for future hot wallet components with appropriate security boundaries

#### Key Share Generation Protocol

- **Cryptographic Approach**: Feldman's Verifiable Secret Sharing (VSS) variant optimized for ECDSA
  - Information-theoretically secure key share distribution
  - Verification mechanisms to detect malicious behavior
  - Minimal trust assumptions during generation
  - Provable security properties and share validation

- **Generation Process**:
  1. Master entropy generation within secure boundary
     - Hardware-based entropy where available
     - Multiple entropy sources combined securely
     - CSPRNG with continual reseeding
  
  2. Key share derivation with verification
     - Each share includes verification values
     - Shares independently verifiable against public key
     - Share validation before backup confirmation
     - Share-specific metadata for identification
  
  3. Share encryption and protection
     - Platform-specific secure storage for device share
     - Strong encryption for backup/recovery shares (XChaCha20-Poly1305)
     - Authentication requirements for share access
     - Hardware binding where platform security allows

- **Security Enhancements**:
  - Zero-knowledge proofs of share validity
  - Tamper-evident share format with version control
  - Cryptographic binding to user identity (optional)
  - Forward secrecy considerations for share updates

#### Key Share Storage Architecture

- **Device Key Share**:
  - Memory encryption when in use (memory-hardened AES-256-GCM)
  - Hardware-backed storage where available:
    - Linux: Encrypted with user authentication
    - macOS: Secure Enclave if available, Keychain with access controls
    - iOS: Secure Enclave with biometric binding
    - Android: StrongBox/Keystore with authentication binding
  - Secure process isolation with stringent access controls
  - Automatic zeroization after timeout periods
  - Never exposed to UI process or network

- **Backup Key Share**:
  - Secondary device: Same protections as primary device
  - Hardware security device: Device-specific secure storage
  - Encrypted backup file:
    - Password-derived key using Argon2id (resource-intensive parameters)
    - XChaCha20-Poly1305 authenticated encryption
    - Share-specific salt and encryption parameters
    - Forward-compatible serialization format
  - Paper backup option:
    - BIP39 mnemonic encoding for compatibility
    - Additional checksum words for integrity verification
    - Clear recovery instructions included
    - Guidance for physical security measures

- **Recovery Key Share**:
  - Cold storage optimized format:
    - Encrypted digital storage (offline)
    - Paper backup with error-correction codes
    - Metal storage option for disaster resistance
  - Optional trusted party storage:
    - Legal arrangements for access conditions
    - Time-locked recovery mechanisms
    - Encrypted with recovery-specific protections
  - Inheritance planning integration (future):
    - Legally compliant access mechanisms
    - Dead-man switch capabilities
    - Multi-party recovery workflows

#### Cross-Device Key Share Management

- **Secure Device Registration**:
  - Mutual device authentication protocol
  - Remote attestation where platform supports
  - Authentication binding to prevent unauthorized devices
  - Device fingerprinting with tampering detection

- **Secure Channel Establishment**:
  - Ephemeral key exchange with perfect forward secrecy
  - Noise Protocol Framework-based secure channel
  - Multiple authentication factors for critical operations
  - Channel binding to operation context

- **Key Share Synchronization**:
  - Public key information synchronization only
  - No private key material transmission after generation
  - Threshold signing protocol for cross-device operations
  - Signature verification on all participating devices

- **Key Share Rotation**:
  - Periodic or on-demand key share rotation
  - Versioned key shares with clear update paths
  - Secure invalidation of previous shares
  - Audit log of share rotation operations

#### Key Derivation and Management

- **HD Wallet Structure**:
  - BIP32/44/49/84 compatibility for address derivation
  - Extended capabilities for share-specific derivation
  - Future-proof derivation paths with versioning
  - Clear path separation for different asset types

- **Address Generation**:
  - Cooperative public key derivation across shares
  - Threshold-derived addresses using standard formats
  - Native SegWit (P2WPKH) as default address type
  - Taproot (P2TR) support for enhanced privacy (future)

- **Metadata Management**:
  - Secure tracking of derived addresses
  - Versioned key share metadata
  - Cryptographic binding of metadata to shares
  - Cross-device synchronization of public information

#### Recovery and Backup Verification

- **Share Recovery Protocol**:
  - Any valid 2-of-3 share combination supported
  - Share validation before recovery attempt
  - New share generation to replace missing/compromised share
  - Original wallet structure preservation

- **Backup Verification**:
  - Zero-knowledge proof of share possession
  - Test decryption of challenge data
  - Signature verification with partial shares
  - Interactive verification protocol for paper backups

- **Emergency Recovery Options**:
  - Pre-signed recovery transactions for extreme situations
  - Time-locked recovery mechanisms
  - Social recovery options (future enhancement)
  - Clear documentation and guidance for each recovery method

### Implementation Approach

- Threshold signature scheme (TSS) rather than on-chain Bitcoin Script multisig
- Single-signature transaction on-chain (appears as P2WPKH or P2TR)
- More efficient and private than traditional on-chain multisig (P2WSH)
- Signature created through secure multi-party computation without revealing key shares
- Compatible with future Taproot enhancements
- **Extension Model**: Protocol designed with message type identifiers for future chain-specific operations

## 2. Key Management

### Key Derivation

- BIP32 Hierarchical Deterministic architecture
- BIP44 derivation paths with Bitcoin-specific coin type
- **Path Structure**: Derivation path architecture designed to accommodate future chain types
- Secret sharing for threshold signature scheme
- Separate key material for each of the three signing participants
- **Extensible Key Purpose System**: Architecture supports future expansion to blinding keys or channel operations

### Key Share Security Distribution

- **Device Key Share**: Generated and stored within secure boundary of primary device
- **Backup Key Share**: Generated within secure boundary, exportable with explicit user action
- **Recovery Key Share**: Generated within secure boundary, mandatory export for cold storage
- **Security Tiering**: Key usage system designed to distinguish between cold storage and future hot wallet operations

### Key Rotation and Recovery

- Support for key rotation without changing wallet addresses
- Versioned key material for tracking which key shares signed which transactions
- Recovery processes requiring only 2 of 3 key shares
- **State Recovery Framework**: Architecture supports future extension to channel state recovery

## 3. Address Management

### Address Generation

- Standard Bitcoin address derivation using BDK
- Progressive address generation with gap limit management
- Address metadata tracking for user labels and usage
- **Chain Type Identifiers**: Address systems include chain type metadata for future extensibility

### Address Types

- Primary: P2WPKH (native SegWit single-signature)
- Optional: P2TR (Taproot) for enhanced privacy (post-MVP)
- **Future Compatibility**: Address system designed for extension to Liquid confidential addresses

### Address Derivation

- Standard BIP44 account and address indices
- Change and receive address chains
- Address gap limit with dynamic expansion
- **Metadata Extension Points**: Address metadata system designed to accommodate chain-specific attributes

## 4. Transaction Handling

### Transaction Construction

- **Architecture**: Abstraction-based design with clear separation between construction and signing
- **Location**: UI process constructs unsigned transactions using BDK
- **Components**:
  - Input selection optimized for privacy and fee efficiency
  - Change address management
  - Fee calculation with multiple rate options
  - Transaction metadata with extension points for chain-specific data
  - **Chain-Agnostic Interfaces**: Core transaction types designed as traits with extension capabilities
  
### Signing Workflow

1. UI process constructs transaction with chain-type identifier
2. User reviews and approves transaction details
3. Signing request sent to secure process with transaction details
4. Secure process validates transaction against security policies
5. Threshold signing protocol initiated with first key share
6. For second key share:
   - If backup key share accessible: signing protocol continued with backup device
   - If recovery key share needed: secure signing protocol with recovery key share
7. Threshold signature generated from participant key shares
8. Fully signed transaction returned to UI for broadcast

#### Threshold Signing Protocol Specifications

- **Cryptographic Protocol**: MuSig2 (Schnorr-based multi-signature scheme)
  - Two-round signing protocol for efficiency
  - Provable security in the plain public-key model
  - Compatible with Bitcoin Schnorr signatures (BIP340)
  - Supports future Taproot transaction types
  - Minimal communication overhead between signers

- **Protocol Phases**:
  1. **Initialization Phase**:
     - Transaction validation on all participating devices
     - PSBT (BIP174) format for transaction exchange
     - Independent fee and output verification
     - Secure communication channel establishment
     - Session-specific signing parameters

  2. **Round 1: Nonce Generation and Exchange**:
     - Each device generates secure nonces:
       - Multiple deterministic nonces per RFC6979 with additional entropy
       - Side-channel resistant generation process
       - Nonce commitment with zero-knowledge proof of correctness
     - Nonce exchange via secure channel
     - Verification of received nonce commitments

  3. **Round 2: Partial Signature Generation**:
     - Each device creates partial signature using its key share
     - Signature generated within secure boundary
     - Verification that partial signature uses correct nonce
     - Side-channel protected signing operation
     - Zero-knowledge proof of signature validity

  4. **Signature Aggregation**:
     - Partial signatures combined into single signature
     - Verification of final signature against Bitcoin consensus rules
     - Final signature indistinguishable from single-key signature
     - Signature serialized in standard Bitcoin format

- **Security Properties**:
  - Forward secrecy for signing sessions
  - Resistant to rogue-key attacks
  - Protection against nonce reuse
  - Resistant to related-key attacks
  - Side-channel mitigation in implementation
  - Session isolation from other operations

#### Cross-Device Communication Security

- **Secure Channel Protocol**: Noise_XX_25519_ChaChaPoly_SHA256
  - Mutual authentication of devices
  - Perfect forward secrecy for all communications
  - Strong encryption with authenticated payloads
  - Resistant to MITM attacks
  - Session binding to specific signing operation

- **Transport Options**:
  1. **Direct Device Connection**:
     - Bluetooth Low Energy with application-layer security
     - Local WiFi with TLS 1.3 and certificate pinning
     - USB connection with application security layer
     - NFC for compatible devices with secure channel

  2. **QR Code Transport** (for air-gapped devices):
     - Encrypted payload fragmented across multiple QR codes
     - Session keys derived from shared secret
     - Error correction and sequence verification
     - Partial state recovery for interrupted scans

  3. **Manual Entry Recovery Path**:
     - Minimal recovery code format for emergency use
     - Error-detecting input format
     - Progressive verification during entry
     - Fallback for all other methods

- **Operation Security**:
  - Explicit user confirmation on all participating devices
  - Clear display of transaction details for verification
  - Identical transaction representation across devices
  - Secure abort capability at any stage
  - Timeout handling with clean state reset

### Test Vectors

- Comprehensive test vectors for threshold signature verification
- Verification against Bitcoin standard test vectors
- Cross-implementation testing with other TSS libraries
- Signature compatibility testing across platforms
- **Test Framework**: Designed to support different signature schemes for future chain types

## 5. UTXO Management

### UTXO Selection Strategy

- Privacy-preserving coin selection algorithm
- Fee optimization based on UTXO age and value
- Consolidation strategies for managing UTXO fragmentation
- Coin control features for advanced users
- **Abstracted State Management**: UTXO tracking designed with abstractions to support different asset types

### UTXO State Tracking

- UTXO source and history metadata
- Confirmation status monitoring
- Spent/unspent state management
- Address reuse prevention
- **State Synchronization Framework**: Designed to accommodate future channel states and Liquid assets

## 6. BDK Integration

### Component Distribution

- **UI Process**:
  - Wallet database (public information only)
  - UTXO tracking and selection
  - Transaction construction
  - Network communication
  - Transaction preparation
  
- **Secure Process**:
  - Key share generation and storage
  - Threshold signing operations
  - Security policy enforcement
  - Signature verification
  
- **Extension Strategy**:
  - BDK interfaces wrapped with local abstractions for future extension
  - Clear separation between key management and transaction operations
  - Capability-based security boundaries for different operation types
  - Pluggable validation logic for different transaction types

### BDK Customization

- Custom signer implementation for threshold signature integration
- Database adapter for cross-boundary storage
- Network layer interface for blockchain communication
- Fee estimation services for transaction construction
- **Abstraction Layers**: Custom interfaces around BDK to facilitate future L2 integration

## 7. Threshold Signature-Specific Security Measures

### Signing Protocol Security

- Verification that signatures come from authorized key shares
- Protection against key share compromise
- Key share verification before signing
- Secure multi-party computation protocol for signature generation
- **Protocol Extensibility**: Designed to accommodate different signature types

### Recovery Procedures

- Emergency spending with any valid key share combination
- Timelocked recovery transactions for backup
- Key share replacement procedures after compromise
- **Recovery Framework**: Designed to accommodate different recovery types, including future channel states

### Transaction Verification

- Independent script validation before signing
- Amount and destination verification
- Fee reasonability checks
- Change address validation
- **Pluggable Validators**: Verification systems designed as pluggable modules for different transaction types

## 8. Backup and Recovery

### Backup Components

- Public key and address information
- Encrypted key shares for each threshold participant
- Recovery instructions with key share usage procedures
- Emergency recovery transactions
- **Versioned Backup Format**: Designed to support future extension for additional state data

### Recovery Methods

- Standard recovery with 2 of 3 key shares present
- Watch-only wallet recovery with public information
- Transaction history reconstruction from blockchain
- **Extensible Recovery Types**: Architecture supports different recovery scenarios

### Verification Procedures

- Backup verification workflows
- Test recovery procedures
- Key share integrity verification
- **Modular Verification System**: Designed to support verification of different data types

## 9. Network Interaction

### Node Connection Strategy

- Optional full node connection
- Multiple public electrum server fallbacks
- Tor support for enhanced privacy
- Blockchain data validation
- **Abstracted Node Interfaces**: Connection architecture designed to support different node types in the future

### Transaction Broadcasting

- Configurable broadcast delay
- Broadcast to multiple nodes
- Double-spend protection
- Transaction status monitoring
- **Chain-Specific Routing**: Broadcasting system includes chain type awareness for future extensions

## 10. Security Policy Engine

### Policy Framework

- Rule-based security policy system with clear extension points
- Policy enforcement isolated within secure boundary
- Tiered authentication with different security levels
- Capability-based permission model with extensible permissions
- **Chain-Aware Design**: Policy architecture accommodates different transaction types

### Authentication Integration

- Session management designed for different security contexts
- Biometric integration with extensible approval flows
- Clear security status communication across different operations
- **Flexible Authentication Levels**: Authentication framework supports different security requirements

## 11. Implementation Priorities for MVP

1. Basic 2-of-3 threshold signature implementation
2. Key share generation and secure storage within security boundaries
3. Secure signing protocol across devices
4. Transaction construction and validation
5. Essential backup and recovery procedures
6. Simple coin selection and UTXO management
7. P2WPKH address generation and validation
8. **Extension Points**: Core architecture with defined extension points for future L2 support

## 12. Future Enhancements (Post-MVP)

- Taproot signature support
- Advanced coin selection algorithms
- Lightning Network integration through LDK
- Liquid Network integration
- Hardware wallet support as optional signing participant
- Timelocked recovery mechanisms