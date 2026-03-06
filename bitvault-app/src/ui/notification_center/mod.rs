//! Notification Center UI
//!
//! UI for displaying application notifications (transactions, alerts, etc.)

mod state;
mod view;

pub use state::NotificationCenterState;
pub use view::render;
