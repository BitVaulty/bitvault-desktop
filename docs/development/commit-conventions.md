# BitVault Commit Conventions

This document outlines the commit message conventions for the BitVault project. Following these guidelines ensures a consistent and readable git history, which is especially important for a security-focused project.

## Commit Message Format

Each commit message should be clear and reference relevant issues or PRs:

```
[optional Type:] <subject>

[optional body]

[references to issues/PRs]
```

### Subject Line

The subject line is mandatory and should:

1. **Use imperative mood** (e.g., "Add feature" not "Added feature" or "Adds feature")
2. **Begin with a capital letter**
3. **Do not end with a period**
4. **Keep it under 50 characters**
5. **Be descriptive and concise**

### Optional Type Prefix

While not required, a type prefix can help categorize changes:

- **Feature**: A new feature
- **Fix**: A bug fix
- **Docs**: Documentation changes
- **Style**: Formatting changes that don't affect code functionality
- **Refactor**: Code changes that neither fix bugs nor add features
- **Test**: Adding or modifying tests
- **Chore**: Changes to build process, tools, etc.
- **Security**: Changes that address security concerns

### Body

The body is optional but recommended for explaining the motivation behind the change:

1. **Use imperative, present tense**
2. **Include motivation for the change**
3. **Contrast with previous behavior if relevant**
4. **Wrap lines at 72 characters**

### Issue References

**Important**: Always reference related issues or PRs when applicable:

- **Fixes**: Use `Fixes #123` or `Closes #123` to automatically close the issue when merged
- **Related**: Use `Relates to #123` or `See #123` for related issues
- **Depends on**: Use `Depends on #123` for dependencies on other PRs

## Branch Naming Conventions

Branch names should be descriptive and follow a consistent pattern:

```
<type>/<issue-number>-<short-description>
```

### Branch Types

- **feature**: For new features
- **fix**: For bug fixes
- **docs**: For documentation changes
- **refactor**: For code refactoring
- **test**: For adding or modifying tests
- **security**: For security-related changes

### Examples

```
feature/123-add-threshold-signatures
fix/45-memory-leak-in-key-derivation
docs/89-update-setup-guide
security/56-improve-process-isolation
```

### Guidelines

1. Use lowercase for branch names
2. Use hyphens to separate words in the description
3. Keep branch names concise but descriptive
4. Always include the issue number when applicable
5. Delete branches after they are merged

## Examples

### Good Commit Messages

```
Add threshold signature verification

Implement verification of 2-of-3 threshold signatures to ensure
all signatures come from authorized key shares.

Fixes #45
```

```
Fix: Correct memory zeroing in key operations

Replace manual memory clearing with zeroize crate to ensure
cryptographic material is properly erased from memory.

Fixes #67, Relates to #89
```

```
Docs: Update security boundary documentation

Closes #123
```

```
Refactor: Improve IPC message validation

Relates to #456
```

### Bad Commit Messages

```
Added a feature  // Uses past tense instead of imperative
```

```
fix bug  // Not capitalized, too vague
```

```
Update documentation with some changes to make it better and more comprehensive for users.  // Too long, not specific
```

```
did some refactoring  // Not capitalized, too vague
```

## Security-Specific Guidelines

For a security-focused project like BitVault, additional guidelines apply:

1. **Never include sensitive information** in commit messages (keys, seeds, passwords)
2. **Explicitly mention security implications** when modifying security-critical code
3. **Reference security boundaries** when changes cross or affect them
4. **Indicate when changes affect key operations** or cryptographic functions
5. **Always reference security-related issues** with appropriate tags

Example:
```
Security: Move transaction validation to secure process

Relocates transaction validation logic from UI process to secure process
to prevent tampering with validation rules.

Security: Strengthens security boundary between UI and core.
Fixes #234
```

## PR Description Guidelines

Pull Request descriptions should provide context and summarize the changes:

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

### PR Description Guidelines

1. **Be comprehensive** but concise
2. **List all related issues**
3. **Summarize key changes** in bullet points
4. **Highlight security implications** if any
5. **Describe testing** performed
6. **Match commit messages** to changes listed
7. **Request specific reviewers** for security-critical changes

### Example PR Description

```
## Description
Add secure IPC authentication between UI and core processes

## Related Issues
Fixes #78
Relates to #45

## Changes
- Add HMAC-based message authentication
- Implement session key derivation
- Add nonce generation for replay protection
- Update IPC message format to include authentication

## Security Considerations
This change strengthens the security boundary between UI and core processes
by preventing message tampering and ensuring message authenticity.

## Testing
- Unit tests for authentication mechanism
- Integration tests for IPC communication
- Manual testing with message tampering attempts
```

