# Security Policy

## Reporting a Vulnerability

The BitVault Wallet team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings and will make every effort to acknowledge your contributions.

### How to Report a Vulnerability

**Please DO NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to:
- security@bitvault.sv

Please include the following information in your report:
- Type of vulnerability
- Full paths of source file(s) related to the vulnerability
- Location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the vulnerability, including how an attacker might exploit it

### Response Timeline

- We aim to acknowledge receipt of vulnerability reports within 48 hours.
- We will provide a more detailed response within 7 days, indicating the next steps in handling your report.
- We will make our best effort to address and fix confirmed vulnerabilities in a timely manner.
- We will coordinate with you to determine an appropriate disclosure date for the vulnerability.

## Security Model

BitVault implements a security-focused architecture with proper isolation between:
- UI components (lower security level)
- Core cryptographic operations (higher security level)
- Key management services (highest security level)

### Key Security Features

- Separation of cryptographic operations from UI code
- Secure entropy generation for key creation
- Encrypted storage of sensitive data
- Memory protection for private key operations
- Input validation and sanitization
- Protection against side-channel attacks

A more detailed explanation of our security model and threat analysis will be provided in future documentation.

## Security Best Practices for Contributors

If you're contributing to BitVault, please follow these security best practices:

1. **Never commit private keys or secrets** to the repository, even in tests
2. **Use secure cryptographic libraries** and avoid implementing cryptographic primitives yourself
3. **Keep dependencies updated** to avoid known security vulnerabilities
4. **Follow the principle of least privilege** when designing API access and permissions
5. **Validate all user inputs** and never trust client-side validation alone
6. **Properly sanitize data** before displaying it to prevent XSS and injection attacks
7. **Report any security concerns** according to the vulnerability reporting process above

For more information on contributing to BitVault, including our code review process and quality standards, please see our [Contributing Guidelines](CONTRIBUTING.md).

## Third-Party Security Audits

No formal security audits have been conducted yet. We plan to engage third-party security researchers to audit the codebase once we reach a stable release.

## Updates to This Policy

This security policy may be updated from time to time. We will announce significant changes via GitHub releases. 