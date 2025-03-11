# BitVault: Dependency Management Guidelines

## Introduction

This document provides guidance on managing dependencies in BitVault, with specific instructions for AI assistants helping with development. Managing dependencies correctly is critical for security, compatibility, and maintainability in cryptocurrency software.

## Dependency Management Principles

1. **Security First**: Dependencies should be vetted for security vulnerabilities
2. **Stability**: Prefer stable, well-tested versions over bleeding edge
3. **Minimal Dependencies**: Only add dependencies when necessary
4. **Version Pinning**: Pin specific versions to ensure reproducible builds
5. **API Compatibility**: Be aware of breaking changes in dependency APIs

## AI Guidance for Dependency Management

### Checking Current Versions

When advising on dependencies, AI assistants should:

1. **Use `cargo outdated` to check for the latest versions**:
   - Always run `cargo outdated` to identify newer versions of dependencies
   - Example: `cargo outdated -p bdk` to check Bitcoin Development Kit versions
   - Pay attention to the "Latest Compatible" column, not just "Latest"

2. **Examine upgrade implications**:
   - Notice the "Breaking" flag for versions that might introduce API changes
   - Look at dependency changelogs when possible
   - Consider the dependency's stability and release cycle

3. **Consider compatibility with the existing codebase**:
   - Cross-check with other dependencies that might be affected
   - Consider platform-specific implications

### Suggesting Version Updates

When recommending dependency version changes:

1. **Follow a progressive approach**:
   - Recommend incremental updates rather than major jumps when possible
   - Prioritize security updates over feature additions
   - For critical security components like BDK, prefer well-established releases

2. **Only suggest downgrading when necessary**:
   - Downgrading should be a last resort, used only when:
     - Newer versions have known bugs affecting our use case
     - API incompatibilities cannot be reasonably fixed
     - Dependency conflicts cannot be resolved with newer versions
   - Always explain the specific reason for suggesting a downgrade

3. **Provide context and justification**:
   - Explain the benefits of the version update
   - Outline potential code changes needed for the update
   - Reference changelogs or documentation when possible

### Handling Version Conflicts and API Changes

1. **Analyzing API changes**:
   - For major version updates, identify potential breaking changes
   - Suggest patterns for adapting code to new APIs
   - Propose incremental changes to minimize risk

2. **Testing strategy**:
   - Recommend specific tests to verify compatibility
   - Suggest focused test cases for affected functionality

## Bitcoin-Specific Dependencies

### BDK (Bitcoin Development Kit)

BDK is a critical dependency that deserves special attention:

1. **Version selection criteria**:
   - Prefer versions that have been well-tested in production
   - Consider compatibility with our target Bitcoin network version
   - Be aware of security implications in cryptographic libraries

2. **Version check procedure**:
   - Run: `cargo outdated -p bdk` to check available versions
   - Check BDK GitHub releases for detailed notes: https://github.com/bitcoindevkit/bdk/releases
   - Verify compatibility with our Bitcoin version

3. **Feature flags management**:
   - Check required and optional feature flags
   - Only enable features we actually need (principle of least privilege)

### Bitcoin Crate

The Bitcoin crate provides fundamental Bitcoin data structures:

1. **Version selection criteria**:
   - Must be compatible with BDK version in use
   - Be aware of network consensus changes

2. **Version check procedure**:
   - Run: `cargo outdated -p bitcoin` to check available versions
   - Check compatibility with BDK's required Bitcoin version

## Security-Critical Dependencies

For cryptographic and security libraries (like `zeroize`, `aes-gcm`, etc.):

1. **Version selection priority**:
   - Security fixes take absolute priority
   - Regularly check for security advisories
   - When in doubt, prefer newer versions with security fixes

2. **Verification procedure**:
   - Check the RustSec Advisory Database: `cargo audit`
   - Review release notes for security implications

## Example Workflow for AI Assistance

When asked to suggest dependency updates:

1. **Analyze current setup**:
   ```bash
   cargo outdated -p <package_name>
   ```

2. **Research version implications**:
   - Check release notes
   - Examine API changes
   - Consider security implications

3. **Recommend appropriate action**:
   - Suggest specific version
   - Explain changes needed in our code
   - Outline testing procedure to verify compatibility

4. **Follow up with verification**:
   - Recommend test commands to run
   - Suggest targeted areas for manual testing

## Automated Dependency Management

AI should recommend implementing these automated safeguards:

1. **Add to CI/CD pipeline**:
   - Regular `cargo audit` checks
   - Dependency update notifications

2. **In build scripts**:
   - Version compatibility checks
   - Feature flag validation

---

By following these guidelines, AI assistants can help maintain a secure, stable dependency graph while making appropriate updates when beneficial. 