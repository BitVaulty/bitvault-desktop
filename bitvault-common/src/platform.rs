//! Platform-specific functionality for BitVault
//!
//! This module provides abstractions for platform-specific operations:
//! - File paths and directory handling
//! - Secure storage locations
//! - Memory protection
//! - OS-specific security features
//!
//! By using these abstractions, the rest of the codebase can remain
//! platform-agnostic while still taking advantage of platform-specific
//! security features when available.

use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Detected platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    /// Linux platforms
    Linux,
    /// macOS platforms
    MacOS,
    /// Windows platforms
    Windows,
    /// iOS (iPhone, iPad)
    IOS,
    /// Android platforms
    Android,
    /// Unknown/other platforms
    Other,
}

impl fmt::Display for PlatformType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformType::Linux => write!(f, "Linux"),
            PlatformType::MacOS => write!(f, "macOS"),
            PlatformType::Windows => write!(f, "Windows"),
            PlatformType::IOS => write!(f, "iOS"),
            PlatformType::Android => write!(f, "Android"),
            PlatformType::Other => write!(f, "Other"),
        }
    }
}

/// Platform capabilities related to security and storage
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    /// Platform type
    pub platform_type: PlatformType,
    /// Whether this platform has a secure enclave or similar hardware
    pub has_secure_enclave: bool,
    /// Whether this platform supports memory locking
    pub supports_memory_locking: bool,
    /// Whether this platform has a secure storage API
    pub has_secure_storage: bool,
    /// Whether this platform has biometric authentication
    pub has_biometric_auth: bool,
}

/// Get the current platform type
pub fn get_platform_type() -> PlatformType {
    #[cfg(target_os = "linux")]
    {
        #[cfg(target_os = "android")]
        return PlatformType::Android;

        #[cfg(not(target_os = "android"))]
        return PlatformType::Linux;
    }

    #[cfg(target_os = "macos")]
    return PlatformType::MacOS;

    #[cfg(target_os = "windows")]
    return PlatformType::Windows;

    #[cfg(target_os = "ios")]
    return PlatformType::IOS;

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "ios",
        target_os = "android"
    )))]
    return PlatformType::Other;
}

/// Get capabilities of the current platform
pub fn get_platform_capabilities() -> PlatformCapabilities {
    let platform_type = get_platform_type();

    match platform_type {
        PlatformType::Linux => PlatformCapabilities {
            platform_type,
            has_secure_enclave: false,
            supports_memory_locking: true,
            has_secure_storage: false,
            has_biometric_auth: false,
        },
        PlatformType::MacOS => PlatformCapabilities {
            platform_type,
            has_secure_enclave: true,
            supports_memory_locking: true,
            has_secure_storage: true,
            has_biometric_auth: true,
        },
        PlatformType::Windows => PlatformCapabilities {
            platform_type,
            has_secure_enclave: false,
            supports_memory_locking: true,
            has_secure_storage: true,
            has_biometric_auth: true,
        },
        PlatformType::IOS => PlatformCapabilities {
            platform_type,
            has_secure_enclave: true,
            supports_memory_locking: true,
            has_secure_storage: true,
            has_biometric_auth: true,
        },
        PlatformType::Android => PlatformCapabilities {
            platform_type,
            has_secure_enclave: false, // Depends on device, being conservative
            supports_memory_locking: true,
            has_secure_storage: true,
            has_biometric_auth: true,
        },
        PlatformType::Other => PlatformCapabilities {
            platform_type,
            has_secure_enclave: false,
            supports_memory_locking: false,
            has_secure_storage: false,
            has_biometric_auth: false,
        },
    }
}

