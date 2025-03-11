use bitvault_common::logging::{self, LogConfig, LogLevel};
use std::sync::Once;

// Ensure logging is only initialized once across all tests
static INIT: Once = Once::new();

fn setup_logging() {
    INIT.call_once(|| {
        let config = LogConfig {
            level: LogLevel::Debug,
            log_file: None,
            include_timestamps: true,
            include_source_location: true,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            console_logging: true,
            json_format: false,
        };

        logging::init(&config).expect("Failed to initialize logging");
    });
}

#[test]
fn test_logging_initialization() {
    setup_logging();

    // Test changing log level
    logging::set_log_level(LogLevel::Info);
    logging::set_log_level(LogLevel::Debug);
    logging::set_log_level(LogLevel::Trace);

    // Just make sure the function runs without panicking
    assert!(true);
}

#[test]
fn test_log_functions() {
    setup_logging();

    // Test various log functions - we're just checking they don't panic
    logging::log_security(
        LogLevel::Info,
        "Test security log",
        Some(serde_json::json!({
            "user_id": "12345",
            "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
        })),
    );

    logging::log_core(
        LogLevel::Info,
        "Test core log",
        Some(serde_json::json!({
            "wallet_id": "w12345",
            "txid": "3a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b"
        })),
    );

    logging::log_network(
        LogLevel::Info,
        "Test network log",
        Some(serde_json::json!({
            "peer": "127.0.0.1:8333",
            "status": "connected"
        })),
    );

    logging::log_transaction(
        LogLevel::Info,
        "Test transaction log",
        Some(serde_json::json!({
            "txid": "3a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b",
            "amount": "0.1"
        })),
    );

    logging::log_ui(
        LogLevel::Info,
        "Test UI log",
        Some(serde_json::json!({
            "screen": "send",
            "action": "button_click"
        })),
    );

    logging::log_storage(
        LogLevel::Info,
        "Test storage log",
        Some(serde_json::json!({
            "operation": "save",
            "status": "success"
        })),
    );

    // Just make sure the function runs without panicking
    assert!(true);
}

#[test]
fn test_default_config() {
    let config = LogConfig::default();

    // Check default values
    assert_eq!(config.level, LogLevel::Info);
    assert!(config.log_file.is_none());
    assert!(config.include_timestamps);
    assert!(config.include_source_location);
    assert!(config.console_logging);
    assert!(!config.json_format);

    // Make sure the max_file_size is reasonable
    assert!(config.max_file_size > 0);
}
