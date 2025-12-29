//! Desktop notification service
//!
//! Provides desktop notifications for important events like:
//! - Transaction sent/received/confirmed
//! - Subscription status changes
//! - Vault events

use notify_rust::Notification;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Notification service for desktop notifications
pub struct NotificationService {
    enabled: Arc<Mutex<bool>>,
}

impl NotificationService {
    /// Create a new notification service
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(Mutex::new(true)), // Enabled by default
        }
    }

    /// Check if notifications are enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().await
    }

    /// Enable or disable notifications
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().await = enabled;
    }

    /// Show a notification
    /// Returns Ok(()) if notification was shown, Err if failed or disabled
    pub async fn notify(&self, title: &str, body: &str) -> Result<(), String> {
        if !self.is_enabled().await {
            return Ok(()); // Silently skip if disabled
        }

        match Notification::new()
            .summary(title)
            .body(body)
            .appname("BitVault")
            .timeout(notify_rust::Timeout::Milliseconds(5000)) // 5 seconds
            .show() {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to show notification: {}", e)),
        }
    }

    /// Show transaction sent notification
    pub async fn notify_transaction_sent(&self, txid: &str, amount_btc: f64) -> Result<(), String> {
        let title = "Transaction Sent";
        let body = format!("Sent {:.8} BTC\nTXID: {}", amount_btc, &txid[..txid.len().min(16)]);
        self.notify(title, &body).await
    }

    /// Show transaction received notification
    pub async fn notify_transaction_received(&self, amount_btc: f64) -> Result<(), String> {
        let title = "Transaction Received";
        let body = format!("Received {:.8} BTC", amount_btc);
        self.notify(title, &body).await
    }

    /// Show transaction confirmed notification
    pub async fn notify_transaction_confirmed(&self, txid: &str) -> Result<(), String> {
        let title = "Transaction Confirmed";
        let body = format!("Transaction confirmed\nTXID: {}", &txid[..txid.len().min(16)]);
        self.notify(title, &body).await
    }

    /// Show subscription expired notification
    pub async fn notify_subscription_expired(&self) -> Result<(), String> {
        let title = "Subscription Expired";
        let body = "Your subscription has expired. Please renew to continue using mainnet features.";
        self.notify(title, body).await
    }

    /// Show subscription renewed notification
    pub async fn notify_subscription_renewed(&self) -> Result<(), String> {
        let title = "Subscription Renewed";
        let body = "Your subscription has been successfully renewed.";
        self.notify(title, body).await
    }

    /// Show vault created notification
    pub async fn notify_vault_created(&self, vault_name: &str) -> Result<(), String> {
        let title = "Vault Created";
        let body = format!("Vault '{}' has been created successfully.", vault_name);
        self.notify(title, &body).await
    }

    /// Show error notification
    pub async fn notify_error(&self, error: &str) -> Result<(), String> {
        let title = "Error";
        self.notify(title, error).await
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}
