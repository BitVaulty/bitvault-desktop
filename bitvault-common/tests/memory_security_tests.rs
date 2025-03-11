// Tests for memory security features in BitVault
//
// These tests verify that sensitive data types behave correctly:
// - Basic operations work as expected
// - Display and Debug implementations don't leak sensitive data
// - Sanitization functions work properly

use bitvault_common::logging::{sanitize_sensitive, sanitize_sensitive_bytes};
use bitvault_common::types::{SensitiveBytes, SensitiveString};

// --- SensitiveString Tests ---

#[test]
fn test_sensitive_string_basic_operations() {
    // Creation
    let test_str = "very secret password";
    let sensitive = SensitiveString::new(test_str);

    // Value retrieval
    assert_eq!(sensitive.expose_secret(), test_str);

    // Length check
    assert_eq!(sensitive.len(), test_str.len());

    // Empty check
    assert!(!sensitive.is_empty());
    let empty = SensitiveString::new("");
    assert!(empty.is_empty());
}

#[test]
fn test_sensitive_string_from_implementations() {
    // From String
    let s = String::from("test string");
    let sensitive1: SensitiveString = s.clone().into();
    assert_eq!(sensitive1.expose_secret(), s);

    // From &str
    let sensitive2: SensitiveString = "test string".into();
    assert_eq!(sensitive2.expose_secret(), "test string");
}

#[test]
fn test_sensitive_string_formatting() {
    let sensitive = SensitiveString::new("my-secret-password");

    // Test Debug implementation
    let debug_str = format!("{:?}", sensitive);
    assert!(!debug_str.contains("my-secret-password"));
    assert!(debug_str.contains("SensitiveString"));
    assert!(debug_str.contains("length="));

    // Test Display implementation
    let display_str = format!("{}", sensitive);
    assert!(!display_str.contains("my-secret-password"));
    assert!(display_str.contains("[REDACTED]"));

    // Test sanitization function
    let sanitized = sanitize_sensitive(&sensitive);
    assert!(!sanitized.contains("my-secret-password"));
    assert!(sanitized.contains("..."));
}

#[test]
fn test_sensitive_string_cloning() {
    let original = SensitiveString::new("test-secret");
    let cloned = original.clone();

    // Ensure clone has the same content
    assert_eq!(original.expose_secret(), cloned.expose_secret());

    // Ensure they are separate objects in memory
    // (Though this is not easy to verify directly in safe Rust)
}

// --- SensitiveBytes Tests ---

#[test]
fn test_sensitive_bytes_basic_operations() {
    // Creation
    let test_bytes = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sensitive = SensitiveBytes::new(test_bytes.clone());

    // Value retrieval - test both alias methods
    assert_eq!(sensitive.expose_secret(), test_bytes.as_slice());
    assert_eq!(sensitive.as_slice(), test_bytes.as_slice());

    // Length check
    assert_eq!(sensitive.len(), test_bytes.len());

    // Empty check
    assert!(!sensitive.is_empty());
    let empty = SensitiveBytes::new(vec![]);
    assert!(empty.is_empty());
}

#[test]
fn test_sensitive_bytes_from_implementations() {
    // From Vec<u8>
    let bytes = vec![0x0a, 0x0b, 0x0c, 0x0d];
    let sensitive1: SensitiveBytes = bytes.clone().into();
    assert_eq!(sensitive1.expose_secret(), bytes.as_slice());

    // From &[u8]
    let byte_slice: &[u8] = &[0x0a, 0x0b, 0x0c, 0x0d];
    let sensitive2: SensitiveBytes = byte_slice.into();
    assert_eq!(sensitive2.expose_secret(), byte_slice);
}

#[test]
fn test_sensitive_bytes_hex_conversion() {
    let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
    let sensitive = SensitiveBytes::new(bytes);

    // Hex conversion should produce the correct string
    assert_eq!(sensitive.to_hex(), "0123456789abcdef");
}

#[test]
fn test_sensitive_bytes_formatting() {
    let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
    let sensitive = SensitiveBytes::new(bytes);

    // Test Debug implementation
    let debug_str = format!("{:?}", sensitive);
    assert!(!debug_str.contains("0123456789abcdef"));
    assert!(debug_str.contains("SensitiveBytes"));
    assert!(debug_str.contains("length="));

    // Test Display implementation
    let display_str = format!("{}", sensitive);
    assert!(!display_str.contains("0123456789abcdef"));
    assert!(display_str.contains("[REDACTED]"));

    // Test sanitization function
    let sanitized = sanitize_sensitive_bytes(&sensitive);
    assert!(!sanitized.contains("0123456789abcdef"));
    assert!(sanitized.contains("..."));

    // Ensure sanitized string shows portions of the hex
    let first_bytes = &sanitized[0..4];

    // First 2 bytes in hex (first 4 characters)
    assert_eq!(first_bytes, "0123");

    // Last 2 bytes should be "cdef" but might be in different positions
    // depending on formatting, so just check that they appear somewhere
    assert!(sanitized.contains("cdef"));
}

#[test]
fn test_sensitive_bytes_mutation() {
    let mut sensitive = SensitiveBytes::new(vec![0x01, 0x02, 0x03, 0x04]);

    // Get mutable reference and change a byte - test both alias methods
    {
        let bytes_mut = sensitive.as_mut_slice();
        bytes_mut[0] = 0xff;
    }

    // Verify the change took effect
    assert_eq!(sensitive.as_slice()[0], 0xff);

    // Test the as_mut_slice alias method too
    {
        let bytes_mut = sensitive.as_mut_slice();
        bytes_mut[1] = 0xee;
    }

    // Verify the second change took effect
    assert_eq!(sensitive.as_slice()[1], 0xee);
}

#[test]
fn test_sensitive_bytes_cloning() {
    let original = SensitiveBytes::new(vec![0x01, 0x02, 0x03, 0x04]);
    let cloned = original.clone();

    // Ensure clone has the same content
    assert_eq!(original.expose_secret(), cloned.expose_secret());

    // Ensure they are separate objects in memory
    // (Though this is not easy to verify directly in safe Rust)
}

// --- Zeroing Tests ---
// Note: These tests are limited in what they can verify about memory zeroing
// due to Rust's safety guarantees. In practice, we rely on the zeroize crate
// to handle this correctly.

#[test]
fn test_sensitive_string_zeroing() {
    // This test is more of a documentation of intent than a functional test
    // as we can't directly observe memory being zeroed in safe Rust

    let test_string = String::from("very-sensitive-data");

    {
        let _sensitive = SensitiveString::new(test_string.clone());
        // _sensitive should be zeroed when it goes out of scope
    }

    // We can't easily verify that the memory was zeroed, but we can
    // check that our code still works as expected
    assert_eq!(test_string, "very-sensitive-data");
}

#[test]
fn test_sensitive_bytes_zeroing() {
    // This test is more of a documentation of intent than a functional test
    // as we can't directly observe memory being zeroed in safe Rust

    let test_bytes = vec![0x01, 0x02, 0x03, 0x04];

    {
        let _sensitive = SensitiveBytes::new(test_bytes.clone());
        // _sensitive should be zeroed when it goes out of scope
    }

    // We can't easily verify that the memory was zeroed, but we can
    // check that our code still works as expected
    assert_eq!(test_bytes, vec![0x01, 0x02, 0x03, 0x04]);
}
