// Utilities
// Note: UR and derivation utilities are in bitvault-common

/// Sanitize an error for display in the UI (removes paths, keys, mnemonics from error text).
pub fn sanitize_error_for_ui(e: &impl std::fmt::Display) -> String {
    let raw = e.to_string();
    let (bucket, friendly) = bitvault_common::get_readable_error(&raw);
    let message = match bucket {
        bitvault_common::BdkErrorBucket::Backend
        | bitvault_common::BdkErrorBucket::Sync
        | bitvault_common::BdkErrorBucket::Chain
        | bitvault_common::BdkErrorBucket::InsufficientFunds => friendly,
        _ => raw,
    };
    bitvault_common::sanitize_error_message(&message)
}

pub mod camera;
pub mod icons;
pub mod images;
pub mod qr;
