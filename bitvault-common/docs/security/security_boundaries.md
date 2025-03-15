# Security Boundaries in BitVault

This document outlines the key security boundaries in the BitVault wallet application and describes how they are implemented and enforced.

## Overview

Security boundaries are critical demarcation lines between components with different security requirements. Properly implemented boundaries help:

1. Limit the impact of security vulnerabilities
2. Prevent sensitive data from leaking across components
3. Establish clear validation and sanitization requirements
4. Improve security auditability
5. Define appropriate access control points

## Key Security Boundaries

BitVault implements several critical security boundaries:

### 1. UI/Core Boundary

**Description**: Separates the user interface from core cryptographic operations.

**Implementation**:
- Specific event types (`CoreRequest`, `CoreResponse`, `UiRequest`, `UiResponse`)
- Payload validation and sanitization in `MessageBus.publish`
- Security-focused logging at boundary crossings
- Strict input validation

**Security Concerns**:
- Prevents UI compromises from affecting wallet funds
- Enforces validation of all user input
- Allows separate security policies for UI and core components

### 2. Network/Wallet Boundary

**Description**: Isolates network operations from wallet state and cryptographic operations.

**Implementation**:
- Network events (`NetworkStatus`, `TransactionReceived`) require validation
- Network inputs are treated as untrusted
- Separate components for network and wallet operations

**Security Concerns**:
- Prevents network-based attacks from compromising wallet security
- Maintains wallet integrity even when network is compromised
- Protects against transaction malleability and other network attacks

### 3. UTXO/Transaction Boundary

**Description**: Separates UTXO management from transaction creation.

**Implementation**:
- UTXO selection strategies receive read-only access to UTXOs
- Transaction building occurs after UTXO selection with minimal crossing of data
- Events sanitize sensitive UTXO data when publishing

**Security Concerns**:
- Ensures transaction creation cannot modify UTXO state unexpectedly
- Protects against double-spend attempts
- Maintains accurate wallet balance information

### 4. Key Management Boundary

**Description**: Isolates cryptographic key material and operations.

**Implementation**:
- Strict verification of security boundary events in `MessageBus.publish`
- Validation preventing leakage of sensitive key material
- Automatic zeroization of sensitive memory
- Careful event filtering and sanitization

**Security Concerns**:
- Prevents exposure of private keys, seeds, and mnemonics
- Ensures cryptographic operations cannot be tampered with
- Maintains proper key handling throughout the wallet lifecycle

## Implementation Details

### Event System Security

The event system (`events.rs`) implements boundary enforcement through:

1. **Security Boundary Documentation**: Module-level documentation identifies boundaries
2. **Event Type Classification**: Specific event types for boundary crossings
3. **Validation and Sanitization**: Payloads validated when crossing boundaries
4. **Security Logging**: Enhanced logging for boundary crossings
5. **Rate Limiting**: Protection against DoS attacks via event flooding

### Key Security Functions

```rust
// Example of security boundary validation in MessageBus
fn is_security_boundary_event(&self, event_type: EventType) -> bool {
    matches!(event_type, 
        EventType::CoreRequest | 
        EventType::CoreResponse | 
        EventType::UiRequest | 
        EventType::UiResponse |
        EventType::SecurityAlert |
        EventType::KeyEvent
    )
}

// Validation of payloads crossing security boundaries
fn validate_security_payload(&self, event_type: EventType, payload: &str) -> Result<(), &'static str> {
    // Ensure payload is valid JSON
    if let Err(_) = serde_json::from_str::<serde_json::Value>(payload) {
        return Err("Invalid JSON payload");
    }
    
    // Specific validations based on event type
    match event_type {
        EventType::KeyEvent => {
            // Check for sensitive key material in payload
            if payload.contains("private_key") || 
               payload.contains("seed") || 
               payload.contains("mnemonic") {
                return Err("Sensitive key material detected in event payload");
            }
        },
        // Other validations...
    }
    
    Ok(())
}
```

## Best Practices

When crossing security boundaries:

1. **Always Validate**: Treat all input crossing a boundary as untrusted
2. **Minimize Data Transfer**: Only pass what's absolutely necessary
3. **Use Appropriate Event Types**: Choose event types that match the boundary being crossed
4. **Audit Boundary Crossings**: Log security-relevant details
5. **Sanitize Sensitive Data**: Never allow sensitive data to cross boundaries
6. **Rate Limit**: Apply rate limiting to prevent abuse
7. **Fail Securely**: When validation fails, handle errors safely

## Future Enhancements

Planned improvements to security boundaries:

1. Formal security boundary verification through static analysis
2. Enhanced auditing of boundary crossings
3. Automatic detection of sensitive data in event payloads
4. Integration with hardware security module boundaries
5. Fine-grained permission model for boundary crossings 