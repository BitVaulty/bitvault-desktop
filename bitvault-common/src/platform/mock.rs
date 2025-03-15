//! Mock implementation of platform capabilities for testing
//! 
//! # WARNING: FOR TESTING PURPOSES ONLY
//! 
//! This mock implementation is designed specifically for testing and should NEVER
//! be used in production environments. It provides simulated platform capabilities
//! that may not reflect the actual system's capabilities.
//! 
//! # Testing Assumptions
//! 
//! - The mock provides configurable responses for all platform-related features
//! - Memory locking operations will not actually lock memory
//! - Directory paths may not correspond to real system paths
//! - Security features are simulated and not actually enabled
//!
//! # Security Note
//! 
//! Using this mock in production could lead to:
//! - Insecure handling of sensitive data
//! - Incorrect assumptions about platform security features
//! - Potential data loss due to incorrect path handling

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use super::capabilities::PlatformCapabilities;
use super::memory;
use super::provider::PlatformProvider;
use super::types::PlatformType;

/// Mock implementation of platform capabilities for testing
#[derive(Debug)]
pub struct MockPlatformProvider {
    platform_type: PlatformType,
    capabilities: PlatformCapabilities,
    data_dir: PathBuf,
    config_dir: PathBuf,
    logs_dir: PathBuf,
    temp_dir: PathBuf,
    secure_storage: RwLock<HashMap<String, Vec<u8>>>,
    biometric_auth_result: Option<Result<bool, String>>,
}

impl Default for MockPlatformProvider {
    fn default() -> Self {
        Self {
            platform_type: PlatformType::Linux,
            capabilities: PlatformCapabilities::new(PlatformType::Linux),
            data_dir: PathBuf::from("/mock/data"),
            config_dir: PathBuf::from("/mock/config"),
            logs_dir: PathBuf::from("/mock/logs"),
            temp_dir: PathBuf::from("/mock/temp"),
            secure_storage: RwLock::new(HashMap::new()),
            biometric_auth_result: None,
        }
    }
}

impl MockPlatformProvider {
    /// Create a new mock platform provider with the specified platform type
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Configure the platform type
    pub fn with_platform_type(mut self, platform_type: PlatformType) -> Self {
        self.platform_type = platform_type;
        self.capabilities = PlatformCapabilities::new(platform_type);
        self
    }
    
    /// Configure whether the mock has a secure enclave
    pub fn with_secure_enclave(mut self, has_secure_enclave: bool) -> Self {
        let mut capabilities = self.capabilities.clone();
        capabilities.has_secure_enclave = has_secure_enclave;
        self.capabilities = capabilities;
        self
    }
    
    /// Configure whether the mock supports memory locking
    pub fn with_memory_locking(mut self, supports_memory_locking: bool) -> Self {
        let mut capabilities = self.capabilities.clone();
        capabilities.supports_memory_locking = supports_memory_locking;
        self.capabilities = capabilities;
        self
    }
    
    /// Configure whether the mock has secure storage
    pub fn with_secure_storage(mut self, has_secure_storage: bool) -> Self {
        let mut capabilities = self.capabilities.clone();
        capabilities.has_secure_storage = has_secure_storage;
        self.capabilities = capabilities;
        
        // Initialize the secure storage if it's being enabled
        if has_secure_storage {
            println!("MockPlatformProvider: Initializing secure storage");
            if let Ok(mut storage) = self.secure_storage.write() {
                // Clear any existing data
                storage.clear(); 
                println!("MockPlatformProvider: Secure storage initialized with an empty state");
            } else {
                println!("MockPlatformProvider: Failed to initialize secure storage - couldn't acquire write lock");
            }
        }
        
        self
    }
    
    /// Configure whether the mock has biometric authentication
    pub fn with_biometric_auth(mut self, has_biometric_auth: bool) -> Self {
        let mut capabilities = self.capabilities.clone();
        capabilities.has_biometric_auth = has_biometric_auth;
        self.capabilities = capabilities;
        self
    }
    
    /// Configure the data directory path
    pub fn with_data_dir(mut self, data_dir: PathBuf) -> Self {
        self.data_dir = data_dir;
        self
    }
    
    /// Configure the config directory path
    pub fn with_config_dir(mut self, config_dir: PathBuf) -> Self {
        self.config_dir = config_dir;
        self
    }
    
    /// Configure the logs directory path
    pub fn with_logs_dir(mut self, logs_dir: PathBuf) -> Self {
        self.logs_dir = logs_dir;
        self
    }
    
    /// Configure the temp directory path
    pub fn with_temp_dir(mut self, temp_dir: PathBuf) -> Self {
        self.temp_dir = temp_dir;
        self
    }
    
    /// Set a predefined result for biometric authentication
    pub fn with_biometric_auth_result(mut self, result: Result<bool, String>) -> Self {
        self.biometric_auth_result = Some(result);
        self
    }
}

