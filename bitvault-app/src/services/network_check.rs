//! Simple connectivity check before network-dependent operations
//! Matches mobile NetworkMonitor behavior: block flows when offline
//!
//! Thin wrapper around bitvault_common::check_connectivity for backward compatibility.

/// Check if we have network connectivity (can reach the API).
/// Uses a HEAD request to the default BitVault API.
pub async fn check_connectivity() -> bool {
    bitvault_common::check_connectivity(None).await
}
