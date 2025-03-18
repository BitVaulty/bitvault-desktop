# BitVault Wallet Documentation

Welcome to the BitVault Wallet documentation. This documentation provides information about the BitVault wallet's implementation, architecture, and development guidelines.

## Documentation Overview

### Core Documentation

- [README.md](../README.md) - Project overview, setup instructions, and development commands
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines and development workflow
- [SECURITY.md](../SECURITY.md) - Security policy and vulnerability reporting
- [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) - Community standards and expectations
- [LICENSE](../LICENSE) - Apache 2.0 license information

### Development Resources

- [Development Setup](../README.md#development-setup) - Prerequisites and installation
- [Development Workflow](../README.md#development) - Development processes
- [Building](../README.md#building) - Production builds
- [Testing](../README.md#testing) - Test execution
- [Linting](../README.md#linting) - Code quality enforcement

### Project Architecture

BitVault is a Rust workspace project with the following crates:
- [bitvault-app](../bitvault-app/) - Main application crate that ties everything together
- [bitvault-core](../bitvault-core/) - Core wallet functionality and cryptographic operations
- [bitvault-common](../bitvault-common/) - Shared utilities and types
- [bitvault-ipc](../bitvault-ipc/) - Inter-process communication between components
- [bitvault-ui](../bitvault-ui/) - User interface components

## Security Overview

BitVault prioritizes security in handling Bitcoin keys and transactions. The wallet implements proper security isolation between UI components and core cryptographic operations.

Key security features include:
- Separation of cryptographic operations from UI code
- Secure entropy generation for key creation
- Encrypted storage of sensitive data
- Memory protection for private key operations

For security vulnerability reporting, please refer to [SECURITY.md](../SECURITY.md).

## Documentation Roadmap

As BitVault development progresses, we plan to expand documentation in the following areas:

1. **Architecture Documentation** - Detailed explanation of the wallet architecture and component interactions
2. **API Reference** - Documentation for the wallet's core APIs and BDK integration
3. **Security Model** - Comprehensive security model including the threat model and key management approach
4. **User Guides** - End-user documentation for wallet setup and usage

## Contributing to Documentation

Documentation improvements are highly valued contributions. If you'd like to help expand our documentation, please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

When contributing to documentation:
1. Focus on documenting existing functionality first
2. Ensure technical accuracy
3. Include code examples where appropriate
4. Document security considerations explicitly 