/// Get the data directory for the wallet
///
/// This returns a platform-specific location that follows OS conventions:
/// - Linux: ~/.local/share/bitvault or XDG_DATA_HOME/bitvault
/// - macOS: ~/Library/Application Support/BitVault
/// - Windows: %APPDATA%\BitVault
///
/// # Returns
/// * PathBuf containing the data directory path
///
/// # Errors
/// * Returns io::Error if the directory couldn't be determined or created
pub fn get_data_dir() -> io::Result<PathBuf> {
    let data_dir = match get_platform_type() {
        PlatformType::Linux => {
            // Use XDG_DATA_HOME if available, otherwise use ~/.local/share
            if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
                PathBuf::from(xdg_data_home).join("bitvault")
            } else {
                let home = dirs::home_dir().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
                })?;
                home.join(".local/share/bitvault")
            }
        }
        PlatformType::MacOS => {
            let home = dirs::home_dir().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
            })?;
            home.join("Library/Application Support/BitVault")
        }
        PlatformType::Windows => {
            if let Ok(app_data) = env::var("APPDATA") {
                PathBuf::from(app_data).join("BitVault")
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "APPDATA environment variable not set",
                ));
            }
        }
        PlatformType::IOS | PlatformType::Android => {
            // Mobile platforms typically have their app-specific data dirs
            // These would be handled by the native code integration
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Mobile platforms handle data directories differently",
            ));
        }
        PlatformType::Other => {
            // Fall back to current directory
            let current_dir = env::current_dir()?;
            current_dir.join("bitvault-data")
        }
    };

    // Create the directory if it doesn't exist
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    Ok(data_dir)
}

/// Get the config directory for the wallet
///
/// This returns a platform-specific location that follows OS conventions:
/// - Linux: ~/.config/bitvault or XDG_CONFIG_HOME/bitvault
/// - macOS: ~/Library/Application Support/BitVault
/// - Windows: %APPDATA%\BitVault
///
/// # Returns
/// * PathBuf containing the config directory path
///
/// # Errors
/// * Returns io::Error if the directory couldn't be determined or created
pub fn get_config_dir() -> io::Result<PathBuf> {
    let config_dir = match get_platform_type() {
        PlatformType::Linux => {
            // Use XDG_CONFIG_HOME if available, otherwise use ~/.config
            if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
                PathBuf::from(xdg_config_home).join("bitvault")
            } else {
                let home = dirs::home_dir().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
                })?;
                home.join(".config/bitvault")
            }
        }
        // For macOS and Windows, we use the same directory for data and config
        PlatformType::MacOS | PlatformType::Windows => get_data_dir()?,
        PlatformType::IOS | PlatformType::Android => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Mobile platforms handle config directories differently",
            ));
        }
        PlatformType::Other => {
            // Fall back to current directory
            let current_dir = env::current_dir()?;
            current_dir.join("bitvault-config")
        }
    };

    // Create the directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

/// Get the logs directory for the wallet
///
/// This returns a platform-specific location that follows OS conventions:
/// - Linux: ~/.local/share/bitvault/logs or XDG_DATA_HOME/bitvault/logs
/// - macOS: ~/Library/Logs/BitVault
/// - Windows: %APPDATA%\BitVault\logs
///
/// # Returns
/// * PathBuf containing the logs directory path
///
/// # Errors
/// * Returns io::Error if the directory couldn't be determined or created
pub fn get_logs_dir() -> io::Result<PathBuf> {
    let logs_dir = match get_platform_type() {
        PlatformType::Linux => {
            // Use data dir + logs
            get_data_dir()?.join("logs")
        }
        PlatformType::MacOS => {
            let home = dirs::home_dir().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
            })?;
            home.join("Library/Logs/BitVault")
        }
        PlatformType::Windows => {
            // Use data dir + logs
            get_data_dir()?.join("logs")
        }
        PlatformType::IOS | PlatformType::Android => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Mobile platforms handle log directories differently",
            ));
        }
        PlatformType::Other => {
            // Fall back to current directory
            let current_dir = env::current_dir()?;
            current_dir.join("bitvault-logs")
        }
    };

    // Create the directory if it doesn't exist
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir)?;
    }

    Ok(logs_dir)
}

/// Get the temp directory for temporary wallet operations
///
/// This returns a platform-specific location that is suitable for temporary files
/// that will be securely deleted after use.
///
/// # Returns
/// * PathBuf containing the temp directory path
///
/// # Errors
/// * Returns io::Error if the directory couldn't be determined or created
pub fn get_temp_dir() -> io::Result<PathBuf> {
    // Start with the system temp directory
    let system_temp = env::temp_dir();
    let wallet_temp = system_temp.join("bitvault-temp");

    // Create a BitVault-specific subdirectory
    if !wallet_temp.exists() {
        fs::create_dir_all(&wallet_temp)?;

        // Set permissions to be restrictive (platform-dependent)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            fs::set_permissions(&wallet_temp, perms)?;
        }
    }

    Ok(wallet_temp)
}

