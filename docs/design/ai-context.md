# BitVault: Bitcoin Wallet AI Context Document

## Project Definition

### Core Identity
- **Name**: BitVault
- **Type**: Non-custodial Bitcoin wallet
- **Scope**: Bitcoin-only (no altcoins) with optional Lightning Network support
- **Primary Value Proposition**: Maximum security with practical usability
- **Development Stage**: Early architecture and design phase

### Architectural Foundations
- **Threshold Signature Model**: 2-of-3 Threshold Signature Scheme (TSS) by default
- **Security Boundaries**: Isolation between security-critical and general application components
- **Cross-Platform Strategy**: Native Rust core with platform-specific security adaptations
- **Component Separation**: Distinct modules for cryptographic operations, business logic, and UI
- **Trust Model**: Zero-trust design, assuming all external components may be compromised

> **For complete architectural details**: See [architecture-overview.md](architecture-overview.md)

## Design Documentation Map

### Security Architecture
- **Security Principles**: Non-custodial control, defense in depth, least privilege, etc.
- **Security Boundaries**: Process isolation, IPC security, platform-specific protections
- **Threat Model**: Comprehensive threat vectors and mitigation strategies
- **Platform-Specific Security**: Tailored security approaches for desktop, mobile, and browser

> **Related Documents**:
> - [security-boundaries.md](security-boundaries.md) - Detailed security isolation model
> - [threat-model.md](threat-model.md) - Comprehensive threat analysis
> - [platform-security.md](platform-security.md) - Platform-specific security implementations
> - [technical-decisions.md](technical-decisions.md) - Security-related technical decisions and rationales

### Bitcoin Implementation
- **Wallet Model**: 2-of-3 threshold signature scheme (TSS)
- **Transaction Flow**: Construction, approval, signing, and broadcast
- **Key Management**: Generation, storage, derivation, and backup
- **Address & UTXO Handling**: Types, derivation, and management

> **Related Documents**:
> - [bitcoin-implementation.md](bitcoin-implementation.md) - Detailed Bitcoin implementation strategy
> - [core-api.md](core-api.md) - API specification for core Bitcoin operations

### Recovery & Backup Systems
- **Key Share Management**: Device, backup, and recovery shares
- **Backup Mechanisms**: Multiple independent backup methods
- **Verification Protocols**: Ensuring backup integrity
- **Recovery Procedures**: Standard and emergency paths

> **Related Document**:
> - [recovery-backup.md](recovery-backup.md) - Comprehensive backup and recovery strategy

### Technical Implementation
- **Technology Stack**: Rust, Bitcoin Development Kit (BDK), egui, etc.
- **Codebase Structure**: Modular organization with security boundaries
  - **`bitvault-core`**: Security-critical operations (isolated context)
  - **`bitvault-common`**: Shared components and interfaces
  - **`bitvault-ui`**: User interface and interaction
  - **`bitvault-app`**: Platform integration

> **Related Documents**:
> - [architecture-overview.md](architecture-overview.md) - Detailed module structure and relationships
> - [implementation-plan.md](implementation-plan.md) - Prioritized implementation roadmap
> - [testing-strategy.md](testing-strategy.md) - Comprehensive testing approach

### User Experience & Workflows
- **Key User Journeys**: Wallet creation, transaction operations, recovery
- **Security Visualization**: Clear communication of security state
- **Authorization Flows**: Explicit approval with appropriate security context

> **Related Documents**:
> - [user-workflows.md](user-workflows.md) - Detailed user journey definitions
> - [ui-ux-design.md](ui-ux-design.md) - UI/UX principles and implementation

## Development Status and Roadmap

### Current Implementation Status
- Basic Rust workspace structure established
- Architectural design and security planning
- Technology selection completed
- Proof-of-concept security boundaries

### Immediate Development Priorities
- Core wallet functionality (BDK integration)
- Security boundary implementation
- Basic UI implementation with egui
- Cross-platform compilation targets

> **For detailed implementation plan**: See [implementation-plan.md](implementation-plan.md)

## Collaboration Guidelines

### Terminology Standards
- **Wallet**: The complete BitVault application
- **Vault**: A specific security configuration or profile
- **Keypair**: A public/private key combination used for signing
- **Signing Device**: Any hardware or software component that can sign transactions
- **Security Policy**: User-defined rules governing transaction approval
- **Recovery Keys**: Backup keys for account restoration
- **Timelock**: Time-based restriction on transaction execution
- **UTXO**: Unspent Transaction Output, the fundamental unit of Bitcoin value

### AI Assistance Parameters

When providing assistance on BitVault development:

#### Security Context Awareness
- **Module Context**: Specify which module is being discussed (core/common/ui/app)
- **Security Boundary Impact**: Note when changes cross security boundaries
- **Bitcoin-Specific Functionality**: Flag Bitcoin-protocol specific components

#### Development Priorities
- **Security First**: Prioritize security over convenience
- **Cross-Platform**: Consider implications for all target platforms
- **Code Simplicity**: Prefer simple, auditable implementations over clever optimizations
- **Documentation**: Emphasize thorough documentation, especially for security aspects

#### Critical Decisions
When addressing questions that involve:
- Security model trade-offs
- Cross-platform compatibility
- Bitcoin protocol specifics
- Key management approaches

Clearly highlight implications and reference relevant design documents.
