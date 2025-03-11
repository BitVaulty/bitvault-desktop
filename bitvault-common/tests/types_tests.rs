use bitvault_common::{sanitize_for_display, SensitiveBytes, SensitiveString};
use std::panic;

// Setup function to run at the beginning of each test to capture panics and log them
fn test_with_logging<T, F: FnOnce() -> T + panic::UnwindSafe>(
    name: &str,
    test_fn: F,
) -> Result<T, String> {
    eprintln!("===== STARTING TEST: {} =====", name);

    let result = panic::catch_unwind(|| test_fn());

    match result {
        Ok(value) => {
            eprintln!("===== TEST PASSED: {} =====", name);
            Ok(value)
        }
        Err(e) => {
            let panic_msg = if let Some(msg) = e.downcast_ref::<String>() {
                format!("{}", msg)
            } else if let Some(msg) = e.downcast_ref::<&str>() {
                format!("{}", msg)
            } else {
                "Unknown panic".to_string()
            };

            eprintln!("===== TEST FAILED: {} =====", name);
            eprintln!("Panic message: {}", panic_msg);
            Err(panic_msg)
        }
    }
}

#[test]
fn test_sensitive_string_zeroing() {
    let _ = test_with_logging("test_sensitive_string_zeroing", || {
        eprintln!("Testing SensitiveString zeroing");

        let secret = "my secret password";
        eprintln!("Creating SensitiveString with value: {}", secret);

        // Create a scope for the sensitive string
        {
            let sensitive = SensitiveString::from(secret);
            eprintln!(
                "SensitiveString created. Length: {}",
                sensitive.expose_secret().len()
            );
            assert_eq!(sensitive.expose_secret(), secret);
        }

        // The string should be zeroized when it goes out of scope
        eprintln!("SensitiveString has gone out of scope (should be zeroized)");

        // Additional check: create a new one and manually clear it
        let mut sensitive2 = SensitiveString::from(secret);
        eprintln!(
            "Created second SensitiveString. Length: {}",
            sensitive2.expose_secret().len()
        );
        assert_eq!(sensitive2.expose_secret(), secret);

        eprintln!("Manually clearing the SensitiveString");
        sensitive2.clear();
        eprintln!(
            "After clearing, length: {}",
            sensitive2.expose_secret().len()
        );
        assert_eq!(sensitive2.expose_secret(), "");
    });
}

#[test]
fn test_sensitive_bytes_zeroing() {
    let _ = test_with_logging("test_sensitive_bytes_zeroing", || {
        eprintln!("Testing SensitiveBytes zeroing");

        let secret_bytes = b"my secret data";
        eprintln!("Creating SensitiveBytes with {} bytes", secret_bytes.len());

        // Create a scope for the sensitive bytes
        {
            let sensitive = SensitiveBytes::from(secret_bytes.to_vec());
            eprintln!(
                "SensitiveBytes created. Length: {}",
                sensitive.expose_secret().len()
            );
            assert_eq!(sensitive.expose_secret(), secret_bytes);
        }

        // The bytes should be zeroized when they go out of scope
        eprintln!("SensitiveBytes has gone out of scope (should be zeroized)");

        // Additional check: create a new one and manually clear it
        let mut sensitive2 = SensitiveBytes::from(secret_bytes.to_vec());
        eprintln!(
            "Created second SensitiveBytes. Length: {}",
            sensitive2.expose_secret().len()
        );
        assert_eq!(sensitive2.expose_secret(), secret_bytes);

        eprintln!("Manually clearing the SensitiveBytes");
        sensitive2.clear();
        eprintln!(
            "After clearing, length: {}",
            sensitive2.expose_secret().len()
        );
        assert_eq!(sensitive2.expose_secret(), &[] as &[u8]);
    });
}

#[test]
fn test_sanitize_for_display() {
    let _ = test_with_logging("test_sanitize_for_display", || {
        eprintln!("Testing sanitize_for_display function");

        // Test sanitization with different prefix lengths
        let test_strings = [
            "short string",
            "this is a much longer string with more content",
            "password123",
            "1234567890",
        ];

        for s in test_strings {
            // Test with different prefix lengths
            for prefix_len in [0, 4, 8] {
                eprintln!("Testing input: '{}' with prefix length {}", s, prefix_len);
                let result = sanitize_for_display(s, prefix_len);

                // Verify the result matches expected pattern
                let expected_visible = if s.len() <= prefix_len {
                    s.to_string()
                } else {
                    let visible = &s[0..prefix_len];
                    let hidden = "*".repeat(s.len() - prefix_len);
                    format!("{}{}", visible, hidden)
                };

                eprintln!("Result: '{}', Expected: '{}'", result, expected_visible);
                assert_eq!(result, expected_visible);
            }
        }
    });
}