## Review Feedback Commits

When addressing review feedback, follow these guidelines:

### Commit Message Format for Review Changes

```
Address review feedback: <specific change>

PR #123
```

### Guidelines for Review Feedback Commits

1. **Be specific** about what feedback is being addressed
2. **Reference the PR number** where the review occurred
3. **Keep changes focused** on addressing specific feedback
4. **Consider using the reviewer's username** for clarity
5. **Separate unrelated feedback** into different commits

### Examples

```
Address review feedback: Fix memory leak in key derivation

PR #92
```

```
Address review feedback: Improve error handling as suggested by @reviewer

PR #45
```

```
Address review feedback: Add missing security boundary checks

Implements additional validation as requested in review comments.

PR #123
```

## Emoji Usage Policy

Emojis are **discouraged** in commit messages for BitVault for the following reasons:

1. They can distract from the technical content
2. They may render differently across platforms
3. They make searching and filtering commit history more difficult
4. They can be inconsistently applied

Focus on clear, concise language instead of emoji to convey meaning and importance.

## Streamlined AI-Assisted Commits

For an agile workflow using AI to generate commit messages:

### Quick Commit Format

```
[optional Type:] <imperative action> <what changed>

Fixes #123
```

### AI Prompt Template

When asking AI to generate a commit message:

```
Generate a concise commit message in the imperative mood that describes these changes 
and references issue #XXX. Use capitalized type prefixes if appropriate: [briefly describe your changes]
```

### AI Review Checklist (5-Second Review)

Before accepting an AI-generated commit message, quickly check:

1. ✓ Uses imperative verb (Add, Fix, Update, etc.)
2. ✓ Clearly states what changed
3. ✓ Under 50 characters for the subject line
4. ✓ References relevant issues or PRs
5. ✓ Flags security implications if relevant
6. ✓ Type prefix is capitalized if used

### Examples of Streamlined AI Commits

```
Feature: Add secure IPC authentication

Fixes #78
```

```
Fix: Correct memory leak in key derivation

Closes #92, Relates to #45
```

```
Docs: Update setup guide for Arch Linux

Fixes #123
```

```
Security: Improve boundary validation in process isolation

Security-related change to prevent unauthorized access.
Fixes #56
```

### When to Add More Detail

For certain commits, a brief body may be necessary:

1. Security-critical changes
2. Non-obvious bug fixes
3. Major architectural changes
4. Breaking changes

In these cases, ask the AI to generate a brief explanation:

```
Generate a commit message with a brief explanation for this security-critical change 
that references issue #XXX. Use capitalized type prefixes: [describe your changes]
```

### Common Corrections for AI-Generated Messages

| Incorrect (Past Tense) | Correct (Imperative) |
|------------------------|----------------------|
| Added feature          | Add feature          |
| Fixed bug              | Fix bug              |
| Updated documentation  | Update documentation |
| Implemented tests      | Implement tests      |
| Refactored code        | Refactor code        |
| Improved performance   | Improve performance  |

## Commit Organization

1. **Make atomic commits** that address a single concern
2. **Separate refactoring from feature changes**
3. **Isolate security-critical changes** for easier review
4. **Group related changes** in a logical sequence
5. **Always reference relevant issues** in each commit

## Quick Reference

### Commit Message Essentials

| Element | Guideline |
|---------|-----------|
| Format | `[optional Type:] <imperative verb> <what changed>` |
| Verb Tense | Imperative (Add, Fix, Update) |
| Length | Subject under 50 chars, body lines under 72 |
| Issue References | Always include: Fixes #123 |
| Security Changes | Explicitly mention security implications |

### Common Types

| Type | Use For |
|------|---------|
| Feature | New functionality |
| Fix | Bug fixes |
| Docs | Documentation changes |
| Refactor | Code restructuring |
| Security | Security-related changes |

### Do's and Don'ts

| Do | Don't |
|----|-------|
| Use imperative mood | Use past tense |
| Reference issues | Include sensitive data |
| Be specific and concise | Be vague or verbose |
| Capitalize first word | Use emojis |
| Separate logical changes | Mix unrelated changes |

Following these conventions will help maintain a clean, understandable project history and facilitate security reviews of the codebase. 