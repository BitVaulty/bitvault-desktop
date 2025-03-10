# BitVault Core API Specification

This document defines the API interface between BitVault's security-critical core and non-critical UI components across the security boundary. The API is designed with extensibility for future Lightning Network and Liquid integration.

## 1. API Overview

### Design Principles

- **Minimal Surface Area**: Expose only essential operations across security boundaries
- **Complete Validation**: All inputs validated before processing
- **Explicit Authorization**: Clear requirements for each operation
- **Stateless When Possible**: Minimize persistent state in secure core
- **Deterministic Behavior**: Operations produce consistent, predictable results
- **Chain Extensibility**: API designed to support multiple chain types
- **Capability-Based Security**: Operations limited by explicit capabilities
- **Future-Proof Versioning**: Explicit versioning for all API components

### Communication Protocol

- Messages passed via platform-specific IPC mechanisms
- Request-response pattern for all operations
- Correlation IDs for asynchronous operations
- Strict schema validation for all data
- Chain type identification in all messages
- Capability verification for all requests

### Rate Limiting Specifications

- **Default Limits**:
  - Authentication attempts: 5 per minute, exponential backoff
  - Key operations: 10 per minute
  - Transaction signing: 5 per minute
  - Recovery operations: 3 per 10 minutes
  - Policy modifications: 3 per hour

- **Implementation**:
  - Token bucket algorithm for request limiting
  - Independent counters for each operation type
  - Authentication failure counter with exponential backoff
  - Persistent tracking across restarts
  - User notification on approaching limits
  - Rate limits can be adjusted by chain type and operation risk

## 2. Wallet Management API

### Initialize Wallet

**Purpose**: Create or load a wallet

**Authorization**: User password/biometrics

**Request**:
- Wallet identifier
- Network and address type configuration
- Storage location
- Authentication proof
- **Chain Types**: Supported chain type list (Bitcoin only for MVP)

**Response**:
- Public keys and master fingerprint
- Available capabilities
- Initialization status
- **Chain Capabilities**: Available features per chain type

**Rate Limits**:
- 3 attempts per minute
- 10 per day for failed attempts

### Get Wallet Status

**Purpose**: Retrieve wallet status

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- **Chain Types**: Optional chain filter

**Response**:
- Public information
- Balance data
- Policy and lock status
- **Chain-Specific Data**: Chain-aware status information

**Rate Limits**:
- 60 per minute

### Lock/Unlock Wallet

**Purpose**: Control access to signing capabilities

**Authorization**: User authentication for unlock, none for lock

**Request**:
- Wallet identifier
- Operation type
- Authentication proof (unlock only)
- Session duration (unlock only)
- **Capability Scope**: Requested capabilities to unlock

**Response**:
- Current lock status
- Session information (unlock only)
- **Available Capabilities**: Granted capabilities by chain type

**Rate Limits**:
- Unlock: 5 per minute, exponential backoff after failures
- Lock: Unlimited

## 3. Key Management API

### Generate New Key Shares

**Purpose**: Generate new key material for threshold signature scheme

**Authorization**: User authentication

**Request**:
- Key parameters (strength, algorithm)
- Backup preferences
- Authentication proof
- **Chain Purpose**: Intended chain usage (Bitcoin for MVP, extensible)
- **Key Purpose**: Functional purpose of keys

**Response**:
- Public key information
- Master fingerprint
- Backup verification data
- **Chain Capabilities**: Supported operations with these keys

**Rate Limits**:
- 3 per hour
- 10 per day

### Import Key Shares

**Purpose**: Import existing key material

**Authorization**: User authentication

**Request**:
- Material type (seed, private key share)
- Import format
- Authentication proof
- **Chain Type**: Chain this key material is for
- **Key Purpose**: Intended purpose of imported key

**Response**:
- Public key information
- Master fingerprint
- Import verification data
- **Chain Validation**: Chain-specific validation results

**Rate Limits**:
- 3 per hour
- 10 per day

### Export Public Information

**Purpose**: Export watch-only wallet data

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- Export format
- **Chain Type**: Chain to export for (Bitcoin for MVP)

**Response**:
- Public keys
- Derivation information
- Wallet configuration
- **Chain-Specific Data**: Chain-dependent export data

**Rate Limits**:
- 10 per hour

## 4. Address Management API

### Derive New Address

**Purpose**: Generate receiving address

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- Address type
- Derivation parameters
- Metadata (optional)
- **Chain Type**: Chain to derive address for (Bitcoin for MVP, extensible)

