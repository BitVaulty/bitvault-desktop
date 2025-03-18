# BitVault Wallet

BitVault is a secure Bitcoin wallet built with Rust, focusing on security, reliability, and proper isolation of cryptographic operations.

## Project Structure

BitVault is a Rust workspace project with the following crates:
- `bitvault-app` - Main application crate that ties everything together
- `bitvault-core` - Core wallet functionality and cryptographic operations
- `bitvault-common` - Shared utilities and types
- `bitvault-ipc` - Inter-process communication between components
- `bitvault-ui` - User interface components

## Development Setup

### Prerequisites

- Rust (latest stable)
- Cargo (Rust package manager)
- Familiarity with Bitcoin concepts (for core functionality)

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/BitVaultWallet.git
   cd BitVaultWallet
   ```

2. Install dependencies:
   ```
   make setup
   ```

### Development

To start the development server:
```
make dev
```

This will launch the application in development mode.

### Building

To build the application for production:
```
make build
```

This will create platform-specific binaries in the release directory.

### Testing

To run tests:
```
make test
```

### Linting

To lint the codebase:
```
make lint
```

To automatically fix linting issues:
```
make lint-fix
```

## Documentation

For detailed documentation on the BitVault architecture, API, and security model, please refer to the [docs/](docs/) directory.

## Contributing

We welcome contributions to BitVault! Please read our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to make contributions.

This includes information on:
- Code style and standards
- Pull request process
- Development workflow
- Testing requirements

## Code of Conduct

This project adheres to a [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) that all contributors are expected to follow. Please read it before participating.

## Security

BitVault prioritizes security in handling Bitcoin keys and transactions. The wallet implements proper security isolation between UI components and core cryptographic operations.

Key security features include:
- Separation of cryptographic operations from UI code
- Secure entropy generation for key creation
- Encrypted storage of sensitive data
- Memory protection for private key operations

If you discover a security vulnerability, please refer to our [SECURITY.md](SECURITY.md) for the responsible disclosure process.

## License

[Apache License 2.0](LICENSE)