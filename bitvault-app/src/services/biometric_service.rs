//! Biometric Authentication Service
//!
//! Platform-specific biometric authentication:
//! - Windows: Windows Hello (face recognition, fingerprint, PIN)
//! - macOS: Touch ID, Face ID
//! - Linux: Not supported (falls back to PIN)

use std::sync::Arc;
use tokio::sync::Mutex;

/// Biometric authentication result
#[derive(Debug, Clone)]
pub enum BiometricResult {
    Success,
    Cancelled,
    Failed(String),
    NotAvailable,
    NotEnrolled,
}

/// Biometric type available on the platform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiometricType {
    WindowsHello,
    TouchID,
    FaceID,
    None,
}

impl BiometricType {
    pub fn display_name(&self) -> &'static str {
        match self {
            BiometricType::WindowsHello => "Windows Hello",
            BiometricType::TouchID => "Touch ID",
            BiometricType::FaceID => "Face ID",
            BiometricType::None => "None",
        }
    }
}

/// Biometric service for platform-specific authentication
pub struct BiometricService {
    enabled: Arc<Mutex<bool>>,
}

impl BiometricService {
    /// Create a new biometric service
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(Mutex::new(true)), // Enabled by default if available
        }
    }

    /// Check if biometrics are available on this platform
    pub async fn is_available(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            // Windows Hello is available if robius-authentication works
            true // Will be checked during actual authentication
        }

        #[cfg(target_os = "macos")]
        {
            // Touch ID/Face ID availability
            true // Will be checked during actual authentication
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            false // Not supported on Linux
        }
    }

    /// Get the biometric type available on this platform
    pub async fn get_biometric_type(&self) -> BiometricType {
        #[cfg(target_os = "windows")]
        {
            BiometricType::WindowsHello
        }

        #[cfg(target_os = "macos")]
        {
            // Try to detect Touch ID vs Face ID
            // For now, return TouchID as default (macOS desktop typically uses Touch ID)
            BiometricType::TouchID
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            BiometricType::None
        }
    }

    /// Check if biometrics are enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().await
    }

    /// Enable or disable biometrics
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().await = enabled;
    }

    /// Authenticate using biometrics
    /// Returns BiometricResult indicating success or failure
    pub async fn authenticate(&self, reason: &str) -> BiometricResult {
        if !self.is_enabled().await {
            return BiometricResult::NotAvailable;
        }

        if !self.is_available().await {
            return BiometricResult::NotAvailable;
        }

        #[cfg(target_os = "windows")]
        {
            self.authenticate_windows(reason).await
        }

        #[cfg(target_os = "macos")]
        {
            self.authenticate_macos(reason).await
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            BiometricResult::NotAvailable
        }
    }

    #[cfg(target_os = "windows")]
    async fn authenticate_windows(&self, _reason: &str) -> BiometricResult {
        // TODO: Implement Windows Hello authentication
        // This requires a stable biometrics crate or platform-specific bindings
        // For now, return NotAvailable as a placeholder
        // Future implementation would use Windows Hello API
        BiometricResult::NotAvailable
    }

    #[cfg(target_os = "macos")]
    async fn authenticate_macos(&self, _reason: &str) -> BiometricResult {
        // TODO: Implement macOS Touch ID/Face ID authentication
        // This requires platform-specific bindings to LocalAuthentication framework
        // For now, return NotAvailable as a placeholder
        // Future implementation would use objc2-local-authentication or similar
        BiometricResult::NotAvailable
    }
}

impl Default for BiometricService {
    fn default() -> Self {
        Self::new()
    }
}