**Response**:
- Derived address
- Public derivation path
- Address type
- **Chain-Specific Data**: Chain-dependent address data

**Rate Limits**:
- 20 per minute
- 1000 per day

### Verify Address

**Purpose**: Confirm address ownership

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- Address
- Derivation path (optional)
- **Chain Type**: Chain this address belongs to

**Response**:
- Verification result
- Ownership status
- Derivation path (if owned)
- **Chain Validation**: Chain-specific validation information

**Rate Limits**:
- 30 per minute

## 5. Transaction API

### Create Unsigned Transaction

**Purpose**: Prepare transaction for signing

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- Recipients (addresses, amounts)
- Fee parameters
- UTXO selection (optional)
- Transaction preferences
- **Chain Type**: Chain for this transaction (Bitcoin for MVP)
- **Transaction Type**: Standard transaction type (extensible for different payment types)

**Response**:
- Unsigned transaction
- Fee calculation
- Change information
- Required signatures
- **Chain-Specific Data**: Chain-dependent transaction data

**Rate Limits**:
- 10 per minute
- 100 per day

### Sign Transaction

**Purpose**: Initiate threshold signing of prepared transaction

**Authorization**: Explicit user approval

**Request**:
- Wallet identifier
- Unsigned transaction
- User approval proof
- Signing preferences
- **Chain Type**: Chain for this transaction
- **Security Context**: Security parameters for this operation

**Response**:
- Signed transaction
- Transaction ID
- Signature information
- **Chain-Specific Results**: Chain-dependent signature data

**Rate Limits**:
- 5 per minute
- 50 per day
- Amount-based limits per security policy

### Get Transaction History

**Purpose**: Retrieve transaction records

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- Filters
- Pagination parameters
- **Chain Types**: Chain filter for history

**Response**:
- Transaction records
- Balance information
- Pagination metadata
- **Chain-Specific Data**: Chain-dependent transaction data

**Rate Limits**:
- 30 per minute

## 6. Security Policy API

### Get Policy Information

**Purpose**: Retrieve security policies

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- Policy type
- **Chain Type**: Chain filter for policies

**Response**:
- Current policy settings
- Enforcement status
- Modification history
- **Chain-Specific Policies**: Chain-dependent policy information

**Rate Limits**:
- 10 per minute

### Update Security Policy

**Purpose**: Modify security settings

**Authorization**: User authentication

**Request**:
- Wallet identifier
- Policy updates
- Authentication proof
- **Chain Type**: Chain this policy applies to
- **Policy Scope**: Scope of application

**Response**:
- Updated policy
- Effective timestamp
- Update confirmation
- **Chain-Specific Validation**: Chain-dependent policy validation

**Rate Limits**:
- 3 per hour
- Cooling period enforced between critical changes

### Verify Policy Compliance

**Purpose**: Pre-check operation compliance

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- Operation type
- Operation parameters
- **Chain Type**: Chain for this operation
- **Transaction Context**: Complete context for evaluation

**Response**:
- Compliance status
- Required authorizations
- Policy guidance
- **Chain-Specific Results**: Chain-dependent compliance data

**Rate Limits**:
- 20 per minute

## 7. Recovery API

### Create Recovery Kit

**Purpose**: Generate wallet recovery information

**Authorization**: User authentication

**Request**:
- Wallet identifier
- Recovery kit type
- Authentication proof
- **Chain Types**: Chains to include in recovery kit

**Response**:
- Recovery instructions
- Verification data
- Recovery metadata
- **Chain-Specific Recovery**: Chain-dependent recovery information

**Rate Limits**:
- 3 per day

### Verify Recovery Information

**Purpose**: Validate recovery data

**Authorization**: Session authentication

**Request**:
- Recovery information
- Verification type
- **Chain Type**: Chain this recovery is for

**Response**:
- Verification result
- Wallet identification
- Recovery capabilities
- **Chain-Specific Validation**: Chain-dependent recovery validation

**Rate Limits**:
- 10 per hour

### Perform Recovery

**Purpose**: Restore wallet

**Authorization**: Recovery credentials

**Request**:
- Recovery information
- Recovery credentials
- Target configuration
- **Chain Types**: Chains to recover
- **Recovery Priorities**: Order of recovery operations

**Response**:
- Recovery status
- Restored wallet information
- Next steps
- **Chain-Specific Results**: Chain-dependent recovery status

