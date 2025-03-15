//! Configuration Manager for BitVault.
//!
//! This module provides a robust configuration management system with:
//! - Event notifications for configuration changes
//! - Support for user profiles and configuration presets
//! - Validation rules for configuration changes
//! - Migration support for configuration format changes
//!
//! ## Security Boundary Documentation
//!
//! This module implements a non-critical security boundary between user preferences
//! and core wallet functionality. The configuration data is considered non-sensitive
//! but must be protected from tampering.
//!
//! ### Security Boundaries:
//!
//! - **Config/Core Boundary**: Separates user-configurable options from core wallet operations
//!   - All configuration values must be validated before use in security-sensitive operations
//!   - Configuration changes must be audited via event system
//!
//! ## Security Considerations
//!
//! - No security-critical information should be stored in this configuration
//! - The configuration module does not handle encryption/decryption
//! - Sensitive values are validated to ensure they meet security requirements
//! - Configuration changes should be logged via the event system
//! - All configuration values must be validated before use in security-sensitive functions
//! - Configuration files must be protected from unauthorized modification

use std::collections::HashMap;
use std::fs::{self, create_dir_all};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::events::{MessageBus, EventType, MessagePriority};
use crate::config::Config;
use crate::logging::{log_security, LogLevel};

/// Error types specific to configuration management
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration section: {0}")]
    InvalidSection(String),
    
    #[error("Invalid configuration key: {0}")]
    InvalidKey(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Configuration profile not found: {0}")]
    ProfileNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Migration error: {0}")]
    MigrationError(String),
}

/// Configuration validator function type
type ValidatorFn = Box<dyn Fn(&str, &str, &str) -> Result<(), String> + Send + Sync>;

/// Configuration change event details
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// Section of the configuration that changed
    pub section: String,
    /// Key within the section that changed
    pub key: String,
    /// Previous value (may be null)
    pub old_value: Option<String>,
    /// New value
    pub new_value: String,
}

/// Configuration Manager for BitVault
///
/// Provides centralized management of configuration with:
/// - Event notifications when configuration changes
/// - Validation rules for configuration changes
/// - Profile management
/// - Migration support for configuration format changes
pub struct ConfigManager {
    /// The current configuration
    config: Arc<RwLock<Config>>,
    
    /// Path to the configuration file
    config_path: PathBuf,
    
    /// Directory for profile storage
    profiles_dir: PathBuf,
    
    /// Message bus for notifications
    message_bus: Option<Arc<MessageBus>>,
    
    /// Validators for configuration changes
    validators: HashMap<String, HashMap<String, ValidatorFn>>,
    
    /// Migration handlers for version upgrades
    migrations: Vec<Box<dyn Fn(&mut Config) -> Result<(), String> + Send + Sync>>,
}

impl ConfigManager {
    /// Create a new ConfigManager with the specified configuration and optional message bus
    pub fn new(
        config: Config, 
        config_path: PathBuf,
        message_bus: Option<Arc<MessageBus>>
    ) -> Self {
        // Determine profiles directory
        let profiles_dir = if let Some(parent) = config_path.parent() {
            let mut dir = parent.to_path_buf();
            dir.push("profiles");
            dir
        } else {
            PathBuf::from("./profiles")
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            profiles_dir,
            message_bus,
            validators: HashMap::new(),
            migrations: Vec::new(),
        }
    }

    /// Create a new ConfigManager by loading configuration from the specified path
    pub fn from_path(
        config_path: impl AsRef<Path>, 
        message_bus: Option<Arc<MessageBus>>
    ) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();
        
        // Ensure the config file exists
        crate::config::ensure_config_exists(&config_path)?;
        
        // Load the configuration
        let config = Config::load(config_path.to_str().unwrap())?;
        
