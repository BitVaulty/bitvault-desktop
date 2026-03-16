// Utilities
// Note: UR and derivation utilities are in bitvault-common

/// Sanitize an error for display in the UI (removes paths, keys, mnemonics from error text).
pub fn sanitize_error_for_ui(e: &impl std::fmt::Display) -> String {
    bitvault_common::sanitize_error_message(&e.to_string())
}

pub mod camera;
pub mod icons;
pub mod images;
pub mod qr;
