//! Recovery and UTXO Refresh UI Flows
//!
//! Handles:
//! - Recovery transaction flow (for UTXOs older than 1 year)
//! - UTXO refresh flow (for UTXOs older than 6 months)
//! - QR code generation and scanning for sharing PSBTs

mod recovery_flow;
mod utxo_refresh_flow;
mod utxo_selection;

pub use recovery_flow::go_back_in_recovery_workflow;
pub use recovery_flow::render as render_recovery;
pub use utxo_refresh_flow::go_back_in_utxo_refresh_workflow;
pub use utxo_refresh_flow::render as render_utxo_refresh;