/// Check if we can use secure memory features on this platform
///
/// This checks if the platform supports memory locking or other
/// secure memory features that can protect sensitive data.
///
/// # Returns
/// * true if secure memory features are available, false otherwise
pub fn has_secure_memory() -> bool {
    get_platform_capabilities().supports_memory_locking
}

/// Lock memory to prevent swapping (if supported by the platform)
///
/// # Arguments
/// * `ptr` - Pointer to the memory to lock
/// * `len` - Length of the memory region to lock
///
/// # Returns
/// * Ok(()) if successful, Err with a message if not supported or failed
pub fn lock_memory(_ptr: *const u8, _len: usize) -> Result<(), String> {
    // Implementation depends on the platform
    #[cfg(unix)]
    {
        // On Unix systems, we can use mlock
        use libc::{mlock, ENOMEM};
        use std::io::Error;

        let ret = unsafe { mlock(_ptr as *const libc::c_void, _len) };
        if ret != 0 {
            let err = Error::last_os_error();
            if err.raw_os_error() == Some(ENOMEM) {
                return Err("Not enough memory or permissions to lock memory".to_string());
            } else {
                return Err(format!("Failed to lock memory: {}", err));
            }
        }
        Ok(())
    }

    #[cfg(windows)]
    {
        // On Windows, we would use VirtualLock
        // This is a simplified implementation, in reality you'd use the windows API directly
        use std::io::Error;
        use winapi::um::memoryapi::VirtualLock;

        let ret = unsafe { VirtualLock(_ptr as *mut _, _len) };
        if ret == 0 {
            let err = Error::last_os_error();
            return Err(format!("Failed to lock memory: {}", err));
        }
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        // For other platforms, we report that it's not supported
        Err("Memory locking not supported on this platform".to_string())
    }
}

/// Unlock previously locked memory
///
/// # Arguments
/// * `ptr` - Pointer to the memory to unlock
/// * `len` - Length of the memory region to unlock
///
/// # Returns
/// * Ok(()) if successful, Err with a message if not supported or failed
pub fn unlock_memory(_ptr: *const u8, _len: usize) -> Result<(), String> {
    // Implementation depends on the platform
    #[cfg(unix)]
    {
        // On Unix systems, we can use munlock
        use libc::munlock;
        use std::io::Error;

        let ret = unsafe { munlock(_ptr as *const libc::c_void, _len) };
        if ret != 0 {
            let err = Error::last_os_error();
            return Err(format!("Failed to unlock memory: {}", err));
        }
        Ok(())
    }

    #[cfg(windows)]
    {
        // On Windows, we would use VirtualUnlock
        use std::io::Error;
        use winapi::um::memoryapi::VirtualUnlock;

        let ret = unsafe { VirtualUnlock(_ptr as *mut _, _len) };
        if ret == 0 {
            let err = Error::last_os_error();
            return Err(format!("Failed to unlock memory: {}", err));
        }
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        // For other platforms, we report that it's not supported
        Err("Memory unlocking not supported on this platform".to_string())
    }
}

/// Secure memory allocation for sensitive data
///
/// Attempts to allocate memory in a way that reduces the risk of it being
/// swapped to disk or appearing in core dumps.
///
/// # Returns
/// * A Vec<u8> potentially with better security properties than regular allocations.
pub fn secure_alloc(size: usize) -> Vec<u8> {
    let buffer = vec![0u8; size];

    // Try to lock the memory if supported
    if has_secure_memory() {
        let _ = lock_memory(buffer.as_ptr(), buffer.len());
    }

    buffer
}

/// Securely erase a buffer
///
/// Overwrites the buffer with zeros to remove sensitive data.
///
/// # Arguments
/// * `buffer` - Mutable reference to the buffer to erase
///
/// # Note
/// This is a primitive implementation. For actual security-critical code,
/// use the zeroize crate instead.
pub fn secure_erase(buffer: &mut [u8]) {
    for byte in buffer.iter_mut() {
        *byte = 0;
    }

    // This fence ensures that the compiler doesn't optimize away the zeroing
    std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
}