        Ok(Self::new(config, config_path, message_bus))
    }

    /// Get a clone of the current configuration
    pub fn get_config(&self) -> Result<Config> {
        let config = self.config.read().map_err(|_| {
            anyhow!("Failed to acquire read lock on configuration")
        })?;
        
        Ok(config.clone())
    }

    /// Update a configuration value in the specified section
    pub fn update_value(&self, section: &str, key: &str, value: &str) -> Result<(), ConfigError> {
        // Check if we have a validator for this section and key
        self.validate_change(section, key, value)?;
        
        // Apply the change, getting the old value
        let old_value = self.apply_change(section, key, value)?;
        
        // Save the updated configuration
        self.save_config()?;
        
        // Notify about the change
        self.notify_change(section, key, old_value.as_deref(), value);
        
        Ok(())
    }

    /// Update multiple configuration values at once
    pub fn update_values(
        &self, 
        changes: &[(&str, &str, &str)]
    ) -> Result<(), ConfigError> {
        // Validate all changes first
        for (section, key, value) in changes {
            self.validate_change(section, key, value)?;
        }
        
        // Apply all changes and collect events to notify
        let mut change_events = Vec::new();
        
        for (section, key, value) in changes {
            let old_value = self.apply_change(section, key, value)?;
            change_events.push((section.to_string(), key.to_string(), old_value, value.to_string()));
        }
        
        // Save the updated configuration
        self.save_config()?;
        
        // Notify about all changes
        for (section, key, old_value, new_value) in change_events {
            self.notify_change(&section, &key, old_value.as_deref(), &new_value);
        }
        
        Ok(())
    }

    /// Add a validator function for a specific configuration section and key
    pub fn add_validator<F>(&mut self, section: &str, key: &str, validator: F) 
    where
        F: Fn(&str, &str, &str) -> Result<(), String> + Send + Sync + 'static
    {
        let section_validators = self.validators
            .entry(section.to_string())
            .or_insert_with(HashMap::new);
            
        section_validators.insert(
            key.to_string(), 
            Box::new(validator)
        );
    }

    /// Add a migration function to handle configuration format changes
    pub fn add_migration<F>(&mut self, migration: F)
    where
        F: Fn(&mut Config) -> Result<(), String> + Send + Sync + 'static
    {
        self.migrations.push(Box::new(migration));
    }

    /// Save the current configuration to the specified profile
    pub fn save_profile(&self, profile_name: &str) -> Result<(), ConfigError> {
        // Create the profiles directory if it doesn't exist
        if !self.profiles_dir.exists() {
            create_dir_all(&self.profiles_dir).map_err(ConfigError::IoError)?;
        }
        
        // Build the profile path
        let mut profile_path = self.profiles_dir.clone();
        profile_path.push(format!("{}.toml", profile_name));
        
        // Get the current configuration
        let config = self.config.read().map_err(|_| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to acquire read lock on configuration"
            ))
        })?;
        
        // Serialize the configuration
        let content = toml::to_string_pretty(&*config).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })?;
        
        // Write the profile file
        fs::write(&profile_path, content).map_err(ConfigError::IoError)?;
        
        // Log the profile save
        log_security(
            LogLevel::Info, 
            &format!("Saved configuration profile: {}", profile_name),
            None
        );
        
        Ok(())
    }

    /// Load a configuration profile
    pub fn load_profile(&self, profile_name: &str) -> Result<(), ConfigError> {
        // Build the profile path
        let mut profile_path = self.profiles_dir.clone();
        profile_path.push(format!("{}.toml", profile_name));
        
        // Check if the profile exists
        if !profile_path.exists() {
            return Err(ConfigError::ProfileNotFound(profile_name.to_string()));
        }
        
        // Load the profile configuration
        let profile_config = Config::load(profile_path.to_str().unwrap())
            .map_err(|e| ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to load profile: {}", e)
            )))?;
        
        // Validate the profile configuration
        profile_config.validate().map_err(|e| {
            ConfigError::ValidationFailed(format!("Profile validation failed: {}", e))
        })?;
        
        // Update the current configuration
        let mut config = self.config.write().map_err(|_| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to acquire write lock on configuration"
            ))
        })?;
        
        *config = profile_config;
        
        // Save the current configuration to persist the profile
        drop(config); // Release the write lock
        self.save_config()?;
        
        // Log the profile load
        log_security(
            LogLevel::Info, 
            &format!("Loaded configuration profile: {}", profile_name),
            None
        );
        
        // Notify about the profile change
        if let Some(ref message_bus) = self.message_bus {
            let event = serde_json::json!({
                "type": "profile_change",
                "profile": profile_name
            }).to_string();
            
            message_bus.publish(
                EventType::Settings,
                &event,
                MessagePriority::Medium
            );
        }
        
        Ok(())
    }

    /// List available configuration profiles
    pub fn list_profiles(&self) -> Result<Vec<String>, ConfigError> {
        // Create the profiles directory if it doesn't exist
        if !self.profiles_dir.exists() {
            return Ok(Vec::new());
        }
        
        // List the profile files
        let entries = fs::read_dir(&self.profiles_dir).map_err(ConfigError::IoError)?;
        
        let mut profiles = Vec::new();
        for entry in entries {
            let entry = entry.map_err(ConfigError::IoError)?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "toml") {
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        profiles.push(name.to_string());
                    }
                }
            }
        }
        
        Ok(profiles)
    }

    /// Apply migrations to update the configuration format if needed
    pub fn apply_migrations(&self) -> Result<bool, ConfigError> {
        if self.migrations.is_empty() {
            return Ok(false);
        }
        
        let mut config = self.config.write().map_err(|_| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to acquire write lock on configuration"
            ))
        })?;
        
        let mut migrated = false;
        
        // Apply each migration in order
        for migration in &self.migrations {
            if let Err(e) = migration(&mut config) {
                log_security(
                    LogLevel::Error, 
                    &format!("Configuration migration failed: {}", e),
                    None
                );
                return Err(ConfigError::MigrationError(e));
            }
            migrated = true;
        }
        
        // If migrations were applied, save the updated configuration
        if migrated {
            drop(config); // Release the write lock
            self.save_config()?;
            
            log_security(
                LogLevel::Info, 
                "Applied configuration migrations successfully",
                None
            );
        }
        
        Ok(migrated)
    }

    /// Export configuration to JSON format
    pub fn export_as_json(&self) -> Result<String, ConfigError> {
        let config = self.config.read().map_err(|_| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to acquire read lock on configuration"
            ))
        })?;
        
        serde_json::to_string_pretty(&*config).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })
    }

    /// Import configuration from JSON format
    pub fn import_from_json(&self, json: &str) -> Result<(), ConfigError> {
        // Parse the JSON
        let imported_config: Config = serde_json::from_str(json).map_err(|e| {
            ConfigError::SerializationError(format!("Invalid JSON format: {}", e))
        })?;
        
        // Validate the imported configuration
        imported_config.validate().map_err(|e| {
            ConfigError::ValidationFailed(format!("Imported configuration validation failed: {}", e))
        })?;
        
        // Update the current configuration
        let mut config = self.config.write().map_err(|_| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to acquire write lock on configuration"
            ))
        })?;
        
        *config = imported_config;
        
        // Save the updated configuration
        drop(config); // Release the write lock
        self.save_config()?;
        
        // Log the import
        log_security(
            LogLevel::Info, 
            "Imported configuration from JSON",
            None
        );
        
        // Notify about the configuration change
        if let Some(ref message_bus) = self.message_bus {
            let event = serde_json::json!({
                "type": "config_import"
            }).to_string();
            
            message_bus.publish(
                EventType::Settings,
                &event,
                MessagePriority::Medium
            );
        }
        
        Ok(())
    }

    /// Create a default configuration profile
    pub fn create_default_profile(&self) -> Result<(), ConfigError> {
        self.save_profile("default")
    }

    /// Reset configuration to default values
    pub fn reset_to_defaults(&self) -> Result<(), ConfigError> {
        // Create a new default configuration
        let default_config = Config::default();
        
        // Update the current configuration
        let mut config = self.config.write().map_err(|_| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to acquire write lock on configuration"
            ))
        })?;
        
        *config = default_config;
        
        // Save the updated configuration
        drop(config); // Release the write lock
        self.save_config()?;
        
        // Log the reset
        log_security(
            LogLevel::Info, 
            "Reset configuration to default values",
            None
        );
        
        // Notify about the configuration change
        if let Some(ref message_bus) = self.message_bus {
            let event = serde_json::json!({
                "type": "config_reset"
            }).to_string();
            
            message_bus.publish(
                EventType::Settings,
                &event,
                MessagePriority::Medium
            );
        }
        
        Ok(())
    }

    /// Add default validators for security-critical settings
    pub fn add_default_validators(&mut self) {
        // Wallet validators
        self.add_validator(
            "wallet", 
            "fee_level", 
            ConfigValidators::one_of(vec!["low", "medium", "high"])
        );
        
        self.add_validator(
            "wallet", 
            "network", 
            ConfigValidators::one_of(vec!["Bitcoin", "Testnet", "Regtest"])
        );
        
        // Network validators
        self.add_validator(
            "network", 
            "timeout_seconds", 
            ConfigValidators::numeric_range(5.0, 300.0)
        );
        
        self.add_validator(
            "network", 
            "max_connections", 
            ConfigValidators::numeric_range(1.0, 20.0)
        );
        
        self.add_validator(
            "network", 
            "use_tor", 
            ConfigValidators::boolean()
        );
        
        // IPC validators
        self.add_validator(
            "ipc", 
            "port", 
            ConfigValidators::numeric_range(1024.0, 65535.0)
        );
        
        self.add_validator(
            "ipc", 
            "max_message_size_mb", 
            ConfigValidators::numeric_range(1.0, 100.0)
        );
        
        self.add_validator(
            "ipc", 
            "timeout_seconds", 
            ConfigValidators::numeric_range(1.0, 120.0)
        );
        
        // UI validators
        self.add_validator(
            "ui", 
            "dark_mode", 
            ConfigValidators::boolean()
        );
        
        self.add_validator(
            "ui", 
            "language", 
            ConfigValidators::non_empty()
        );
        
        // Storage validators
        self.add_validator(
            "storage", 
            "encrypted", 
            ConfigValidators::boolean()
        );
    }

    /// Create a new ConfigManager with default validators
    pub fn new_with_validators(
        config: Config, 
        config_path: PathBuf,
        message_bus: Option<Arc<MessageBus>>
    ) -> Self {
        let mut manager = Self::new(config, config_path, message_bus);
        manager.add_default_validators();
        manager
    }

    /// Create a new ConfigManager by loading configuration from the specified path with default validators
    pub fn from_path_with_validators(
        config_path: impl AsRef<Path>, 
        message_bus: Option<Arc<MessageBus>>
    ) -> Result<Self> {
        let mut manager = Self::from_path(config_path, message_bus)?;
        manager.add_default_validators();
        Ok(manager)
    }

    /// Create common pre-defined profiles for different use cases
    pub fn create_common_profiles(&self) -> Result<(), ConfigError> {
        // Create default profile first
        self.create_default_profile()?;
        
        // High Security Profile
        self.save_high_security_profile()?;
        
        // Privacy-focused Profile
        self.save_privacy_profile()?;
        
        // Performance-optimized Profile
        self.save_performance_profile()?;
        
        // Testnet Profile
        self.save_testnet_profile()?;
        
        // Mobile-optimized Profile
        self.save_mobile_profile()?;
        
        Ok(())
    }

    /// Create and save a high security profile
    pub fn save_high_security_profile(&self) -> Result<(), ConfigError> {
        // Start with current config
        let mut high_security_config = self.get_config().map_err(|e| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to get config: {}", e)
            ))
        })?;
        
        // Modify for high security
        high_security_config.wallet.prevent_address_reuse = true;
        high_security_config.wallet.coin_selection = "privacy".to_string();
        high_security_config.network.use_tor = true;
        high_security_config.network.trusted_peers = Vec::new(); // Only connect to trusted peers
        high_security_config.network.max_connections = 4; // Limit connections for reduced attack surface
        high_security_config.storage.encrypted = true;
        high_security_config.ipc.timeout_seconds = 5; // Short timeouts to reduce risk
        
        // Create a temporary ConfigManager with the high security config
        let temp_dir = PathBuf::from("/tmp"); // This is just for initialization, we don't save to this path
        let temp_manager = ConfigManager::new(high_security_config, temp_dir, None);
        
        // Save using our profile storage logic
        temp_manager.save_profile("high_security")?;
        
        // Copy the profile file to our profiles directory
        let src_path = temp_manager.profiles_dir.join("high_security.toml");
        let dst_path = self.profiles_dir.join("high_security.toml");
        
        if src_path.exists() {
            // Create profiles directory if it doesn't exist
            if !self.profiles_dir.exists() {
                create_dir_all(&self.profiles_dir).map_err(ConfigError::IoError)?;
            }
            
            // Read the content from the temporary file
            let content = fs::read_to_string(&src_path).map_err(ConfigError::IoError)?;
            
            // Write to our profile path
            fs::write(&dst_path, content).map_err(ConfigError::IoError)?;
        }
        
        // Log the profile creation
        log_security(
            LogLevel::Info, 
            "Created high security profile",
            None
        );
        
        Ok(())
    }

    /// Create and save a privacy-focused profile
    pub fn save_privacy_profile(&self) -> Result<(), ConfigError> {
        // Start with current config
        let mut privacy_config = self.get_config().map_err(|e| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to get config: {}", e)
            ))
        })?;
        
        // Modify for privacy
        privacy_config.wallet.prevent_address_reuse = true;
        privacy_config.wallet.coin_selection = "privacy".to_string();
        privacy_config.network.use_tor = true;
        privacy_config.ui.show_fiat = false; // Don't show fiat for better privacy
        
        // Update the profile directly in our profiles directory
        let profile_path = self.profiles_dir.join("privacy.toml");
        
        // Create profiles directory if it doesn't exist
        if !self.profiles_dir.exists() {
            create_dir_all(&self.profiles_dir).map_err(ConfigError::IoError)?;
        }
        
        // Serialize the configuration
        let content = toml::to_string_pretty(&privacy_config).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })?;
        
        // Write the profile file
        fs::write(&profile_path, content).map_err(ConfigError::IoError)?;
        
        // Log the profile creation
        log_security(
            LogLevel::Info, 
            "Created privacy profile",
            None
        );
        
        Ok(())
    }

    /// Create and save a performance-optimized profile
    pub fn save_performance_profile(&self) -> Result<(), ConfigError> {
        // Start with current config
        let mut perf_config = self.get_config().map_err(|e| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to get config: {}", e)
            ))
        })?;
        
        // Modify for performance
        perf_config.wallet.coin_selection = "economical".to_string(); // Use economical coin selection
        perf_config.wallet.fee_level = "low".to_string(); // Use lower fees
        perf_config.network.max_connections = 12; // More connections for better propagation
        perf_config.network.use_tor = false; // Don't use Tor for better performance
        perf_config.ipc.max_message_size_mb = 2; // Larger message size for fewer roundtrips
        
        // Update the profile directly in our profiles directory
        let profile_path = self.profiles_dir.join("performance.toml");
        
        // Create profiles directory if it doesn't exist
        if !self.profiles_dir.exists() {
            create_dir_all(&self.profiles_dir).map_err(ConfigError::IoError)?;
        }
        
        // Serialize the configuration
        let content = toml::to_string_pretty(&perf_config).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })?;
        
        // Write the profile file
        fs::write(&profile_path, content).map_err(ConfigError::IoError)?;
        
        // Log the profile creation
        log_security(
            LogLevel::Info, 
            "Created performance profile",
            None
        );
        
        Ok(())
    }

    /// Create and save a testnet profile
    pub fn save_testnet_profile(&self) -> Result<(), ConfigError> {
        // Start with current config
        let mut testnet_config = self.get_config().map_err(|e| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to get config: {}", e)
            ))
        })?;
        
        // Modify for testnet
        testnet_config.wallet.network = "Testnet".to_string();
        testnet_config.wallet.fee_level = "low".to_string(); // Lower fees on testnet
        testnet_config.storage.encrypted = true; // Still keep encryption
        
        // Update the profile directly in our profiles directory
        let profile_path = self.profiles_dir.join("testnet.toml");
        
        // Create profiles directory if it doesn't exist
        if !self.profiles_dir.exists() {
            create_dir_all(&self.profiles_dir).map_err(ConfigError::IoError)?;
        }
        
        // Serialize the configuration
        let content = toml::to_string_pretty(&testnet_config).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })?;
        
        // Write the profile file
        fs::write(&profile_path, content).map_err(ConfigError::IoError)?;
        
        // Log the profile creation
        log_security(
            LogLevel::Info, 
            "Created testnet profile",
            None
        );
        
        Ok(())
    }

    /// Create and save a mobile-optimized profile
    pub fn save_mobile_profile(&self) -> Result<(), ConfigError> {
        // Start with current config
        let mut mobile_config = self.get_config().map_err(|e| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to get config: {}", e)
            ))
        })?;
        
        // Modify for mobile
        mobile_config.network.max_connections = 4; // Fewer connections to save battery
        mobile_config.ipc.timeout_seconds = 30; // Longer timeouts for slower mobile networks
        mobile_config.ipc.max_message_size_mb = 1; // Smaller message size for reduced memory usage
        mobile_config.network.timeout_seconds = 60; // Longer network timeouts
        
        // Update the profile directly in our profiles directory
        let profile_path = self.profiles_dir.join("mobile.toml");
        
        // Create profiles directory if it doesn't exist
        if !self.profiles_dir.exists() {
            create_dir_all(&self.profiles_dir).map_err(ConfigError::IoError)?;
        }
        
        // Serialize the configuration
        let content = toml::to_string_pretty(&mobile_config).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })?;
        
        // Write the profile file
        fs::write(&profile_path, content).map_err(ConfigError::IoError)?;
        
        // Log the profile creation
        log_security(
            LogLevel::Info, 
            "Created mobile profile",
            None
        );
        
        Ok(())
    }

    // Private helper methods

    /// Save the current configuration to file
    fn save_config(&self) -> Result<(), ConfigError> {
        let config = self.config.read().map_err(|_| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to acquire read lock on configuration"
            ))
        })?;
        
        // Serialize the configuration
        let content = toml::to_string_pretty(&*config).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                create_dir_all(parent).map_err(ConfigError::IoError)?;
            }
        }
        
        // Write the configuration file
        fs::write(&self.config_path, content).map_err(ConfigError::IoError)?;
        
        Ok(())
    }

    /// Validate a configuration change
    fn validate_change(&self, section: &str, key: &str, value: &str) -> Result<(), ConfigError> {
        // Check if we have a validator for this section and key
        if let Some(section_validators) = self.validators.get(section) {
            if let Some(validator) = section_validators.get(key) {
                if let Err(e) = validator(section, key, value) {
                    return Err(ConfigError::ValidationFailed(e));
                }
            }
        }
        
        Ok(())
    }

    /// Apply a configuration change and return the old value
    fn apply_change(&self, section: &str, key: &str, value: &str) -> Result<Option<String>, ConfigError> {
        let mut config = self.config.write().map_err(|_| {
            ConfigError::IoError(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to acquire write lock on configuration"
            ))
        })?;
        
        // Convert to a mutable Value for easier manipulation
        let mut config_value = serde_json::to_value(&*config).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })?;
        
        // Get a mutable reference to the section
        let section_value = config_value.get_mut(section).ok_or_else(|| {
            ConfigError::InvalidSection(section.to_string())
        })?;
        
        // Check if the key exists in the section
        if !section_value.get(key).is_some() {
            return Err(ConfigError::InvalidKey(key.to_string()));
        }
        
        // Get the old value as a string, if possible
        let old_value = section_value.get(key).and_then(|v| {
            if v.is_string() {
                v.as_str().map(|s| s.to_string())
            } else {
                // Convert non-string values to string representation
                Some(v.to_string())
            }
        });
        
        // Update the value
        let value_json: Value = match value.parse() {
            Ok(v) => v,
            Err(_) => {
                // If it's not valid JSON, treat it as a string
                Value::String(value.to_string())
            }
        };
        
        section_value[key] = value_json;
        
        // Convert back to Config
        *config = serde_json::from_value(config_value).map_err(|e| {
            ConfigError::SerializationError(e.to_string())
        })?;
        
        // Validate the updated configuration
        config.validate().map_err(|e| {
            ConfigError::ValidationFailed(format!("Validation failed after update: {}", e))
        })?;
        
        Ok(old_value)
    }

    /// Notify about a configuration change
    fn notify_change(&self, section: &str, key: &str, old_value: Option<&str>, new_value: &str) {
        // Log the change
        let change_info = format!(
            "Configuration changed: [{}.{}] {} -> {}", 
            section, key, 
            old_value.unwrap_or("null"), 
            new_value
        );
        
        log_security(LogLevel::Info, &change_info, None);
        
        // Send event if message bus is available
        if let Some(ref message_bus) = self.message_bus {
            let event = ConfigChangeEvent {
                section: section.to_string(),
                key: key.to_string(),
                old_value: old_value.map(|s| s.to_string()),
                new_value: new_value.to_string(),
            };
            
            let event_json = serde_json::to_string(&event).unwrap_or_else(|_| {
                format!(
                    r#"{{"section":"{}","key":"{}","old_value":{},"new_value":"{}"}}"#,
                    section, key, 
                    old_value.map_or("null".to_string(), |v| format!(r#""{}""#, v)),
                    new_value
                )
            });
            
            message_bus.publish(
                EventType::Settings,
                &event_json,
                MessagePriority::Medium
            );
        }
    }
}

/// Helper structure for creating built-in validators
pub struct ConfigValidators;

impl ConfigValidators {
    /// Create a validator that ensures a value is one of the allowed options
    pub fn one_of(allowed_values: Vec<&str>) -> impl Fn(&str, &str, &str) -> Result<(), String> {
        let allowed = allowed_values.into_iter().map(String::from).collect::<Vec<_>>();
        move |_, _, value| {
            if allowed.contains(&value.to_string()) {
                Ok(())
            } else {
                Err(format!("Value must be one of: {}", allowed.join(", ")))
            }
        }
    }
    
    /// Create a validator that ensures a numeric value is within a range
    pub fn numeric_range(min: f64, max: f64) -> impl Fn(&str, &str, &str) -> Result<(), String> {
        move |_, _, value| {
            let num = value.parse::<f64>().map_err(|_| {
                format!("Value must be a number, got: {}", value)
            })?;
            
            if num >= min && num <= max {
                Ok(())
            } else {
                Err(format!("Value must be between {} and {}", min, max))
            }
        }
    }
    
    /// Create a validator that ensures a string is not empty
    pub fn non_empty() -> impl Fn(&str, &str, &str) -> Result<(), String> {
        |_, _, value| {
            if value.trim().is_empty() {
                Err("Value cannot be empty".to_string())
            } else {
                Ok(())
            }
        }
    }
    
    /// Create a validator that ensures a boolean value
    pub fn boolean() -> impl Fn(&str, &str, &str) -> Result<(), String> {
        |_, _, value| {
            match value.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" => Ok(()),
                _ => Err(format!("Value must be a boolean (true/false), got: {}", value))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;
    
    // Helper to create a test config manager
    fn setup_test_config_manager() -> (TempDir, ConfigManager) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Create a default config
        let config = Config::default();
        
        // Create the config manager
        let config_manager = ConfigManager::new(
            config,
            config_path,
            None
        );
        
        (temp_dir, config_manager)
    }
    
    #[test]
    fn test_config_manager_initialization() {
        let (_temp_dir, config_manager) = setup_test_config_manager();
        
        // Verify we can get the config
        let config = config_manager.get_config().unwrap();
        assert_eq!(config.wallet.network, "Bitcoin");
    }
    
    #[test]
    fn test_update_configuration() {
        let (_temp_dir, config_manager) = setup_test_config_manager();
        
        // Update a configuration value
        config_manager.update_value("wallet", "fee_level", "high").unwrap();
        
        // Verify the change
        let config = config_manager.get_config().unwrap();
        assert_eq!(config.wallet.fee_level, "high");
    }
    
    #[test]
    fn test_validation() {
        let (_temp_dir, mut config_manager) = setup_test_config_manager();
        
        // Add a validator
        config_manager.add_validator(
            "wallet", 
            "fee_level", 
            ConfigValidators::one_of(vec!["low", "medium", "high"])
        );
        
        // Valid update should succeed
        config_manager.update_value("wallet", "fee_level", "high").unwrap();
        
        // Invalid update should fail
        let result = config_manager.update_value("wallet", "fee_level", "ultra");
        assert!(result.is_err());
        
        // Verify the first change was applied but not the second
        let config = config_manager.get_config().unwrap();
        assert_eq!(config.wallet.fee_level, "high");
    }
    
    #[test]
    fn test_profile_management() {
        let (_temp_dir, config_manager) = setup_test_config_manager();
        
        // Create a modified configuration
        config_manager.update_value("wallet", "fee_level", "high").unwrap();
        config_manager.update_value("ui", "dark_mode", "false").unwrap();
        
        // Save as a profile
        config_manager.save_profile("high_fees").unwrap();
        
        // Reset to defaults
        config_manager.reset_to_defaults().unwrap();
        
        // Verify reset worked
        let config = config_manager.get_config().unwrap();
        assert_eq!(config.wallet.fee_level, "medium");
        assert_eq!(config.ui.dark_mode, true);
        
        // Load the profile
        config_manager.load_profile("high_fees").unwrap();
        
        // Verify profile was loaded
        let config = config_manager.get_config().unwrap();
        assert_eq!(config.wallet.fee_level, "high");
        assert_eq!(config.ui.dark_mode, false);
        
        // List profiles
        let profiles = config_manager.list_profiles().unwrap();
        assert!(profiles.contains(&"high_fees".to_string()));
    }
    
    #[test]
    fn test_json_export_import() {
        let (_temp_dir, config_manager) = setup_test_config_manager();
        
        // Modify the configuration
        config_manager.update_value("wallet", "fee_level", "high").unwrap();
        
        // Export to JSON
        let json = config_manager.export_as_json().unwrap();
        
        // Reset to defaults
        config_manager.reset_to_defaults().unwrap();
        
        // Verify reset
        let config = config_manager.get_config().unwrap();
        assert_eq!(config.wallet.fee_level, "medium");
        
        // Import from JSON
        config_manager.import_from_json(&json).unwrap();
        
        // Verify import
        let config = config_manager.get_config().unwrap();
        assert_eq!(config.wallet.fee_level, "high");
    }
    
    #[test]
    fn test_migrations() {
        let (_temp_dir, mut config_manager) = setup_test_config_manager();
        
        // Add a migration function
        config_manager.add_migration(|config| {
            config.wallet.fee_level = "high".to_string();
            Ok(())
        });
        
        // Apply migrations
        let migrated = config_manager.apply_migrations().unwrap();
        assert!(migrated);
        
        // Verify migration was applied
        let config = config_manager.get_config().unwrap();
        assert_eq!(config.wallet.fee_level, "high");
    }
} 