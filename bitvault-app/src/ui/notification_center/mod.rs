//! Notification Center UI
//!
//! UI for displaying application notifications (transactions, alerts, etc.)

mod view;
mod state;

pub use view::render;
pub use state::NotificationCenterState;