/// Check if a directory is writable by creating a temporary file in it
pub fn check_dir_writable(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", path.display()));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
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
        Err(e) => Err(format!("Failed to write to directory: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[test]
    fn test_get_platform_type() {
        let platform = get_platform_type();
        // We can only assert that we get a valid platform type
        assert!(matches!(
            platform,
            PlatformType::Linux
                | PlatformType::MacOS
                | PlatformType::Windows
                | PlatformType::IOS
                | PlatformType::Android
                | PlatformType::Other
        ));
    }
    
    #[test]
    fn test_get_platform_capabilities() {
        let capabilities = get_platform_capabilities();
        assert_eq!(capabilities.platform_type, get_platform_type());
    }
}

/// Mock implementation of platform capabilities for testing
/// 
/// # WARNING: FOR TESTING PURPOSES ONLY
/// 
/// This mock implementation is designed specifically for testing and should NEVER
/// be used in production environments. It provides simulated platform capabilities
/// that may not reflect the actual system's capabilities.
/// 
/// # Testing Assumptions
/// 
/// - The mock provides configurable responses for all platform-related features
/// - Memory locking operations will not actually lock memory
/// - Directory paths may not correspond to real system paths
/// - Security features are simulated and not actually enabled
///
/// # Security Note
/// 
/// Using this mock in production could lead to:
/// - Insecure handling of sensitive data
/// - Incorrect assumptions about platform security features
/// - Potential data loss due to incorrect path handling
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockPlatformProvider {
    platform_type: PlatformType,
    has_secure_enclave: bool,
    supports_memory_locking: bool,
    has_secure_storage: bool,
    has_biometric_auth: bool,
    data_dir: PathBuf,
    config_dir: PathBuf,
    logs_dir: PathBuf,
    temp_dir: PathBuf,
}

#[cfg(test)]
impl Default for MockPlatformProvider {
    fn default() -> Self {
        Self {
            platform_type: PlatformType::Linux,
            has_secure_enclave: false,
            supports_memory_locking: true,
            has_secure_storage: true,
            has_biometric_auth: false,
            data_dir: PathBuf::from("/mock/data"),
            config_dir: PathBuf::from("/mock/config"),
            logs_dir: PathBuf::from("/mock/logs"),
            temp_dir: PathBuf::from("/mock/temp"),
        }
    }
}

#[cfg(test)]
impl MockPlatformProvider {
    /// Create a new mock platform provider with the specified platform type
    pub fn new(platform_type: PlatformType) -> Self {
        Self {
            platform_type,
            ..Default::default()
        }
    }
    
    /// Configure whether the mock has a secure enclave
    pub fn with_secure_enclave(mut self, has_secure_enclave: bool) -> Self {
        self.has_secure_enclave = has_secure_enclave;
        self
    }
    
    /// Configure whether the mock supports memory locking
    pub fn with_memory_locking(mut self, supports_memory_locking: bool) -> Self {
        self.supports_memory_locking = supports_memory_locking;
        self
    }
    
    /// Configure whether the mock has secure storage
    pub fn with_secure_storage(mut self, has_secure_storage: bool) -> Self {
        self.has_secure_storage = has_secure_storage;
        self
    }
    
    /// Configure whether the mock has biometric authentication
    pub fn with_biometric_auth(mut self, has_biometric_auth: bool) -> Self {
        self.has_biometric_auth = has_biometric_auth;
        self
    }
    
    /// Configure the data directory path
    pub fn with_data_dir(mut self, data_dir: PathBuf) -> Self {
        self.data_dir = data_dir;
        self
    }
    
    /// Get the platform capabilities from this mock
    pub fn get_capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            platform_type: self.platform_type,
            has_secure_enclave: self.has_secure_enclave,
            supports_memory_locking: self.supports_memory_locking,
            has_secure_storage: self.has_secure_storage,
            has_biometric_auth: self.has_biometric_auth,
        }
    }
    
    /// Get the data directory from this mock
    pub fn get_data_dir(&self) -> io::Result<PathBuf> {
        Ok(self.data_dir.clone())
    }
    
    /// Get the config directory from this mock
    pub fn get_config_dir(&self) -> io::Result<PathBuf> {
        Ok(self.config_dir.clone())
    }
    
    /// Get the logs directory from this mock
    pub fn get_logs_dir(&self) -> io::Result<PathBuf> {
        Ok(self.logs_dir.clone())
    }
    
    /// Get the temp directory from this mock
    pub fn get_temp_dir(&self) -> io::Result<PathBuf> {
        Ok(self.temp_dir.clone())
    }
}
