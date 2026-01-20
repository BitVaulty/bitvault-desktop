//! Hardware Wallet QR Code UI Components
//!
//! Handles:
//! - Displaying QR codes for hardware wallets to scan (multi-part UR)
//! - Scanning QR codes from hardware wallets (for signed PSBTs)
//! - Batch scanning for multi-part UR codes (Jade, Passport)

mod batch_qr_scanner;
mod qr_display;
mod qr_scanner;

pub use batch_qr_scanner::BatchQrScannerState;
pub use batch_qr_scanner::render_batch_qr_scanner;
pub use qr_display::{render_qr_display, QrDisplayState};
pub use qr_scanner::QrScannerState;