impl PlatformProvider for MockPlatformProvider {
    fn get_platform_type(&self) -> PlatformType {
        self.platform_type
    }
    
    fn get_capabilities(&self) -> PlatformCapabilities {
        self.capabilities.clone()
    }
    
    fn get_data_dir(&self) -> io::Result<PathBuf> {
        Ok(self.data_dir.clone())
    }
    
    fn get_config_dir(&self) -> io::Result<PathBuf> {
        Ok(self.config_dir.clone())
    }
    
    fn get_logs_dir(&self) -> io::Result<PathBuf> {
        Ok(self.logs_dir.clone())
    }
    
    fn get_temp_dir(&self) -> io::Result<PathBuf> {
        Ok(self.temp_dir.clone())
    }
    
    fn has_secure_memory(&self) -> bool {
        self.capabilities.supports_memory_locking
    }
    
    fn lock_memory(&self, _ptr: *const u8, _len: usize) -> Result<(), String> {
        if self.capabilities.supports_memory_locking {
            Ok(())
        } else {
            Err("Memory locking not supported in this mock".to_string())
        }
    }
    
    fn unlock_memory(&self, _ptr: *const u8, _len: usize) -> Result<(), String> {
        if self.capabilities.supports_memory_locking {
            Ok(())
        } else {
            Err("Memory unlocking not supported in this mock".to_string())
        }
    }
    
    fn secure_alloc(&self, size: usize) -> Vec<u8> {
        // Don't actually lock memory in tests
        memory::secure_alloc(size, false)
    }
    
    fn secure_erase(&self, buffer: &mut [u8]) {
        memory::secure_erase(buffer)
    }
    
    fn check_dir_writable(&self, path: &Path) -> Result<(), String> {
        // Check for specific mock test path
        if path == Path::new("/mock/not-writable") {
            return Err("Directory is not writable (mock)".to_string());
        }
        
        // Check if the directory exists
        if !path.exists() {
            return Err(format!("Directory does not exist: {}", path.display()));
        }
        
        // If it exists, consider it writable in the mock
        Ok(())
    }
    
    fn store_secure_item(&self, key: &str, value: &[u8]) -> Result<(), String> {
        if !self.capabilities.has_secure_storage {
            return Err("Secure storage not supported in this mock".to_string());
        }
        
        println!("MockPlatformProvider: Storing key {} with value length: {}", key, value.len());
        match self.secure_storage.write() {
            Ok(mut storage) => {
                storage.insert(key.to_string(), value.to_vec());
                println!("MockPlatformProvider: Stored key {} successfully", key);
                Ok(())
            }
            Err(_) => {
                let err = "Failed to acquire write lock on secure storage".to_string();
                println!("MockPlatformProvider: ERROR - {}", err);
                Err(err)
            }
        }
    }
    
    fn retrieve_secure_item(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        if !self.capabilities.has_secure_storage {
            return Err("Secure storage not supported in this mock".to_string());
        }
        
        println!("MockPlatformProvider: Retrieving key {}", key);
        match self.secure_storage.read() {
            Ok(storage) => {
                let result = storage.get(key).cloned();
                if let Some(ref val) = result {
                    println!("MockPlatformProvider: Retrieved key {} successfully with value length: {}", key, val.len());
                } else {
                    println!("MockPlatformProvider: Key {} not found in secure storage", key);
                }
                Ok(result)
            }
            Err(_) => {
                let err = "Failed to acquire read lock on secure storage".to_string();
                println!("MockPlatformProvider: ERROR - {}", err);
                Err(err)
            }
        }
    }
    
    fn delete_secure_item(&self, key: &str) -> Result<(), String> {
        if !self.capabilities.has_secure_storage {
            return Err("Secure storage not supported in this mock".to_string());
        }
        
        println!("MockPlatformProvider: Deleting key {}", key);
        match self.secure_storage.write() {
            Ok(mut storage) => {
                let existed = storage.remove(key).is_some();
                if existed {
                    println!("MockPlatformProvider: Deleted key {} successfully", key);
                } else {
                    println!("MockPlatformProvider: Key {} did not exist, nothing to delete", key);
                }
                Ok(())
            }
            Err(_) => {
                let err = "Failed to acquire write lock on secure storage".to_string();
                println!("MockPlatformProvider: ERROR - {}", err);
                Err(err)
            }
        }
    }
    
    fn biometric_auth_available(&self) -> bool {
        self.capabilities.has_biometric_auth
    }
    
    fn authenticate_with_biometrics(&self, _reason: &str) -> Result<bool, String> {
        if !self.capabilities.has_biometric_auth {
            return Err("Biometric authentication not supported in this mock".to_string());
        }
        
        match &self.biometric_auth_result {
            Some(result) => result.clone(),
            None => Ok(true) // Default to successful authentication if not specified
        }
    }
} 