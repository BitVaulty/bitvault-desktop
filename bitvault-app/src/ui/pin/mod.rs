//! PIN entry and setup UI
//!
//! Handles:
//! - PIN entry for authentication
//! - PIN setup during vault creation
//! - PIN confirmation
//! - PIN verification for sensitive operations

mod entry;
mod setup;
mod verification;

pub use entry::{render_pin_entry, PinEntryState};
pub use setup::{render_pin_setup, PinSetupState};
pub use verification::{render_pin_verification, PinVerificationState};
