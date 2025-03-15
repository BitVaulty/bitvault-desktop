//! Configuration management for BitVault.
//!
//! This module handles application settings stored in TOML format.
//! It provides a strongly-typed configuration structure with validation
//! and reasonable defaults.
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

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::vec::Vec;
use serde_json::json;
use crate::events::{MessageBus, EventType, MessagePriority};

/// Main configuration structure for BitVault
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub wallet: WalletConfig,

    #[serde(default)]
    pub network: NetworkConfig,

    #[serde(default)]
    pub ipc: IpcConfig,

    #[serde(default)]
    pub ui: UiConfig,

    #[serde(default)]
    pub storage: StorageConfig,
}

/// Wallet-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    #[serde(default = "default_network")]
    pub network: String,

    #[serde(default = "default_fiat")]
    pub fiat_currency: String,

    /// Default fee level (low, medium, high)
    #[serde(default = "default_fee_level")]
    pub fee_level: String,

    /// Address re-use prevention
    #[serde(default = "default_true")]
    pub prevent_address_reuse: bool,

    /// Coin selection strategy (privacy, economical, efficient)
    #[serde(default = "default_coin_selection")]
    pub coin_selection: String,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            network: default_network(),
            fiat_currency: default_fiat(),
            fee_level: default_fee_level(),
            prevent_address_reuse: default_true(),
            coin_selection: default_coin_selection(),
        }
    }
}

/// Network-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,

    #[serde(default)]
    pub use_tor: bool,

    /// Trusted peers to connect to (optional)
    #[serde(default)]
    pub trusted_peers: Vec<String>,

    /// Maximum number of connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout(),
            use_tor: false,
            trusted_peers: Vec::new(),
            max_connections: default_max_connections(),
        }
    }
}

/// Inter-process communication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcConfig {
    #[serde(default = "default_ipc_port")]
    pub port: u16,

    #[serde(default = "default_message_size")]
    pub max_message_size_mb: u32,

    /// IPC timeout in seconds
    #[serde(default = "default_ipc_timeout")]
    pub timeout_seconds: u32,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            port: default_ipc_port(),
            max_message_size_mb: default_message_size(),
            timeout_seconds: default_ipc_timeout(),
        }
    }
}

/// User interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub dark_mode: bool,

    #[serde(default = "default_language")]
    pub language: String,

    /// Display amounts in BTC (true) or sats (false)
    #[serde(default = "default_true")]
    pub display_as_btc: bool,

    /// Show fiat equivalent values
    #[serde(default = "default_true")]
    pub show_fiat: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            dark_mode: default_true(),
            language: default_language(),
            display_as_btc: default_true(),
            show_fiat: default_true(),
        }
    }
}

/// Storage and database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_db_path")]
    pub database_path: String,

    #[serde(default = "default_true")]
    pub encrypted: bool,

    /// Backup directory path
    #[serde(default = "default_backup_path")]
    pub backup_directory: String,

    /// Transaction history retention in days (0 = forever)
    #[serde(default = "default_retention")]
    pub tx_retention_days: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_path: default_db_path(),
            encrypted: default_true(),
            backup_directory: default_backup_path(),
            tx_retention_days: default_retention(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &str) -> Result<Self> {
        let content =
            fs::read_to_string(path).map_err(|e| anyhow!("Failed to read config file: {}", e))?;

        let config: Config =
            toml::from_str(&content).map_err(|e| anyhow!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &str, message_bus: Option<&MessageBus>) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        fs::write(path, content).map_err(|e| anyhow!("Failed to write config file: {}", e))?;
        
        // Emit config update event if message bus is provided
        if let Some(bus) = message_bus {
            let payload = json!({
                "action": "config_saved",
                "path": path,
                "wallet_network": self.wallet.network,
                "fiat_currency": self.wallet.fiat_currency,
                "fee_level": self.wallet.fee_level,
                "ui_language": self.ui.language,
                "dark_mode": self.ui.dark_mode,
            });
            
            bus.publish(
                EventType::ConfigUpdate,
                &payload.to_string(),
                MessagePriority::Low
            );
        }

        Ok(())
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate IPC port
        if self.ipc.port == 0 {
            return Err(anyhow!("Invalid IPC port: must be greater than 0"));
        }

        // Validate network timeout
        if self.network.timeout_seconds == 0 {
            return Err(anyhow!("Invalid network timeout: must be greater than 0"));
        }

        // Validate max message size
        if self.ipc.max_message_size_mb == 0 {
            return Err(anyhow!("Invalid max message size: must be greater than 0"));
        }

        // Validate Bitcoin network type
        match self.wallet.network.as_str() {
            "Bitcoin" | "Testnet" | "Regtest" => {}
            _ => anyhow::bail!("Invalid network type: {}", self.wallet.network),
        }

        Ok(())
    }
}

/// Ensure a configuration file exists at the specified path
/// If it doesn't exist, create it with default values
pub fn ensure_config_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        let default_config = Config::default();
        let content = toml::to_string_pretty(&default_config)
            .map_err(|e| anyhow!("Failed to serialize default config: {}", e))?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("Failed to create config directory: {}", e))?;
            }
        }

        fs::write(path, content)
            .map_err(|e| anyhow!("Failed to write default config file: {}", e))?;
    }

    Ok(())
}

// Default value functions

fn default_fee_level() -> String {
    "medium".to_string()
}

fn default_true() -> bool {
    true
}

fn default_coin_selection() -> String {
    "privacy".to_string()
}

fn default_max_connections() -> u32 {
    8
}

fn default_ipc_timeout() -> u32 {
    15
}

fn default_backup_path() -> String {
    if cfg!(target_os = "linux") {
        "/home/user/.bitvault/backup".to_string()
    } else if cfg!(target_os = "macos") {
        "~/Library/Application Support/BitVault/backup".to_string()
    } else if cfg!(target_os = "windows") {
        "%APPDATA%\\BitVault\\backup".to_string()
    } else {
        "./bitvault-data/backup".to_string()
    }
}

fn default_retention() -> u32 {
    0 // 0 means keep forever
}

fn default_network() -> String {
    "Bitcoin".to_string()
}

fn default_fiat() -> String {
    "USD".to_string()
}

fn default_timeout() -> u32 {
    30
}

fn default_ipc_port() -> u16 {
    8999
}

fn default_message_size() -> u32 {
    1
}

fn default_language() -> String {
    "en".to_string()
}

fn default_db_path() -> String {
    "/home/user/.bitvault/db".to_string()
}
