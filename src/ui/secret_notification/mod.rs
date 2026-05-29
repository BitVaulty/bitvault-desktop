//! Secret Notifications UI
//!
//! UI for setting up Telegram secret notifications

mod state;
mod view;

pub use state::{SecretNotificationPhase, SecretNotificationState};
pub use view::render;
