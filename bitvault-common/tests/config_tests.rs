use anyhow::Result;
use bitvault_common::config::{self, Config};
use std::fs;
use tempfile::TempDir;

// Helper function to create a temporary directory and config file for testing
fn setup_test_config() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("config.toml");
    let config_path_str = config_path.to_string_lossy().to_string();

    // Create a minimal valid config
    let config_content = r#"
        [wallet]
        network = "Bitcoin"
        fiat_currency = "USD"
        fee_level = "medium"
        prevent_address_reuse = true
        coin_selection = "privacy"
        
        [network]
        timeout_seconds = 30
        use_tor = false
        max_connections = 8
        
        [ipc]
        port = 8999
        max_message_size_mb = 1
        timeout_seconds = 15
        
        [ui]
        dark_mode = true
        language = "en"
        display_as_btc = true
        show_fiat = true
        
        [storage]
        database_path = "/home/user/.bitvault/db"
        encrypted = true
        backup_directory = "/home/user/.bitvault/backup"
        tx_retention_days = 0
    "#;

    // Write the config file and verify it was created successfully
    fs::write(&config_path, config_content).expect("Failed to write test config");
    assert!(config_path.exists(), "Failed to create config file");

    (temp_dir, config_path_str)
}

#[test]
fn test_ensure_config_exists_creates_default() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("non_existent_config.toml");

    // The file doesn't exist yet, so this should create it with defaults
    let result = config::ensure_config_exists(&config_path);
    assert!(
        result.is_ok(),
        "Failed to create default config: {:?}",
        result
    );

    // Verify the file was created
    assert!(config_path.exists(), "Config file was not created");

    // Load the created config and verify it has default values
    let config = Config::load(config_path.to_str().unwrap())?;
    assert_eq!(
        config.wallet.network, "Bitcoin",
        "Default network should be Bitcoin"
    );
    assert_eq!(
        config.wallet.fiat_currency, "USD",
        "Default fiat currency should be USD"
    );
    assert_eq!(
        config.ui.dark_mode, true,
        "Default UI theme should be dark mode"
    );
    Ok(())
}

#[test]
fn test_load_valid_config() -> Result<()> {
    let (_temp_dir, config_path) = setup_test_config();

    // Load the config
    let config = Config::load(&config_path)?;

    // Verify values were loaded correctly
    assert_eq!(config.wallet.network, "Bitcoin");
    assert_eq!(config.wallet.fiat_currency, "USD");
    assert_eq!(config.wallet.fee_level, "medium");
    assert_eq!(config.wallet.prevent_address_reuse, true);
    assert_eq!(config.wallet.coin_selection, "privacy");

    assert_eq!(config.network.timeout_seconds, 30);
    assert_eq!(config.network.use_tor, false);
    assert_eq!(config.network.max_connections, 8);

    assert_eq!(config.ipc.port, 8999);
    assert_eq!(config.ipc.max_message_size_mb, 1);
    assert_eq!(config.ipc.timeout_seconds, 15);

    assert_eq!(config.ui.dark_mode, true);
    assert_eq!(config.ui.language, "en");
    assert_eq!(config.ui.display_as_btc, true);
    assert_eq!(config.ui.show_fiat, true);

    assert_eq!(config.storage.database_path, "/home/user/.bitvault/db");
    assert_eq!(config.storage.encrypted, true);
    assert_eq!(
        config.storage.backup_directory,
        "/home/user/.bitvault/backup"
    );
    assert_eq!(config.storage.tx_retention_days, 0);

    Ok(())
}

#[test]
fn test_load_invalid_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("invalid_config.toml");
    let config_path_str = config_path.to_string_lossy().to_string();

    // Create an invalid config (invalid toml syntax)
    let invalid_content = r#"
        [wallet]
        network = "Bitcoin
        # Missing closing quote causes parsing error
    "#;

    fs::write(&config_path, invalid_content).expect("Failed to write invalid config");
    assert!(config_path.exists(), "Failed to create invalid config file");

    // Attempt to load the invalid config
    let result = Config::load(&config_path_str);
    assert!(result.is_err(), "Loading invalid config should fail");

    // Check that the error message indicates a parsing problem
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("parse") || err.contains("missing") || err.contains("TOML"),
        "Error should indicate parsing issue: {}",
        err
    );
}

#[test]
fn test_config_validation() -> Result<()> {
    let (_temp_dir, config_path) = setup_test_config();

    // Load valid config
    let mut config = Config::load(&config_path)?;

    // Validate the config - should be valid
    let validation_result = config.validate();
    assert!(
        validation_result.is_ok(),
        "Config should be valid: {:?}",
        validation_result
    );

    // Make the config invalid by setting an invalid port
    config.ipc.port = 0; // Invalid port

    // Save the invalid config
    let invalid_path = _temp_dir.path().join("invalid_port_config.toml");
    let invalid_path_str = invalid_path.to_string_lossy().to_string();
    config.save(&invalid_path_str, None)?;

    // Reload and validate - should fail validation
    let invalid_config = Config::load(&invalid_path_str)?;
    let validation_result = invalid_config.validate();
    assert!(
        validation_result.is_err(),
        "Invalid config should fail validation"
    );

    // Check that the error message mentions the port
    let err = validation_result.unwrap_err().to_string();
    assert!(
        err.contains("port"),
        "Error should mention invalid port: {}",
        err
    );

    Ok(())
}

