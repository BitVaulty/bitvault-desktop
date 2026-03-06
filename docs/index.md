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

- [Quick Start](../README.md#quick-start) - Build, run, and test
- [Development](../README.md#development) - Platform setup and Makefile targets
- [Arch Linux setup](setup/arch-linux-setup.md) - Platform-specific dependencies

### Project Architecture

BitVault Desktop is a Rust application with the following structure:
- Application source at repo root (`src/`, `tests/`)
- [bitvault-common](https://github.com/BitVaulty/bitvault-common) - Shared utilities and types (external dependency)

## Security Overview

BitVault prioritizes security in handling Bitcoin keys and transactions. The wallet implements proper security isolation between UI components and core cryptographic operations.

Key security features include:
- Separation of cryptographic operations from UI code
- Secure entropy generation for key creation
- Encrypted storage of sensitive data
- Memory protection for private key operations

For security vulnerability reporting, please refer to [SECURITY.md](../SECURITY.md).

## Documentation

### Architecture & Design
- [architecture-overview.md](design/architecture-overview.md) - Architecture and module layout
- [security-boundaries.md](design/security-boundaries.md) - Security boundaries
- [threat-model.md](design/threat-model.md) - Threat analysis
- [ai-context.md](design/ai-context.md) - AI assistance context and doc map

### Setup & Development
- [arch-linux-setup.md](setup/arch-linux-setup.md) - Development setup (Arch Linux)
- [contribution-guidelines.md](development/contribution-guidelines.md) - Contribution process
- [E2E_TESTING.md](development/E2E_TESTING.md) - End-to-end testing

## Contributing to Documentation

Documentation improvements are highly valued contributions. If you'd like to help expand our documentation, please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

When contributing to documentation:
1. Focus on documenting existing functionality first
2. Ensure technical accuracy
3. Include code examples where appropriate
4. Document security considerations explicitly 