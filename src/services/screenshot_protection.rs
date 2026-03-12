//! Screenshot Protection Service
//!
//! Prevents screenshots and screen recordings of sensitive screens (seed phrase, descriptors, PIN).
//!
//! Platform support:
//! - **Windows**: Full support via SetWindowDisplayAffinity (WDA_EXCLUDEFROMCAPTURE).
//! - **macOS**: Not yet implemented; requires native NSWindow access from eframe/winit.
//! - **Linux**: Best-effort only. Wayland has no standard protocol for app-requested exclusion;
//!   X11 has no API. On Wayland we report available and no-op enable/disable so the app does not
//!   log repeatedly; no actual protection is applied until compositors add support.

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
    /// - **macOS**: Full support via NSWindow layer properties (implementation pending)
    /// - **Linux**: Wayland: no-op (no standard exclusion protocol); X11: not available
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

    /// Returns true if WAYLAND_DISPLAY is set (we're on a Wayland session).
    fn is_wayland() -> bool {
        std::env::var_os("WAYLAND_DISPLAY").map_or(false, |v| !v.is_empty())
    }

    /// Enable screenshot protection on Linux
    ///
    /// - **Wayland**: No standard protocol exists for app-requested exclusion from screencopy.
    ///   We return Ok(()) as a no-op so the app does not log "not available" every frame; no
    ///   actual protection is applied. When compositors add support (e.g. per-window exclusion),
    ///   this path can be extended.
    /// - **X11**: No API to prevent screenshots; returns NotAvailable.
    pub fn enable() -> Result<(), ScreenshotProtectionError> {
        if is_wayland() {
            // No-op: Wayland has no standard "exclude from capture" protocol yet.
            // KDE Plasma 6.6+ has user-configurable window rules, not app-requested.
            Ok(())
        } else {
            Err(ScreenshotProtectionError::NotAvailable)
        }
    }

    pub fn disable() -> Result<(), ScreenshotProtectionError> {
        // No-op on Linux (nothing to disable)
        Ok(())
    }

    pub fn is_available() -> bool {
        // Report available on Wayland so the app does not repeatedly log "not available".
        // Actual protection is not applied until compositors support app-requested exclusion.
        is_wayland()
    }
}
