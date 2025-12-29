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

pub use entry::{PinEntryState, render_pin_entry};
pub use setup::{PinSetupState, render_pin_setup};
pub use verification::{PinVerificationState, render_pin_verification};
