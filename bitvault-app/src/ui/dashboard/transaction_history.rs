//! Transaction History Tab
//!
//! Shows list of all transactions with:
//! - Date/time
//! - Amount
//! - Status (pending/confirmed)
//! - Click to view details

use crate::state::{AppState, Navigation, View};
use chrono::{Local, TimeZone};
use eframe::egui;

/// Transaction filter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionFilter {
    Pending,
    Sent,
    Received,
}

impl TransactionFilter {
    fn all() -> Vec<Self> {
        vec![Self::Pending, Self::Sent, Self::Received]
    }

    fn title(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Sent => "Sent",
            Self::Received => "Received",
        }
    }

    fn matches(&self, tx: &bitvault_common::types::TransactionInfo) -> bool {
        match self {
            Self::Pending => tx.status == bitvault_common::types::TransactionStatus::Pending,
            Self::Sent => tx.status == bitvault_common::types::TransactionStatus::Sent,
            Self::Received => tx.status == bitvault_common::types::TransactionStatus::Received,
        }
    }
}

/// Transaction history state
struct TransactionHistoryState {
    transactions: Vec<bitvault_common::types::TransactionInfo>,
    filtered_transactions: Vec<bitvault_common::types::TransactionInfo>,
    current_filter: TransactionFilter,
    is_loading: bool,
    error: Option<String>,
    last_refresh: Option<std::time::Instant>,
}

impl Default for TransactionHistoryState {
    fn default() -> Self {
        Self {
            transactions: Vec::new(),
            filtered_transactions: Vec::new(),
            current_filter: TransactionFilter::Pending,
            is_loading: false,
            error: None,
            last_refresh: None,
        }
    }
}

impl TransactionHistoryState {
    fn apply_filter(&mut self) {
        self.filtered_transactions = self
            .transactions
            .iter()
            .filter(|tx| self.current_filter.matches(tx))
            .cloned()
            .collect();
    }
}

// Thread-local state for transaction history
thread_local! {
    static TX_HISTORY_STATE: std::cell::RefCell<TransactionHistoryState> =
        std::cell::RefCell::new(TransactionHistoryState::default());
}

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical(|ui| {
        ui.heading("Transaction History");

        if !app_state.is_vault_loaded() {
            ui.label("No vault loaded");
            return;
        }

        ui.separator();

        // Filter buttons and refresh button
        TX_HISTORY_STATE.with(|state| {
            let mut state = state.borrow_mut();

            ui.horizontal(|ui| {
                // Filter buttons
                for filter in TransactionFilter::all() {
                    let is_selected = state.current_filter == filter;
                    let button = if is_selected {
                        ui.selectable_label(true, filter.title())
                    } else {
                        ui.selectable_label(false, filter.title())
                    };

                    if button.clicked() {
                        state.current_filter = filter;
                        state.apply_filter();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Refresh").clicked() {
                        refresh_transactions(ui, app_state);
                    }
                });
            });
        });

        ui.add_space(10.0);

        // Get state
        TX_HISTORY_STATE.with(|state| {
            let mut state = state.borrow_mut();

            // Show error if any
            if let Some(ref error) = state.error {
                ui.colored_label(egui::Color32::RED, error);
                ui.add_space(10.0);
            }

            // Show loading indicator
            if state.is_loading {
                ui.label("Loading transactions...");
                return;
            }

            // Apply filter
            state.apply_filter();

            // Show transaction list
            if state.filtered_transactions.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label("No transactions yet");
                    ui.label("When you make transactions, they will appear here");
                    ui.add_space(20.0);

                    // Buttons - centered
                    let button_width = 120.0;
                    let (rect, _) = ui.allocate_exact_size(
                        egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
                        egui::Sense::click()
                    );
                    let mut button_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                    if button_ui.button("Send BTC").clicked() {
                        navigation.navigate_to(View::SendTransaction);
                    }
                    button_ui.add_space(10.0);
                    if button_ui.button("Receive BTC").clicked() {
                        navigation.navigate_to(View::Receive);
                    }
                });
            } else {
                // Transaction list
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for tx in &state.filtered_transactions {
                            render_transaction_row(ui, navigation, tx);
                            ui.separator();
                        }
                    });
            }

            // Auto-refresh if needed (every 30 seconds)
            if let Some(last_refresh) = state.last_refresh {
                if last_refresh.elapsed().as_secs() > 30 {
                    refresh_transactions(ui, app_state);
                }
            } else {
                // Initial load
                refresh_transactions(ui, app_state);
            }
        });
    });
}

fn refresh_transactions(ui: &mut egui::Ui, app_state: &mut AppState) {
    TX_HISTORY_STATE.with(|state| {
        let mut state = state.borrow_mut();

        if state.is_loading {
            return; // Already loading
        }

        state.is_loading = true;
        state.error = None;

        // Get vault service and runtime
        if let (Some(vault_service), Some(runtime)) =
            (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
        {
            let result = runtime.block_on(async {
                let vs = vault_service.read().await;
                vs.list_transactions().await
            });

            match result {
                Ok(transactions) => {
                    state.transactions = transactions;
                    state.apply_filter(); // Apply filter to newly loaded transactions
                    state.last_refresh = Some(std::time::Instant::now());
                    state.is_loading = false;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to load transactions: {}", e));
                    state.is_loading = false;
                }
            }
        } else {
            state.error = Some("Vault not loaded or runtime not available".to_string());
            state.is_loading = false;
        }
    });
}

fn render_transaction_row(
    ui: &mut egui::Ui,
    navigation: &mut Navigation,
    tx: &bitvault_common::types::TransactionInfo,
) {
    ui.horizontal(|ui| {
        // Status indicator
        let status_color = match tx.status {
            bitvault_common::types::TransactionStatus::Pending => egui::Color32::YELLOW,
            bitvault_common::types::TransactionStatus::Sent => egui::Color32::RED,
            bitvault_common::types::TransactionStatus::Received => egui::Color32::GREEN,
        };
        ui.colored_label(status_color, tx.status.as_str());

        ui.separator();

        // Date/time
        let date_str = if tx.timestamp > 0 {
            if let Some(dt) = Local.timestamp_opt(tx.timestamp, 0).single() {
                dt.format("%Y-%m-%d %H:%M").to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Pending".to_string()
        };
        ui.label(date_str);

        ui.separator();

        // Amount
        let amount = tx.total_amount_btc();
        let amount_str = if amount >= 0.0 {
            format!("+{:.8} BTC", amount)
        } else {
            format!("{:.8} BTC", amount)
        };
        let amount_color = if amount >= 0.0 {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };
        ui.colored_label(amount_color, amount_str);

        ui.separator();

        // Address (truncated)
        let address_display = if tx.address.len() > 20 {
            format!(
                "{}...{}",
                &tx.address[..10],
                &tx.address[tx.address.len() - 10..]
            )
        } else {
            tx.address.clone()
        };
        ui.label(address_display);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Details").clicked() {
                navigation.navigate_to(View::TransactionDetail {
                    txid: tx.tx_id.clone(),
                });
            }
        });
    });
}
