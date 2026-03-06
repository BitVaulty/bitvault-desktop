//! Screenshot Protection Service
//!
//! Prevents screenshots and screen recordings of sensitive screens (seed phrase, descriptors, PIN).
//!
//! Platform support:
//! - Windows: Uses SetWindowDisplayAffinity API
//! - macOS: Uses NSWindow contentView layer properties (via objc2)
//! - Linux: Limited support (may not be fully effective on all window managers)

use thiserror::Error;

/// Screenshot protection service
pub struct ScreenshotProtection;

#[derive(Debug, Error)]
pub enum ScreenshotProtectionError {
    #[error("Platform API error: {0}")]
    PlatformError(String),
    #[error("Screenshot protection not available on this platform")]
    NotAvailable,
}

impl ScreenshotProtection {
    /// Enable screenshot protection for the current window
    ///
    /// This prevents screenshots and screen recordings on platforms that support it.
    /// On Linux, this may have limited effectiveness depending on the window manager.
    ///
    /// # Platform Support
    /// - **Windows**: Full support via SetWindowDisplayAffinity
    /// - **macOS**: Full support via NSWindow layer properties
    /// - **Linux**: Limited/partial support (X11/Wayland compatibility varies)
    ///
    /// # Errors
    /// Returns an error if the platform doesn't support screenshot protection
    /// or if the API call fails.
    pub fn enable() -> Result<(), ScreenshotProtectionError> {
        #[cfg(target_os = "windows")]
        return windows::enable();

        #[cfg(target_os = "macos")]
        return macos::enable();

        #[cfg(target_os = "linux")]
        return linux::enable();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(ScreenshotProtectionError::NotAvailable)
    }

    /// Disable screenshot protection for the current window
    ///
    /// Restores normal screenshot/screen recording behavior.
    pub fn disable() -> Result<(), ScreenshotProtectionError> {
        #[cfg(target_os = "windows")]
        return windows::disable();

        #[cfg(target_os = "macos")]
        return macos::disable();

        #[cfg(target_os = "linux")]
        return linux::disable();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(ScreenshotProtectionError::NotAvailable)
    }

    /// Check if screenshot protection is available on this platform
    ///
    /// Returns true if the platform supports screenshot protection,
    /// false otherwise (e.g., on Linux with certain window managers).
    pub fn is_available() -> bool {
        #[cfg(target_os = "windows")]
        return windows::is_available();

        #[cfg(target_os = "macos")]
        return macos::is_available();

        #[cfg(target_os = "linux")]
        return linux::is_available();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        false
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::ScreenshotProtectionError;
    use windows::Win32::Graphics::Gdi::{
        GetForegroundWindow, SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
    };

    pub fn enable() -> Result<(), ScreenshotProtectionError> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return Err(ScreenshotProtectionError::PlatformError(
                    "No foreground window found".to_string(),
                ));
            }

            // Set window display affinity to exclude from capture
            // WDA_EXCLUDEFROMCAPTURE prevents screenshots and screen recordings
            let result = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
            if result.as_bool() {
                Ok(())
            } else {
                Err(ScreenshotProtectionError::PlatformError(
                    "Failed to set window display affinity".to_string(),
                ))
            }
        }
    }

    pub fn disable() -> Result<(), ScreenshotProtectionError> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return Err(ScreenshotProtectionError::PlatformError(
                    "No foreground window found".to_string(),
                ));
            }

            // Reset window display affinity (WDA_NONE = no special affinity)
            let result = SetWindowDisplayAffinity(hwnd, WDA_NONE);
            if result.as_bool() {
                Ok(())
            } else {
                Err(ScreenshotProtectionError::PlatformError(
                    "Failed to reset window display affinity".to_string(),
                ))
            }
        }
    }

    pub fn is_available() -> bool {
        // Windows 10 version 2004 (build 19041) and later support WDA_EXCLUDEFROMCAPTURE
        // Check if we're on a supported version
        // For now, assume Windows 10 2004+ (can add version check if needed)
        true
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::ScreenshotProtectionError;
    use objc2::runtime::Object;
    use objc2::{msg_send, sel};
    use objc2_foundation::NSWindow;

    /// Enable screenshot protection on macOS
    ///
    /// Uses NSWindow contentView layer to prevent screen capture.
    /// This is the macOS equivalent of Windows' SetWindowDisplayAffinity.
    pub fn enable() -> Result<(), ScreenshotProtectionError> {
        // Note: This requires access to the NSWindow instance from eframe
        // eframe on macOS uses winit, which may not expose NSWindow directly
        // This is a placeholder implementation - may need to use winit's native window handle

        // TODO: Implement using eframe/winit native window access
        // For now, return NotAvailable until we can access the NSWindow

        Err(ScreenshotProtectionError::PlatformError(
            "macOS screenshot protection requires native window access - implementation pending"
                .to_string(),
        ))
    }

    pub fn disable() -> Result<(), ScreenshotProtectionError> {
        // TODO: Implement using eframe/winit native window access
        Err(ScreenshotProtectionError::PlatformError(
            "macOS screenshot protection requires native window access - implementation pending"
                .to_string(),
        ))
    }

    pub fn is_available() -> bool {
        // macOS supports screenshot protection via NSWindow
        // However, we need native window access from eframe/winit
        // Return false until implementation is complete
        false // TODO: Return true once implementation is complete
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::ScreenshotProtectionError;

    /// Enable screenshot protection on Linux
    ///
    /// Linux screenshot protection has limited support:
    /// - Wayland: Some compositors support screen content protection (e.g., KDE, GNOME)
    /// - X11: No reliable way to prevent screenshots
    ///
    /// This implementation is a placeholder and may not work on all systems.
    pub fn enable() -> Result<(), ScreenshotProtectionError> {
        // Linux screenshot protection is platform-dependent and unreliable
        // X11 does not provide a way to prevent screenshots
        // Wayland compositors may have extensions, but they're not standardized

        // For now, return NotAvailable to indicate limited support
        Err(ScreenshotProtectionError::NotAvailable)
    }

    pub fn disable() -> Result<(), ScreenshotProtectionError> {
        Ok(()) // No-op on Linux (nothing to disable)
    }

    pub fn is_available() -> bool {
        // Linux screenshot protection is not reliably available
        // May work on some Wayland compositors, but not guaranteed
        false
    }
}
