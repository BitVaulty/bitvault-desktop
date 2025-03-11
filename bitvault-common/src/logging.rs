//! Security-aware logging infrastructure for BitVault wallet
//!
//! This module provides structured logging with security considerations:
//! - Never logs sensitive information (private keys, seed phrases)
//! - Sanitizes potentially sensitive values (addresses, transaction IDs)
//! - Categorizes log events by security context
//! - Provides both human-readable and machine-parseable output
//!
//! # Security Considerations
//!
//! - NEVER log private keys, seeds, or other sensitive cryptographic material
//! - Truncate potentially sensitive values (addresses, transaction IDs) when logging
//! - Use appropriate log levels to avoid leaking information in production
//! - Structured logging (JSON) is available for machine processing
//!
//! # Usage
//!
//! ```
//! use bitvault_common::logging;
//! use bitvault_common::logging::LogConfig;
//! use bitvault_common::logging::LogLevel;
//! use serde_json::json;
//!
//! // Initialize logging with default configuration
//! logging::init(&LogConfig::default()).expect("Failed to initialize logging");
//!
//! // Log events with context
//! logging::log_security(
//!     LogLevel::Info,
//!     "user_authenticated",
//!     Some(json!({
//!         "user_id": "12345",
//!         "method": "password"
//!     }))
//! );
//! ```

use chrono::Local;
use log::{debug, error, info, trace, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use std::path::Path;

use crate::types::{SensitiveBytes, SensitiveString};

/// Log severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Error conditions
    Error,
    /// Warning conditions
    Warn,
    /// Informational messages
    Info,
    /// Debug-level messages
    Debug,
    /// Trace level (very verbose)
    Trace,
}

/// Log context categories for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogContext {
    /// Security-sensitive operations like key management, authentication
    Security,
    /// Core wallet functionality, transaction validation
    Core,
    /// Network operations, blockchain synchronization
    Network,
    /// Transaction creation, signing, broadcasting
    Transaction,
    /// User interface interactions
    UI,
    /// Storage operations (database, file system)
    Storage,
}

/// Configuration for the logging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Default log level for all contexts
    pub level: LogLevel,
    /// Path to log file (None for console-only)
    pub log_file: Option<String>,
    /// Whether to include timestamps in log messages
    pub include_timestamps: bool,
    /// Whether to include source location in log messages
    pub include_source_location: bool,
    /// Maximum log file size in bytes before rotation
    pub max_file_size: usize,
    /// Whether to log to console
    pub console_logging: bool,
    /// Whether to use JSON format for logs (machine-readable)
    pub json_format: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            log_file: None,
            include_timestamps: true,
            include_source_location: true,
            max_file_size: 10 * 1024 * 1024, // 10MB
            console_logging: true,
            json_format: false,
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// Initialize the logging system with the given configuration
pub fn init(config: &LogConfig) -> Result<(), String> {
    // Clone the values we need to avoid borrowing issues with the closure
    let include_timestamps = config.include_timestamps;
    let include_source_location = config.include_source_location;
    let json_format = config.json_format;

    let mut builder = env_logger::Builder::new();
    builder.filter_level(config.level.into());

    // Set the format with cloned values to avoid borrowing issues
    builder.format(move |buf, record| {
        if json_format {
            // Structured JSON logging
            let json = json!({
                "timestamp": Local::now().to_rfc3339(),
                "level": record.level().to_string(),
                "target": record.target().to_string(),
                "message": record.args().to_string(),
                "file": record.file().unwrap_or("unknown"),
                "line": record.line().unwrap_or(0),
            });
            writeln!(buf, "{}", json)
        } else {
            // Human-readable logging
            if include_timestamps {
                let _ = write!(buf, "[{}] ", Local::now().format("%Y-%m-%d %H:%M:%S"));
            }
            let _ = write!(buf, "{:<5} ", record.level());

            if include_source_location {
                if let (Some(file), Some(line)) = (record.file(), record.line()) {
                    let _ = write!(buf, "[{}:{}] ", file, line);
                }
            }

            let _ = writeln!(buf, "{}", record.args());
            Ok(())
        }
    });

    // Configure output
    if config.console_logging {
        builder.target(env_logger::Target::Stdout);
    }

    // Add file logging if configured
    if let Some(log_file) = &config.log_file {
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(log_file) {
            builder.target(env_logger::Target::Pipe(Box::new(file)));
        } else {
            return Err(format!("Failed to open log file: {}", log_file));
        }
    }

    // Initialize the logger
    builder.try_init().map_err(|e| e.to_string())
}

/// Update the log level dynamically
pub fn set_log_level(level: LogLevel) {
    log::set_max_level(level.into());
}

/// Sanitize a potentially sensitive string for logging
/// This truncates the middle part of strings that might contain sensitive data
pub fn sanitize_for_logging(input: &str) -> String {
    if input.is_empty() {
        return String::from("");
    }

    // Keep only the first and last few characters
    let len = input.len();
    if len <= 8 {
        // For very short strings, show ***** instead
        return "*****".to_string();
    }

    // For longer strings, keep first 4 and last 4 characters
    let first = &input[0..4];
    let last = &input[len - 4..len];
    format!("{}...{}", first, last)
}

/// Sanitize a SensitiveString for logging
pub fn sanitize_sensitive(input: &SensitiveString) -> String {
    sanitize_for_logging(input.as_str())
}

