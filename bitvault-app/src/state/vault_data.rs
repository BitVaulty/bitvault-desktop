//! Vault data cache
//!
//! Caches vault data (balance, address) to avoid repeated async calls
//! and to work with egui's immediate mode UI

use std::sync::{Arc, Mutex};

/// Cached vault data
#[derive(Debug, Clone, Default)]
pub struct VaultData {
    /// Confirmed balance in satoshis
    pub confirmed_balance: Option<u64>,
    /// Available balance in satoshis
    pub available_balance: Option<u64>,
    /// Current receive address
    pub receive_address: Option<String>,
    /// Last update timestamp
    pub last_update: Option<std::time::Instant>,
    /// Loading state
    pub is_loading: bool,
}

impl VaultData {
    /// Create new empty vault data
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if data needs refresh (older than 30 seconds)
    pub fn needs_refresh(&self) -> bool {
        if self.is_loading {
            return false; // Don't refresh if already loading
        }
        self.last_update
            .map(|t| t.elapsed().as_secs() > 30)
            .unwrap_or(true)
    }

    /// Update balance
    pub fn update_balance(&mut self, confirmed: u64, available: u64) {
        self.confirmed_balance = Some(confirmed);
        self.available_balance = Some(available);
        self.last_update = Some(std::time::Instant::now());
        self.is_loading = false;
    }

    /// Update address
    pub fn update_address(&mut self, address: String) {
        self.receive_address = Some(address);
        self.last_update = Some(std::time::Instant::now());
        self.is_loading = false;
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Format balance as BTC string
    pub fn format_balance_btc(&self) -> String {
        if let Some(confirmed) = self.confirmed_balance {
            format!("{:.8} BTC", confirmed as f64 / 100_000_000.0)
        } else {
            "Loading...".to_string()
        }
    }

    /// Format available balance as BTC string
    pub fn format_available_btc(&self) -> String {
        if let Some(available) = self.available_balance {
            format!("{:.8} BTC", available as f64 / 100_000_000.0)
        } else {
            "Loading...".to_string()
        }
    }
}

/// Shared vault data (for async updates)
pub type SharedVaultData = Arc<Mutex<VaultData>>;
