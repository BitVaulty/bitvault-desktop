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
            self.is_windows_hello_available().await
        }

        #[cfg(target_os = "macos")]
        {
            self.is_macos_biometrics_available().await
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            false // Not supported on Linux
        }
    }

    #[cfg(target_os = "windows")]
    async fn is_windows_hello_available(&self) -> bool {
        use windows::{
            Security::Credentials::UI::UserConsentVerifier,
            Foundation::IAsyncOperation,
        };

        match UserConsentVerifier::CheckAvailabilityAsync() {
            Ok(operation) => {
                match operation.get().await {
                    Ok(availability) => {
                        matches!(
                            availability,
                            windows::Security::Credentials::UI::UserConsentVerificationAvailability::Available
                        )
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    #[cfg(target_os = "macos")]
    async fn is_macos_biometrics_available(&self) -> bool {
        use objc2::rc::Retained;
        use objc2::runtime::AnyClass;
        use objc2::{class, msg_send};
        use std::ptr;

        // LAPolicyDeviceOwnerAuthenticationWithBiometrics = 1
        const LA_POLICY_BIOMETRICS: i64 = 1;

        unsafe {
            // Get LAContext class
            let la_context_class = match AnyClass::get(c"LAContext") {
                Some(cls) => cls,
                None => return false, // LocalAuthentication framework not available
            };

            // Create LAContext instance
            let context: *mut objc2::runtime::AnyObject = msg_send![la_context_class, alloc];
            let context: *mut objc2::runtime::AnyObject = msg_send![context, init];
            
            if context.is_null() {
                return false;
            }

            // Check if biometrics can be evaluated
            let mut error: *mut objc2::runtime::AnyObject = ptr::null_mut();
            let can_evaluate: bool = msg_send![
                context,
                canEvaluatePolicy: LA_POLICY_BIOMETRICS
                error: &mut error
            ];

            // Release context
            let _: () = msg_send![context, release];

            can_evaluate
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
    async fn authenticate_windows(&self, reason: &str) -> BiometricResult {
        use windows::{
            core::HSTRING,
            Security::Credentials::UI::{
                UserConsentVerifier, UserConsentVerificationAvailability,
                UserConsentVerificationResult,
            },
            Foundation::IAsyncOperation,
        };

        // Check if Windows Hello is available
        let availability_op = match UserConsentVerifier::CheckAvailabilityAsync() {
            Ok(op) => op,
            Err(_) => return BiometricResult::NotAvailable,
        };

        let availability = match availability_op.get().await {
            Ok(avail) => avail,
            Err(_) => return BiometricResult::NotAvailable,
        };

        match availability {
            UserConsentVerificationAvailability::Available => {
                // Request authentication
                let reason_hstring = HSTRING::from(reason);
                let auth_op = match UserConsentVerifier::RequestVerificationAsync(&reason_hstring) {
                    Ok(op) => op,
                    Err(_) => return BiometricResult::Failed("Failed to start authentication".to_string()),
                };

                let result = match auth_op.get().await {
                    Ok(res) => res,
                    Err(_) => return BiometricResult::Failed("Authentication operation failed".to_string()),
                };

                match result {
                    UserConsentVerificationResult::Verified => BiometricResult::Success,
                    UserConsentVerificationResult::Canceled => BiometricResult::Cancelled,
                    UserConsentVerificationResult::DeviceNotPresent => BiometricResult::NotAvailable,
                    UserConsentVerificationResult::NotConfiguredForUser => BiometricResult::NotEnrolled,
                    _ => BiometricResult::Failed("Authentication failed".to_string()),
                }
            }
            UserConsentVerificationAvailability::DeviceNotPresent => BiometricResult::NotAvailable,
            UserConsentVerificationAvailability::NotConfiguredForUser => BiometricResult::NotEnrolled,
            _ => BiometricResult::NotAvailable,
        }
    }

    #[cfg(target_os = "macos")]
    async fn authenticate_macos(&self, reason: &str) -> BiometricResult {
        use objc2::runtime::AnyClass;
        use objc2::{class, msg_send};
        use objc2_foundation::NSString;
        use std::ptr;
        use std::sync::mpsc;
        use std::time::Duration;

        // LAPolicyDeviceOwnerAuthenticationWithBiometrics = 1
        const LA_POLICY_BIOMETRICS: i64 = 1;

        // LAError codes
        const LA_ERROR_AUTHENTICATION_FAILED: i64 = -1;
        const LA_ERROR_USER_CANCEL: i64 = -2;
        const LA_ERROR_USER_FALLBACK: i64 = -3;
        const LA_ERROR_SYSTEM_CANCEL: i64 = -4;
        const LA_ERROR_PASSCODE_NOT_SET: i64 = -5;
        const LA_ERROR_BIOMETRY_NOT_AVAILABLE: i64 = -6;
        const LA_ERROR_BIOMETRY_NOT_ENROLLED: i64 = -7;
        const LA_ERROR_BIOMETRY_LOCKOUT: i64 = -8;

        // Create a channel for the async result
        let (tx, rx) = mpsc::channel::<BiometricResult>();

        let reason_string = reason.to_string();

        // Spawn blocking task for Objective-C interaction
        // Note: LAContext authentication must happen on the main thread
        tokio::task::spawn_blocking(move || {
            unsafe {
                // Get LAContext class
                let la_context_class = match AnyClass::get(c"LAContext") {
                    Some(cls) => cls,
                    None => {
                        let _ = tx.send(BiometricResult::NotAvailable);
                        return;
                    }
                };

                // Create LAContext instance
                let context: *mut objc2::runtime::AnyObject = msg_send![la_context_class, alloc];
                let context: *mut objc2::runtime::AnyObject = msg_send![context, init];

                if context.is_null() {
                    let _ = tx.send(BiometricResult::NotAvailable);
                    return;
                }

                // Check if biometrics can be evaluated
                let mut error: *mut objc2::runtime::AnyObject = ptr::null_mut();
                let can_evaluate: bool = msg_send![
                    context,
                    canEvaluatePolicy: LA_POLICY_BIOMETRICS
                    error: &mut error
                ];

                if !can_evaluate {
                    let _: () = msg_send![context, release];
                    
                    if !error.is_null() {
                        let error_code: i64 = msg_send![error, code];
                        let result = match error_code {
                            LA_ERROR_BIOMETRY_NOT_AVAILABLE => BiometricResult::NotAvailable,
                            LA_ERROR_BIOMETRY_NOT_ENROLLED => BiometricResult::NotEnrolled,
                            LA_ERROR_PASSCODE_NOT_SET => BiometricResult::NotEnrolled,
                            _ => BiometricResult::NotAvailable,
                        };
                        let _ = tx.send(result);
                    } else {
                        let _ = tx.send(BiometricResult::NotAvailable);
                    }
                    return;
                }

                // Create NSString for reason
                let reason_nsstring = NSString::from_str(&reason_string);

                // For synchronous evaluation, we use evaluatePolicy:localizedReason:reply:
                // which requires a block. Since block2 may have compatibility issues,
                // we'll use a simpler approach with evaluateAccessControl on Sonoma+
                // or fall back to a placeholder for older systems.
                //
                // For production, this should use dispatch_semaphore or run on main thread
                // with proper block handling.
                
                // Simplified approach: Just check availability (already did) and return success
                // Real implementation would require block2 or dispatch handling
                let _: () = msg_send![context, release];
                
                // For now, if we got this far, biometrics are available
                // A full implementation would call evaluatePolicy:localizedReason:reply:
                // with a block callback. Without block2, we return a simulated success
                // that indicates the system supports biometrics.
                //
                // TODO: Implement full authentication with block2 when available
                let _ = tx.send(BiometricResult::NotAvailable);
            }
        });

        // Wait for result with timeout
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(result) => result,
            Err(_) => BiometricResult::Failed("Authentication timeout".to_string()),
        }
    }
}

impl Default for BiometricService {
    fn default() -> Self {
        Self::new()
    }
}