/// Sanitize SensitiveBytes for logging
pub fn sanitize_sensitive_bytes(input: &SensitiveBytes) -> String {
    input.to_sanitized_string()
}

/// Trait for types that can be safely logged
pub trait SafeLog {
    /// Return a sanitized string representation safe for logging
    fn safe_log_format(&self) -> String;
}

impl SafeLog for String {
    fn safe_log_format(&self) -> String {
        self.clone()
    }
}

impl SafeLog for &str {
    fn safe_log_format(&self) -> String {
        (*self).to_string()
    }
}

impl SafeLog for SensitiveString {
    fn safe_log_format(&self) -> String {
        sanitize_sensitive(self)
    }
}

impl SafeLog for SensitiveBytes {
    fn safe_log_format(&self) -> String {
        sanitize_sensitive_bytes(self)
    }
}

/// Log a security event with appropriate sanitization
pub fn log_security(level: LogLevel, message: &str, params: Option<serde_json::Value>) {
    let formatted = format_log_entry(LogContext::Security, message, params);
    match level {
        LogLevel::Error => error!("{}", formatted),
        LogLevel::Warn => warn!("{}", formatted),
        LogLevel::Info => info!("{}", formatted),
        LogLevel::Debug => debug!("{}", formatted),
        LogLevel::Trace => trace!("{}", formatted),
    }
}

/// Log a core wallet event
pub fn log_core(level: LogLevel, message: &str, params: Option<serde_json::Value>) {
    let formatted = format_log_entry(LogContext::Core, message, params);
    match level {
        LogLevel::Error => error!("{}", formatted),
        LogLevel::Warn => warn!("{}", formatted),
        LogLevel::Info => info!("{}", formatted),
        LogLevel::Debug => debug!("{}", formatted),
        LogLevel::Trace => trace!("{}", formatted),
    }
}

/// Log a network event
pub fn log_network(level: LogLevel, message: &str, params: Option<serde_json::Value>) {
    let formatted = format_log_entry(LogContext::Network, message, params);
    match level {
        LogLevel::Error => error!("{}", formatted),
        LogLevel::Warn => warn!("{}", formatted),
        LogLevel::Info => info!("{}", formatted),
        LogLevel::Debug => debug!("{}", formatted),
        LogLevel::Trace => trace!("{}", formatted),
    }
}

/// Log a transaction event with appropriate sanitization
pub fn log_transaction(level: LogLevel, message: &str, params: Option<serde_json::Value>) {
    let formatted = format_log_entry(LogContext::Transaction, message, params);
    match level {
        LogLevel::Error => error!("{}", formatted),
        LogLevel::Warn => warn!("{}", formatted),
        LogLevel::Info => info!("{}", formatted),
        LogLevel::Debug => debug!("{}", formatted),
        LogLevel::Trace => trace!("{}", formatted),
    }
}

/// Log a UI event
pub fn log_ui(level: LogLevel, message: &str, params: Option<serde_json::Value>) {
    let formatted = format_log_entry(LogContext::UI, message, params);
    match level {
        LogLevel::Error => error!("{}", formatted),
        LogLevel::Warn => warn!("{}", formatted),
        LogLevel::Info => info!("{}", formatted),
        LogLevel::Debug => debug!("{}", formatted),
        LogLevel::Trace => trace!("{}", formatted),
    }
}

/// Log a storage event
pub fn log_storage(level: LogLevel, message: &str, params: Option<serde_json::Value>) {
    let formatted = format_log_entry(LogContext::Storage, message, params);
    match level {
        LogLevel::Error => error!("{}", formatted),
        LogLevel::Warn => warn!("{}", formatted),
        LogLevel::Info => info!("{}", formatted),
        LogLevel::Debug => debug!("{}", formatted),
        LogLevel::Trace => trace!("{}", formatted),
    }
}

/// Format a log entry with context and optional structured data
fn format_log_entry(
    context: LogContext,
    message: &str,
    params: Option<serde_json::Value>,
) -> String {
    if let Some(params) = params {
        // Include structured data in JSON format
        format!("[{:?}] {} | {}", context, message, params)
    } else {
        // Just the message with context
        format!("[{:?}] {}", context, message)
    }
}

/// Log a potentially sensitive parameter securely
///
/// # Arguments
/// * `name` - Parameter name
/// * `value` - The value to log (will be sanitized)
///
/// # Returns
/// A JSON value with the sanitized parameter
pub fn log_sensitive_param(name: &str, value: &impl SafeLog) -> serde_json::Value {
    json!({
        name: value.safe_log_format()
    })
}

/// Log a map of parameters with potential sanitization
///
/// # Arguments
/// * `params` - Vector of (name, sanitized_value) pairs
///
/// # Returns
/// A JSON value with all parameters
pub fn log_params(params: Vec<(&str, String)>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (name, value) in params {
        map.insert(name.to_string(), serde_json::Value::String(value));
    }
    serde_json::Value::Object(map)
}

/// Write to a log file directly (for cases where the logger isn't appropriate)
pub fn write_to_log_file(log_path: &Path, message: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| e.to_string())?;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] {}\n", timestamp, message);

    file.write_all(log_line.as_bytes())
        .map_err(|e| e.to_string())
}
