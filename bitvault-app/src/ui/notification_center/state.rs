//! State for notification center

use chrono::{DateTime, Utc};

/// Notification data structure
#[derive(Clone, Debug)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notification_type: NotificationType,
}

/// Type of notification
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotificationType {
    Transaction,
    Alert,
    Info,
}

/// State for notification center
#[derive(Default)]
pub struct NotificationCenterState {
    pub notifications: Vec<Notification>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub last_fetch: Option<DateTime<Utc>>,
}

impl NotificationCenterState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Fetch notifications from vault service (using transaction history)
    pub async fn fetch_notifications(
        &mut self,
        vault_service: &std::sync::Arc<tokio::sync::RwLock<bitvault_common::wallet::VaultService>>,
    ) {
        self.is_loading = true;
        self.error = None;

        let guard = vault_service.read().await;
        match guard.list_transactions().await {
            Ok(transactions) => {
                let now = Utc::now();
                self.notifications = transactions
                    .into_iter()
                    .map(|tx| {
                        let title = match tx.status.as_str() {
                            "confirmed" => "Transaction Confirmed",
                            "pending" => "Transaction Pending",
                            "sent" => "Transaction Sent",
                            "received" => "Transaction Received",
                            _ => "Transaction Update",
                        };

                        let amount = if tx.amount_received_btc > 0.0 {
                            format!("Received {:.8} BTC", tx.amount_received_btc)
                        } else if tx.amount_sent_btc > 0.0 {
                            format!("Sent {:.8} BTC", tx.amount_sent_btc)
                        } else {
                            "Amount: 0 BTC".to_string()
                        };

                        Notification {
                            id: tx.tx_id.clone(),
                            title: title.to_string(),
                            body: format!("{} - {}", amount, tx.tx_id),
                            created_at: DateTime::<Utc>::from_timestamp(tx.timestamp, 0)
                                .unwrap_or_else(Utc::now),
                            expires_at: None, // Transactions don't expire
                            notification_type: NotificationType::Transaction,
                        }
                    })
                    .collect();

                // Sort by most recent first
                self.notifications.sort_by(|a, b| {
                    b.created_at.cmp(&a.created_at)
                });

                self.last_fetch = Some(now);
            }
            Err(e) => {
                self.error = Some(format!("Failed to fetch notifications: {}", e));
            }
        }

        self.is_loading = false;
    }
}
