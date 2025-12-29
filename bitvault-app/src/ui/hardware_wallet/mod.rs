//! Hardware Wallet QR Code UI Components
//!
//! Handles:
//! - Displaying QR codes for hardware wallets to scan (multi-part UR)
//! - Scanning QR codes from hardware wallets (for signed PSBTs)

mod qr_display;
mod qr_scanner;

pub use qr_display::{render_qr_display, QrDisplayState};
pub use qr_scanner::{render_qr_scanner, QrScannerState};
