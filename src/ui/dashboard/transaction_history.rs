//! Transaction History Tab
//!
//! Shows list of all transactions with:
//! - Date/time
//! - Amount
//! - Status (pending/confirmed)
//! - Click to view details

use crate::state::{AppState, Navigation, View};
use crate::ui::components::{
    badge, button, card, BadgeStyle, ButtonStyle, Colors, Spacing, Typography,
};
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
    let ctx = ui.ctx().clone();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.add_space(Spacing::MD);

            ui.label(Typography::heading("Transaction History").color(Colors::text_primary(&ctx)));

            if !app_state.is_vault_loaded() {
                card(ui, |ui| {
                    ui.label(
                        Typography::body("No vault loaded").color(Colors::text_secondary(&ctx)),
                    );
                });
                return;
            }

            ui.add_space(Spacing::MD);

            // Filter buttons and refresh button
            TX_HISTORY_STATE.with(|state| {
                let mut state = state.borrow_mut();

                ui.horizontal(|ui| {
                    // Filter buttons with badge styling
                    for filter in TransactionFilter::all() {
                        let is_selected = state.current_filter == filter;
                        let filter_badge = match filter {
                            TransactionFilter::Pending => BadgeStyle::Warning,
                            TransactionFilter::Sent => BadgeStyle::Error,
                            TransactionFilter::Received => BadgeStyle::Success,
                        };

                        if is_selected {
                            badge(ui, filter.title(), filter_badge);
                        } else {
                            let response = ui.selectable_label(false, filter.title());
                            if response.clicked() {
                                state.current_filter = filter;
                                state.apply_filter();
                            }
                        }
                        ui.add_space(Spacing::SM);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if button(ui, "🔄 Refresh", ButtonStyle::Secondary).clicked() {
                            refresh_transactions(ui, app_state);
                        }
                    });
                });
            });

            ui.add_space(Spacing::MD);

            // Get state
            TX_HISTORY_STATE.with(|state| {
                let mut state = state.borrow_mut();

                // Show error if any
                if let Some(ref error) = state.error {
                    card(ui, |ui| {
                        ui.label(Typography::body(error).color(Colors::ERROR));
                    });
                    ui.add_space(Spacing::MD);
                }

                // Show loading indicator
                if state.is_loading {
                    ui.vertical_centered(|ui| {
                        ui.spinner();
                        ui.add_space(Spacing::MD);
                        ui.label(
                            Typography::body("Loading transactions...")
                                .color(Colors::text_secondary(&ctx)),
                        );
                    });
                    return;
                }

                // Apply filter
                state.apply_filter();

                // Show transaction list
                if state.filtered_transactions.is_empty() {
                    card(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(Spacing::XL);
                            ui.label(
                                Typography::heading_small("No transactions yet")
                                    .color(Colors::text_primary(&ctx)),
                            );
                            ui.add_space(Spacing::MD);
                            ui.label(
                                Typography::body(
                                    "When you make transactions, they will appear here",
                                )
                                .color(Colors::text_secondary(&ctx)),
                            );
                            ui.add_space(Spacing::LG);

                            ui.horizontal(|ui| {
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        if button(ui, "Send BTC", ButtonStyle::Primary).clicked() {
                                            navigation.navigate_to(View::SendTransaction);
                                        }
                                        ui.add_space(Spacing::MD);
                                        if button(ui, "Receive BTC", ButtonStyle::Secondary)
                                            .clicked()
                                        {
                                            navigation.navigate_to(View::Receive);
                                        }
                                    },
                                );
                            });
                            ui.add_space(Spacing::XL);
                        });
                    });
                } else {
                    // Transaction cards
                    for tx in &state.filtered_transactions {
                        render_transaction_card(ui, navigation, tx, &ctx);
                        ui.add_space(Spacing::SM);
                    }
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

            ui.add_space(Spacing::LG);
        });
    });
}

fn refresh_transactions(_ui: &mut egui::Ui, app_state: &mut AppState) {
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
                    state.error = Some(format!("Failed to load transactions: {}", crate::utils::sanitize_error_for_ui(&e)));
                    state.is_loading = false;
                }
            }
        } else {
            state.error = Some("Vault not loaded or runtime not available".to_string());
            state.is_loading = false;
        }
    });
}

fn render_transaction_card(
    ui: &mut egui::Ui,
    navigation: &mut Navigation,
    tx: &bitvault_common::types::TransactionInfo,
    ctx: &egui::Context,
) {
    let amount = tx.total_amount_btc();
    let is_positive = amount >= 0.0;

    card(ui, |ui| {
        let response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with(&tx.tx_id),
            egui::Sense::click(),
        );

        // Hover effect
        if response.hovered() {
            ui.painter().rect_filled(
                response.rect,
                12.0,
                if ctx.style().visuals.dark_mode {
                    Colors::GRAY_700
                } else {
                    Colors::GRAY_100
                },
            );
        }

        ui.vertical(|ui| {
            ui.add_space(Spacing::MD);

            // Header row: Status badge and amount
            ui.horizontal(|ui| {
                // Status badge
                let status_badge = match tx.status {
                    bitvault_common::types::TransactionStatus::Pending => BadgeStyle::Warning,
                    bitvault_common::types::TransactionStatus::Sent => BadgeStyle::Error,
                    bitvault_common::types::TransactionStatus::Received => BadgeStyle::Success,
                };
                badge(ui, tx.status.as_str(), status_badge);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Amount (large, prominent)
                    let amount_str = if is_positive {
                        format!("+{:.8} BTC", amount)
                    } else {
                        format!("{:.8} BTC", amount.abs())
                    };
                    ui.label(Typography::heading_small(amount_str).color(if is_positive {
                        Colors::SUCCESS
                    } else {
                        Colors::ERROR
                    }));
                });
            });

            ui.add_space(Spacing::SM);

            // Date/time
            let date_str = if tx.timestamp > 0 {
                if let Some(dt) = Local.timestamp_opt(tx.timestamp, 0).single() {
                    dt.format("%B %d, %Y at %H:%M").to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Pending".to_string()
            };
            ui.label(Typography::body(date_str).color(Colors::text_secondary(ctx)));

            ui.add_space(Spacing::XS);

            // Address (truncated, monospace)
            let address_display = if tx.address.len() > 30 {
                format!(
                    "{}...{}",
                    &tx.address[..15],
                    &tx.address[tx.address.len() - 15..]
                )
            } else {
                tx.address.clone()
            };
            ui.label(
                Typography::caption(address_display)
                    .color(Colors::text_muted(ctx))
                    .monospace(),
            );

            // Description if available
            if let Some(ref desc) = tx.description {
                if !desc.is_empty() {
                    ui.add_space(Spacing::XS);
                    ui.label(Typography::caption(desc).color(Colors::text_secondary(ctx)));
                }
            }

            ui.add_space(Spacing::MD);

            // Details button
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if button(ui, "View Details →", ButtonStyle::Text).clicked() {
                        navigation.navigate_to(View::TransactionDetail {
                            txid: tx.tx_id.clone(),
                        });
                    }
                });
            });

            ui.add_space(Spacing::MD);
        });

        if response.clicked() {
            navigation.navigate_to(View::TransactionDetail {
                txid: tx.tx_id.clone(),
            });
        }
    });
}
