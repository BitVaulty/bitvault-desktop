//! Platform-specific path functions
//!
//! This module provides functions for getting platform-specific paths
//! for data, configuration, logs, and temporary files.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::error::PlatformResult;
use super::types::PlatformType;

/// Get the data directory for the wallet on Linux
pub fn get_linux_data_dir() -> io::Result<PathBuf> {
    // Use XDG_DATA_HOME if available, otherwise use ~/.local/share
    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        let dir = PathBuf::from(xdg_data_home).join("bitvault");
        create_dir_if_missing(&dir)?;
        Ok(dir)
    } else {
        let home = dirs::home_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
        })?;
        let dir = home.join(".local/share/bitvault");
        create_dir_if_missing(&dir)?;
        Ok(dir)
    }
}

/// Get the data directory for the wallet on macOS
pub fn get_macos_data_dir() -> io::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;
    let dir = home.join("Library/Application Support/BitVault");
    create_dir_if_missing(&dir)?;
    Ok(dir)
}

/// Get the data directory for the wallet on Windows
pub fn get_windows_data_dir() -> io::Result<PathBuf> {
    if let Ok(app_data) = env::var("APPDATA") {
        let dir = PathBuf::from(app_data).join("BitVault");
        create_dir_if_missing(&dir)?;
        Ok(dir)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "APPDATA environment variable not set",
        ))
    }
}

/// Get the data directory for the wallet on mobile platforms
pub fn get_mobile_data_dir() -> io::Result<PathBuf> {
    // Mobile platforms typically have their app-specific data dirs
    // These would be handled by the native code integration
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Mobile platforms handle data directories differently",
    ))
}

/// Get the data directory for the wallet on other platforms
pub fn get_other_data_dir() -> io::Result<PathBuf> {
    // Fall back to current directory
    let dir = env::current_dir()?.join("bitvault-data");
    create_dir_if_missing(&dir)?;
    Ok(dir)
}

/// Get the config directory for the wallet on Linux
pub fn get_linux_config_dir() -> io::Result<PathBuf> {
    // Use XDG_CONFIG_HOME if available, otherwise use ~/.config
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        let dir = PathBuf::from(xdg_config_home).join("bitvault");
        create_dir_if_missing(&dir)?;
        Ok(dir)
    } else {
        let home = dirs::home_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
        })?;
        let dir = home.join(".config/bitvault");
        create_dir_if_missing(&dir)?;
        Ok(dir)
    }
}

/// Get the config directory for the wallet on macOS and Windows
pub fn get_desktop_config_dir(platform_type: PlatformType) -> io::Result<PathBuf> {
    // For macOS and Windows, we use the same directory for data and config
    match platform_type {
        PlatformType::MacOS => get_macos_data_dir(),
        PlatformType::Windows => get_windows_data_dir(),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unexpected platform type",
        )),
    }
}

/// Get the logs directory for the wallet on Linux and Windows
pub fn get_default_logs_dir(data_dir: &Path) -> io::Result<PathBuf> {
    let dir = data_dir.join("logs");
    create_dir_if_missing(&dir)?;
    Ok(dir)
}

/// Get the logs directory for the wallet on macOS
pub fn get_macos_logs_dir() -> io::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;
    let dir = home.join("Library/Logs/BitVault");
    create_dir_if_missing(&dir)?;
    Ok(dir)
}

/// Get a platform-specific temp directory for the wallet
pub fn get_temp_dir() -> io::Result<PathBuf> {
    // Start with the system temp directory
    let system_temp = env::temp_dir();
    let wallet_temp = system_temp.join("bitvault-temp");
    create_dir_if_missing(&wallet_temp)?;

    // Set permissions to be restrictive (platform-dependent)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o700);
        fs::set_permissions(&wallet_temp, perms)?;
    }

    Ok(wallet_temp)
}

/// Create a directory if it doesn't exist
fn create_dir_if_missing(dir: &Path) -> io::Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Check if a directory is writable by creating a temporary file in it
pub fn check_dir_writable(path: &Path) -> PlatformResult<()> {
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", path.display()).into());
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()).into());
    }
    
    let file_name = format!(
        "bitvault-write-test-{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    
    let test_path = path.join(file_name);
    
    match fs::File::create(&test_path) {
        Ok(_) => {
            // Successfully created, now clean up
            let _ = fs::remove_file(&test_path);
            Ok(())
        }
        Err(e) => Err(format!("Failed to write to directory: {}", e).into()),
    }
} 