#[test]
fn test_config_modification_and_save() -> Result<()> {
    let (_temp_dir, config_path) = setup_test_config();

    // Load the original config
    let mut config = Config::load(&config_path)?;

    // Modify some values
    config.wallet.fiat_currency = "EUR".to_string();
    config.ui.dark_mode = false;
    config.network.use_tor = true;

    // Save the modified config to a new file
    let modified_path = _temp_dir.path().join("modified_config.toml");
    let modified_path_str = modified_path.to_string_lossy().to_string();
    config.save(&modified_path_str, None)?;

    // Load the modified config and verify changes were saved
    let modified_config = Config::load(&modified_path_str)?;
    assert_eq!(
        modified_config.wallet.fiat_currency, "EUR",
        "Fiat currency should be changed to EUR"
    );
    assert_eq!(
        modified_config.ui.dark_mode, false,
        "Dark mode should be turned off"
    );
    assert_eq!(
        modified_config.network.use_tor, true,
        "Tor should be enabled"
    );

    // Original values that weren't changed should remain the same
    assert_eq!(
        modified_config.wallet.network, "Bitcoin",
        "Network should still be Bitcoin"
    );
    assert_eq!(
        modified_config.ipc.port, 8999,
        "IPC port should be unchanged"
    );
    Ok(())
}

#[test]
fn test_default_values() -> Result<()> {
    // Create a default configuration
    let config = Config::default();

    // Verify default values are set correctly
    assert_eq!(
        config.wallet.network, "Bitcoin",
        "Default network should be Bitcoin"
    );
    assert_eq!(
        config.wallet.fiat_currency, "USD",
        "Default fiat currency should be USD"
    );
    assert_eq!(
        config.wallet.fee_level, "medium",
        "Default fee level should be medium"
    );
    assert_eq!(
        config.wallet.prevent_address_reuse, true,
        "Address reuse prevention should be enabled by default"
    );
    assert_eq!(
        config.wallet.coin_selection, "privacy",
        "Default coin selection strategy should be privacy"
    );

    assert_eq!(
        config.network.timeout_seconds, 30,
        "Default timeout should be 30 seconds"
    );
    assert_eq!(
        config.network.use_tor, false,
        "Tor should be disabled by default"
    );
    assert_eq!(
        config.network.max_connections, 8,
        "Default max connections should be 8"
    );

    assert_eq!(config.ipc.port, 8999, "Default IPC port should be 8999");
    assert_eq!(
        config.ipc.max_message_size_mb, 1,
        "Default max message size should be 1 MB"
    );
    assert_eq!(
        config.ipc.timeout_seconds, 15,
        "Default IPC timeout should be 15 seconds"
    );

    assert_eq!(
        config.ui.dark_mode, true,
        "Dark mode should be enabled by default"
    );
    assert_eq!(
        config.ui.language, "en",
        "Default language should be English"
    );
    assert_eq!(
        config.ui.display_as_btc, true,
        "Display as BTC should be enabled by default"
    );
    assert_eq!(
        config.ui.show_fiat, true,
        "Show fiat should be enabled by default"
    );

    assert!(
        config.storage.database_path.contains(".bitvault/db"),
        "Database path should be in .bitvault/db"
    );
    assert_eq!(
        config.storage.encrypted, true,
        "Storage should be encrypted by default"
    );
    assert!(
        config.storage.backup_directory.contains("backup"),
        "Backup directory should contain 'backup'"
    );
    assert_eq!(
        config.storage.tx_retention_days, 0,
        "Default tx retention should be forever (0)"
    );

    // Validate default config should be valid
    let validation_result = config.validate();
    assert!(
        validation_result.is_ok(),
        "Default config should be valid: {:?}",
        validation_result
    );
    Ok(())
}

#[test]
fn test_accessing_config_sections() -> Result<()> {
    let (_temp_dir, config_path) = setup_test_config();

    // Verify the config file exists
    let config_file_path = _temp_dir.path().join("config.toml");
    assert!(config_file_path.exists(), "Config file does not exist");

    // Load the config and verify it exists
    let config = Config::load(&config_path)?;

    // Access wallet settings
    let wallet_settings = &config.wallet;
    assert_eq!(wallet_settings.network, "Bitcoin");
    assert_eq!(wallet_settings.fiat_currency, "USD");
    assert_eq!(wallet_settings.fee_level, "medium");

    // Access network settings
    let network_settings = &config.network;
    assert_eq!(network_settings.timeout_seconds, 30);
    assert_eq!(network_settings.use_tor, false);
    assert_eq!(network_settings.max_connections, 8);

    // Access UI settings
    let ui_settings = &config.ui;
    assert_eq!(ui_settings.dark_mode, true);
    assert_eq!(ui_settings.language, "en");
    assert_eq!(ui_settings.display_as_btc, true);

    Ok(())
}

#[test]
fn test_security_critical_settings() -> Result<()> {
    // This test specifically focuses on security-critical settings
    // to ensure they have appropriate defaults and validation

    let (_temp_dir, config_path) = setup_test_config();

    // Verify the config file exists
    assert!(
        std::path::Path::new(&config_path).exists(),
        "Config file does not exist"
    );

    let config = Config::load(&config_path)?;

    // Verify security-critical storage settings
    assert_eq!(
        config.storage.encrypted, true,
        "Storage should be encrypted by default for security"
    );

    // Verify network timeout is reasonable
    assert!(
        config.network.timeout_seconds > 0 && config.network.timeout_seconds <= 300,
        "Network timeout should be within reasonable bounds"
    );

    // Verify IPC message size is reasonable
    assert!(
        config.ipc.max_message_size_mb > 0 && config.ipc.max_message_size_mb <= 100,
        "IPC message size should be within reasonable bounds"
    );

    Ok(())
}