**Rate Limits**:
- 3 per day
- Progressive limits based on recovery attempt history

## 8. Session Management API

### Create Session

**Purpose**: Establish authenticated session

**Authorization**: User authentication

**Request**:
- Authentication credentials
- Session parameters
- Environment information
- **Requested Capabilities**: Capabilities to grant to this session

**Response**:
- Session identifier
- Session capabilities
- Expiration information
- **Granted Capabilities**: Capabilities granted by chain and operation

**Rate Limits**:
- 5 per minute
- Exponential backoff after failed attempts
- IP-based limits for repeated failures

### Refresh Session

**Purpose**: Extend session

**Authorization**: Valid session token

**Request**:
- Session identifier
- Refresh parameters
- **Capability Changes**: Requested capability modifications

**Response**:
- Updated session
- New expiration
- **Updated Capabilities**: Current capability status

**Rate Limits**:
- 30 per hour

### End Session

**Purpose**: Terminate session

**Authorization**: Session token or none

**Request**:
- Session identifier

**Response**:
- Termination confirmation

**Rate Limits**:
- Unlimited

## 9. Chain Management API

**Note**: This API section is defined for future extension but will be inactive in the MVP Bitcoin-only implementation.

### Get Chain Capabilities

**Purpose**: Query supported chain operations

**Authorization**: Session authentication

**Request**:
- Wallet identifier
- Chain type

**Response**:
- Supported operations
- Chain status
- Feature availability
- Configuration requirements

**Rate Limits**:
- 10 per minute

### Update Chain Configuration

**Purpose**: Modify chain-specific settings

**Authorization**: User authentication

**Request**:
- Wallet identifier
- Chain type
- Configuration changes
- Authentication proof

**Response**:
- Updated configuration
- Chain status
- Required actions

**Rate Limits**:
- 5 per hour

## 10. Error Handling

### Error Response Format

All errors include:
- Category (security, validation, system)
- Error code
- User-friendly message
- Recoverability status
- Suggested action
- Chain context (when applicable)

### Error Codes

| Code Range | Category | Description |
|------------|----------|-------------|
| 1000-1999  | Authentication | Authentication and session errors |
| 2000-2999  | Authorization | Permission and policy violations |
| 3000-3999  | Validation | Input parameter validation errors |
| 4000-4999  | Resource | Wallet and system resource issues |
| 5000-5999  | Operational | Execution and runtime errors |
| 6000-6999  | Security | Security policy and boundary violations |
| 7000-7999  | Rate Limiting | Throttling and request limit errors |
| 8000-8999  | Chain Specific | Chain-dependent errors |
| 9000-9999  | System | Internal system errors |

#### Common Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 1001 | Invalid credentials | Authentication failed |
| 1002 | Session expired | Authentication session timed out |
| 1005 | Too many attempts | Authentication rate limit exceeded |
| 2001 | Operation not authorized | User lacks permission |
| 2005 | Policy violation | Operation violates security policy |
| 3001 | Invalid parameter | Request parameter validation failed |
| 3005 | Schema violation | Message format validation failed |
| 4001 | Wallet not found | Requested wallet doesn't exist |
| 5001 | Signing failed | Transaction signing operation failed |
| 6001 | Security boundary violation | Attempted security boundary bypass |
| 7001 | Rate limit exceeded | Operation rate limit reached |
| 8001 | Unsupported chain operation | Operation not supported for this chain |
| 9001 | Internal error | Unspecified internal error |

### Common Error Categories

- **Authentication**: Credential or session issues
- **Authorization**: Insufficient permissions
- **Validation**: Invalid input parameters
- **Policy**: Security policy violations
- **Resource**: Wallet or system resource issues
- **Operational**: Execution failures
- **Chain-Specific**: Chain-dependent issues

## 11. Implementation Guidelines

### API Versioning

- Version information in all requests
- Backward compatibility within major versions
- Security updates may require version migration
- Chain-specific versioning for specialized operations

### Performance Considerations

- Progress reporting for long operations
- Rate-limiting for resource-intensive operations
- Critical operations prioritized
- Chain-dependent performance optimization

### Security Implementation

- Comprehensive logging (excluding sensitive data)
- Anomaly detection with verification escalation
- Temporary lockout after repeated failures
- Policy enforcement for all operations
- Chain-specific security validations

### Chain Extensibility Framework

- Chain type registry for supported operations
- Capability-based access control by chain type
- Chain-specific message validation
- Pluggable chain implementations
- Clear security boundaries between chain types