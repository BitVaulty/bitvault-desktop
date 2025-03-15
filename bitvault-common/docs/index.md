# BitVault Documentation

Welcome to the BitVault documentation. This directory contains detailed information about the design, architecture, and implementation of the BitVault Bitcoin wallet.

## Contents

The documentation covers the following topics:

1. **Architecture**
   - [Event-Driven Architecture](architecture/event_architecture.md) - Our message-based component communication system
   - [Architecture Overview](architecture/updated_architecture.md) - The current architecture of BitVault

2. **Security**
   - [Security Boundaries](security/security_boundaries.md) - Critical security demarcation points and enforcement

3. **Platform**
   - [Platform Overview](platform/platform_overview.md) - Platform abstraction and capabilities

4. **Key Management**
   - [Key Management Overview](key_management/key_management_overview.md) - Cryptographic key management

5. **UTXO Management**
   - [UTXO Management](utxo/utxo_management.md) - Overview of UTXO handling
   - [UTXO Implementation](utxo/utxo_implementation.md) - Technical details of UTXO selection
   - [UTXO Testing](utxo/utxo_testing.md) - Testing approach and findings
   - [UTXO Examples](utxo/utxo_examples.md) - Code examples for UTXO selection

## Key Design Principles

BitVault follows these core design principles:

1. **Security First** - Security takes precedence over convenience
2. **Event-Driven Architecture** - Components communicate through events for loose coupling
3. **Proper Security Isolation** - Clear boundaries between security-critical and non-critical components
4. **Cross-Platform Support** - Core functionality works across desktop and mobile platforms
5. **Testability** - All components are designed with testing in mind

## Documentation Standards

The documentation in this repository follows these guidelines:

1. **Source Code is Canonical** - The actual implementation in source code is the ultimate reference
2. **Security Boundaries are Explicit** - Security-critical components are clearly marked
3. **File Naming** - All documentation files use lowercase with underscores (e.g., `file_name.md`)
4. **Cross-References** - Documentation references related components when appropriate

## Important Implementation Notes

- Security-sensitive operations are clearly marked and isolated
- Event flows crossing security boundaries are carefully controlled
- The wallet uses BDK (Bitcoin Development Kit) for core Bitcoin functionality
- Domain-specific events allow for better isolation of components

## Getting Started

For developers looking to contribute to BitVault or understand its architecture:

1. Start with understanding the [Event-Driven Architecture](architecture/event_architecture.md)
2. Review the [Security Boundaries](security/security_boundaries.md) to understand security considerations
3. Check the module documentation in the codebase for implementation details 