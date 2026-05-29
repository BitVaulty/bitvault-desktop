//! Vault creation step implementations

mod common;
mod main_coowner;
mod restore;
mod shared;
mod view_only;

pub use common::render_name_hardware_wallet;

// Re-export all render_* functions for vault_creation::render
pub use main_coowner::{
    render_create_vault,
    render_display_exchange_data,
    render_display_own_keys,
    render_enter_exchange_data,
    render_email_auth,
    render_scan_coowner_keys,
};
pub use restore::{
    render_enter_seed_phrase,
    render_scan_descriptor_restore,
    render_select_seed_phrase_size,
};
pub use shared::{
    render_completed,
    render_display_seed_phrase,
    render_legal_acknowledgment,
    render_mnemonic_generation,
    render_name_vault,
    render_role_selection,
    render_set_pin,
    render_set_time_delay,
    render_verify_seed_phrase,
    LegalDocumentKind,
};
pub use view_only::{
    render_scan_descriptor_view_only,
    render_view_only_complete,
};
