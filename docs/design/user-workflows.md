# BitVault User Workflows

This document defines essential user workflows for BitVault, validating that our security model and Bitcoin implementation support real-world wallet operations.

## 1. Wallet Creation and Setup

### Initial Setup Process

1. **Application Initialization**
   - Security capability detection on user's platform
   - Security status and capabilities displayed
   - Secure process launched with appropriate isolation

2. **Wallet Creation**
   - User selects 2-of-3 threshold signature configuration (MVP default)
   - Strong password creation and verification
   - Clear explanation of the threshold signature security model

3. **Key Share Generation**
   - Device Key Share generated within secure boundary
   - Backup Key Share setup with guided process
   - Recovery Key Share creation with storage instructions
   - All public key information combined into wallet structure

4. **Security Configuration**
   - Basic spending limits configuration
   - Session timeout settings
   - Transaction approval requirements
   - Platform-specific security optimizations

5. **Backup Verification**
   - Verification challenges for backup material
   - Recovery key share storage attestation
   - Backup completion confirmation

### Security Boundary Interactions
- Key share generation occurs exclusively within secure process
- Only public keys and wallet structure cross the boundary
- Security policy stored and enforced within secure boundary
- Authentication session established for ongoing operations

## 2. Key Management and Backup

### Device Key Share Management

1. **Generation and Storage**
   - Created within platform security boundary
   - Encrypted at rest with user credentials
   - Never exposed to UI process
   - Bound to hardware security where available

2. **Usage Pattern**
   - Unlocked only with user authentication
   - Active only during authenticated sessions
   - Automatic locking after timeout or on demand

### Backup Key Share Configuration

1. **Secondary Device Option**
   - Key share generation on separate device
   - Secure public key exchange between devices
   - Mutual verification protocol
   - Independent security contexts maintained

2. **Physical Backup Option**
   - Key share material displayed for recording
   - BIP39 seed phrase with clear instructions
   - Verification quiz to confirm proper backup
   - Usage instructions for recovery scenarios

### Recovery Key Share Protection

1. **Cold Storage Approach**
   - Key share generated within secure boundary
   - Physical recording in secure format
   - Storage location recommendations
   - Verification process to confirm backup

2. **Distributed Security Options**
   - Encrypted key share fragments with recovery instructions
   - Separate storage location guidance
   - Verification of all components
   - Recovery procedure documentation

### Security Boundary Considerations
- All private key share material generated within secure boundary
- Backup verification occurs without exposing private key shares
- Key share usage strictly controlled through authentication

## 3. Bitcoin Transaction Operations

### Receiving Bitcoin

1. **Address Generation**
   - User requests new receiving address
   - Secure process derives standard single-signature address (P2WPKH)
   - Address displayed with QR code and metadata
   - Address verification option (on secondary device)

2. **Payment Monitoring**
   - UTXO monitoring for incoming transactions
   - Confirmation tracking with status updates
   - Balance updates with confirmation thresholds
   - Transaction details in wallet history

### Sending Bitcoin

1. **Transaction Construction**
   - Recipient address entry and validation
   - Amount specification with balance check
   - Fee selection with time/cost tradeoffs
   - UTXO selection (automatic or manual)
   - Change address management (internal)

2. **Transaction Review**
   - Complete transaction details displayed
   - Fee analysis with recommendations
   - Security policy compliance verification
   - Warnings for unusual parameters

3. **Threshold Signing Initiation (Device Key Share)**
   - User explicitly approves transaction
   - Authentication for signing operation
   - Request sent to secure boundary with approval proof
   - Secure process verifies policy compliance
   - Threshold signing protocol initiated with device key share

4. **Second Key Share Signing**
   - Signing request transferred to secondary signing source
   - User verifies transaction details on second device
   - Second key share participates in signing protocol
   - Threshold signature generated from participant key shares
   - Transaction fully signed with standard single signature
   - Transaction ready for broadcast

5. **Transaction Broadcast**
   - Network fee confirmation
   - Broadcast to Bitcoin network
   - Initial confirmation monitoring
   - Transaction receipt and status tracking

### Security Boundary Interactions
- Transaction construction in UI process
- Threshold signing exclusively in secure boundary
- Secure protocol for cross-device signing
- Policy validation enforced by secure process
- Only signed transaction hash returned to UI

## 4. Wallet Recovery

### Standard Recovery Process

1. **Recovery Initiation**
   - "Recover Existing Wallet" option
   - Threshold signature wallet type selection
   - Explanation of required components

2. **Key Share Import Process**
   - Import of any two key shares from original set
   - Secure validation of key share material
   - Creation of new key share to replace missing one
   - Wallet reconstruction with original wallet structure

3. **Wallet Reconstruction**
   - Public key information recreation
   - Address derivation path recovery
   - Blockchain scanning for balance verification
   - Transaction history recovery

4. **Security Restoration**
   - New key share generation for missing components
   - Updated backup creation
   - Security policy reapplication
   - Recovery documentation

### Emergency Access Options

1. **Fallback Access Mechanisms**
   - Pre-configured recovery transactions
   - Social recovery options (post-MVP)
   - Timelocked recovery paths (post-MVP)

2. **Security Considerations During Recovery**
   - Validation of recovery source legitimacy
   - Clear security status indicators
   - Guidance for secure recovery environment
   - Compromise detection measures

## 5. Security Policy Management

### Policy Configuration

1. **Policy Components**
   - Spending limits (per-transaction and time-based)
   - Authentication requirements
   - Address restrictions (whitelist/blacklist)
   - Time-based restrictions

2. **Policy Modification**
   - Current policy review interface
   - Strong authentication for changes
   - Clear security implications explanation
   - Cooling-off period for sensitive changes
   - Confirmation and acknowledgment

3. **Policy Enforcement**
   - All policies enforced by secure process
   - Cannot be bypassed by UI
   - Clear violation notifications
   - Exception handling process
   - Policy audit logging

### Security Boundary Interactions
- Policies stored within secure boundary
- Policy enforcement independent of UI
- Policy modifications require strong authentication
- Policy history maintained securely

## 6. Implementation Priorities

### MVP Essential Workflows

1. **Core Bitcoin Operations**
   - 2-of-3 threshold signature wallet creation
   - Key share backup and verification
   - Basic transaction creation and signing
   - Standard wallet recovery

2. **Security Foundations**
   - Process isolation security model
   - Basic security policy enforcement
   - Authentication and session management
   - Backup verification procedures

3. **Usability Baseline**
   - Clear security status indicators
   - Guided backup processes
   - Transaction review and approval flows
   - Recovery assistance mechanisms

### Security-Critical User Interactions

1. **Authentication Touchpoints**
   - Wallet unlocking
   - Transaction approval
   - Key operations
   - Policy modifications

2. **Security Awareness Features**
   - Security status dashboard
   - Clear boundary explanations
   - Authorization explanations
   - Threat mitigation guidance 