# BitVault Contribution Guidelines

Thank you for your interest in contributing to BitVault, a security-focused Bitcoin wallet. This document outlines the process for contributing to the project and the standards we expect contributors to follow.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Security Considerations](#security-considerations)
3. [Getting Started](#getting-started)
4. [Development Environment](#development-environment)
5. [Contribution Process](#contribution-process)
6. [Pull Request Guidelines](#pull-request-guidelines)
7. [Security Disclosure Process](#security-disclosure-process)
8. [Code Style and Standards](#code-style-and-standards)
9. [Testing Requirements](#testing-requirements)
10. [Documentation](#documentation)
11. [License and Copyright](#license-and-copyright)
12. [Communication Channels](#communication-channels)

## Code of Conduct

We are committed to providing a friendly, safe, and welcoming environment for all contributors. By participating in this project, you agree to abide by our Code of Conduct (see [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)).

## Security Considerations

BitVault is a security-critical application that handles Bitcoin private keys and transactions. All contributions must prioritize security over convenience or features.

### Security Principles

1. **Defense in Depth**: Multiple layers of security controls
2. **Least Privilege**: Components should have minimal necessary access
3. **Secure by Default**: Security should not depend on optional configuration
4. **Zero Trust**: Assume all external components may be compromised
5. **Explicit Security Boundaries**: Clear isolation between security domains

### Security-Critical Areas

Contributions to these areas require additional scrutiny and expertise:

- `bitvault-core`: Contains all cryptographic operations and key management
- Security boundary implementations (process isolation, IPC)
- Cryptographic operations and key handling
- Transaction signing and validation
- Security policy enforcement

## Getting Started

### Issue Selection

1. Start with issues labeled `good-first-issue` or `help-wanted`
2. Comment on the issue to express your interest before starting work
3. For security-critical issues, discuss your approach with maintainers first
4. Create a new issue if you find a bug or have a feature suggestion

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone git@github.com:YOUR_USERNAME/BitVaultWallet.git
   cd BitVaultWallet
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream git@github.com:BitVaulty/BitVaultWallet.git
   ```

## Development Environment

Follow the setup instructions in [docs/setup/arch-linux-setup.md](../setup/arch-linux-setup.md) to configure your development environment.

### Security Requirements

1. Keep your development environment updated with security patches
2. Use disk encryption to protect source code and test data
3. Never commit sensitive test data (private keys, seeds, etc.)
4. Be cautious when installing development dependencies

## Contribution Process

### Branching Strategy

1. Create a branch from `main` using the naming convention:
   ```
   <type>/<issue-number>-<short-description>
   ```
   Example: `feature/123-add-threshold-signatures`

2. Keep your branch focused on a single issue or feature
3. Regularly rebase your branch on the latest `main`

### Commit Guidelines

Follow the commit message conventions in [docs/development/commit-conventions.md](./commit-conventions.md).

Key points:
- Use imperative mood in commit messages
- Reference issues in commit messages
- Make atomic, focused commits
- Explicitly mention security implications

## Pull Request Guidelines

### Before Submitting

1. Ensure your code follows the project's style guidelines
2. Add or update tests to cover your changes
3. Ensure all tests pass locally
4. Update documentation as needed
5. Rebase your branch on the latest `main`

### PR Submission

1. Create a pull request against the `main` branch
2. Fill out the PR template completely
3. Link to any related issues
4. Explicitly describe security implications
5. Request reviews from appropriate maintainers

### PR Description Format

```
## Description
Brief description of the changes

## Related Issues
Fixes #123
Relates to #456

## Changes
- Change 1
- Change 2
- Change 3

## Security Considerations
Any security implications of these changes

## Testing
How these changes were tested
```

### Review Process

1. All PRs require at least one review from a maintainer
2. Security-critical changes require additional reviews
3. Address all review comments promptly
4. Maintainers may request changes or additional tests
5. Once approved, a maintainer will merge your PR

## Security Disclosure Process

### Reporting Security Issues

**Do not report security vulnerabilities through public GitHub issues.**

Instead:

1. Email security@bitvault.example.com with details about the vulnerability
2. Include steps to reproduce, impact, and suggested mitigation if possible
3. Allow time for the issue to be addressed before public disclosure

### Responsible Disclosure

We follow responsible disclosure principles:

1. We will acknowledge receipt of your report within 48 hours
2. We will provide an estimated timeline for a fix
3. We will notify you when the issue is fixed
4. We will recognize your contribution (unless you prefer to remain anonymous)

## Code Style and Standards

### Rust Guidelines

1. Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
2. Use `cargo fmt` and `cargo clippy` before submitting code
3. Avoid `unsafe` code unless absolutely necessary
4. Provide comprehensive error handling
5. Use strong typing and avoid type casting

### Security-Specific Guidelines

1. Use constant-time operations for cryptographic comparisons
2. Explicitly zero memory containing sensitive data
3. Validate all inputs, especially across security boundaries
4. Use defensive programming techniques
5. Document security assumptions and guarantees

## Testing Requirements

### Minimum Test Coverage

1. All new code should have unit tests
2. Security-critical code requires additional integration tests
3. Boundary-crossing code requires specific security tests
4. Bug fixes should include regression tests

### Security Testing

1. Test for proper isolation across security boundaries
2. Verify cryptographic operations against test vectors
3. Test error handling and edge cases
4. Validate memory handling for sensitive data

## Documentation

### Code Documentation

1. Use doc comments (`///`) for public API documentation
2. Document security guarantees and assumptions
3. Explain complex algorithms or security measures
4. Update existing documentation affected by your changes

### User Documentation

1. Update user-facing documentation for new features
2. Document security implications for user actions
3. Provide clear error messages and recovery steps

## License and Copyright

### License

BitVault is licensed under [LICENSE NAME]. By contributing to BitVault, you agree to license your contributions under the same license.

### Copyright Assignment

Contributors retain copyright to their contributions but grant the project the right to use and distribute those contributions according to the project license.

### Third-Party Code

1. Clearly identify any third-party code in your contributions
2. Ensure third-party code has a compatible license
3. Document the source and license of third-party code

## Communication Channels

- **GitHub Issues**: Feature requests, bug reports, and project planning
- **Pull Requests**: Code review and technical discussion
- **Security Issues**: security@bitvault.sv for private disclosure

---

Thank you for contributing to BitVault! Your efforts help build a more secure Bitcoin wallet for everyone. 