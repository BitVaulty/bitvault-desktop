# Contributing to BitVault Wallet

Thank you for your interest in contributing to BitVault Wallet! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md). We expect all contributors to adhere to these guidelines to ensure a positive and respectful community.

## Security Considerations

BitVault is a Bitcoin wallet application that handles sensitive cryptographic operations and user funds. Security is our highest priority.

**Before contributing, please review our [Security Policy](SECURITY.md).**

When working on features that involve:
- Cryptographic operations
- Key management
- Transaction handling
- Network communication
- Data storage

Please pay special attention to the security implications of your changes and document any security considerations in your pull requests.

For security issues, please follow the responsible disclosure process outlined in our [Security Policy](SECURITY.md) rather than filing a public issue.

## Project Overview

### Project Structure

BitVault is a Rust workspace project with the following crates:
- `bitvault-app` - Main application crate that ties everything together
- `bitvault-core` - Core wallet functionality and cryptographic operations
- `bitvault-common` - Shared utilities and types
- `bitvault-ipc` - Inter-process communication between components
- `bitvault-ui` - User interface components

### Prerequisites

- Rust (latest stable)
- Cargo (Rust package manager)
- Familiarity with Bitcoin concepts (for core functionality)

### Setting Up Development Environment

1. Fork the repository
2. Clone your fork locally
3. Install dependencies with `make setup`
4. Start the development server with `make dev`

## Development Workflow

### Branching Strategy

- `main` branch is always deployable
- Create feature branches from `main` using the format `feature/your-feature-name`
- Create bugfix branches using `fix/issue-description`

### Commit Messages

Follow conventional commits format:
```
type(scope): short description

Longer description if needed
```

Types include:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Pull Requests

1. Update your feature branch with the latest changes from `main`
2. Ensure tests pass with `make test`
3. Ensure linting passes with `make lint`
4. Open a PR against the `main` branch
5. Fill out the PR template completely

### Code Review Process

All submissions require review before being merged. Reviewers will check for:
- Functionality
- Security considerations
- Code quality
- Test coverage
- Documentation

## Quality Assurance

### Testing

- Write unit tests for new functionality
- Ensure existing tests continue to pass
- Include integration tests for components
- For Bitcoin-specific functionality, include test vectors from the Bitcoin Core test suite where applicable

Run tests with:
```
make test
```

### Documentation

- Update documentation when changing functionality
- Document APIs using standard formats
- Include examples where appropriate
- Document security considerations explicitly

### Style Guide

- Rust: Follow Rustfmt conventions
- Follow project-specific coding conventions

## License

By contributing to BitVault Wallet, you agree that your contributions will be licensed under the project's [Apache 2.0 License](LICENSE). 