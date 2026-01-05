//! Vault Detail Tab
//!
//! Shows:
//! - Balance (confirmed/available)
//! - Vault address
//! - Recent transactions
//! - Quick actions (Send, Receive buttons)

use crate::state::{AppState, Navigation};
use chrono::{Local, TimeZone};
use eframe::egui;

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);

        if !app_state.is_vault_loaded() {
            ui.heading("No Vault Loaded");
            ui.label("Create or import a vault to get started");
            ui.add_space(10.0);
            if ui.button("Create Vault").clicked() {
                navigation.navigate_to(crate::state::View::VaultCreation);
            }
            return;
        }

        // Get vault data (read from shared state)
        let vault_data = match app_state.vault_data.lock() {
            Ok(data) => data.clone(),
            Err(_) => {
                ui.label("Error: Mutex poisoned");
                return;
            }
        };

        // Balance display
        ui.heading("Balance");
        ui.add_space(10.0);

        // Display cached balance or loading
        let balance_text = vault_data.format_balance_btc();
        ui.label(format!("Confirmed: {}", balance_text));

        let available_text = vault_data.format_available_btc();
        ui.label(format!("Available: {}", available_text));

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() || vault_data.needs_refresh() {
                if let Some(ref mut handler) = app_state.async_handler {
                    handler.fetch_balance();
                    handler.fetch_address();
                }
            }

            if ui.button("Switch Vault").clicked() {
                navigation.navigate_to(crate::state::View::VaultSelection);
            }
        });

        ui.add_space(20.0);

        // Vault address
        ui.heading("Vault Address");
        ui.add_space(10.0);

        if let Some(ref address) = vault_data.receive_address {
            ui.label(address);
            if ui.button("Copy").clicked() {
                ui.output_mut(|o| {
                    o.copied_text = address.clone();
                });
            }
        } else {
            ui.label("Loading address...");
            // Trigger async address fetch on first render
            if !vault_data.is_loading {
                if let Some(ref mut handler) = app_state.async_handler {
                    handler.fetch_address();
                }
            }
        }

        ui.add_space(30.0);

        // Quick actions
        ui.horizontal(|ui| {
            if ui.button("Send").clicked() {
                navigation.navigate_to(crate::state::View::SendTransaction);
            }
            if ui.button("Receive").clicked() {
                navigation.navigate_to(crate::state::View::Receive);
            }
        });

        ui.add_space(20.0);

        // Recent transactions
        ui.heading("Recent Transactions");

        // Fetch and display recent transactions
        if let (Some(vault_service), Some(runtime)) =
            (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
        {
            let result = runtime.block_on(async {
                let vs = vault_service.read().await;
                vs.list_transactions().await
            });

            match result {
                Ok(transactions) => {
                    if transactions.is_empty() {
                        ui.label("No transactions yet");
                        ui.label("When you make transactions, they will appear here");
                    } else {
                        // Show up to 5 most recent transactions
                        let recent_txs: Vec<_> = transactions.iter().take(5).collect();

                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for tx in recent_txs {
                                    ui.horizontal(|ui| {
                                        // Status
                                        let status_icon = match tx.status {
                                            bitvault_common::types::TransactionStatus::Pending => {
                                                "⏳"
                                            }
                                            bitvault_common::types::TransactionStatus::Sent => "📤",
                                            bitvault_common::types::TransactionStatus::Received => {
                                                "📥"
                                            }
                                        };
                                        ui.label(status_icon);

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

                                        // Date
                                        if tx.timestamp > 0 {
                                            if let Some(dt) =
                                                Local.timestamp_opt(tx.timestamp, 0).single()
                                            {
                                                ui.label(dt.format("%m/%d").to_string());
                                            }
                                        }

                                        // Click to view details
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("→").clicked() {
                                                    navigation.navigate_to(
                                                        crate::state::View::TransactionDetail {
                                                            txid: tx.tx_id.clone(),
                                                        },
                                                    );
                                                }
                                            },
                                        );
                                    });
                                    ui.separator();
                                }
                            });

                        if transactions.len() > 5 {
                            ui.add_space(5.0);
                            if ui.button("View All Transactions").clicked() {
                                navigation.set_dashboard_tab(1); // Switch to transaction history tab
                            }
                        }
                    }
                }
                Err(e) => {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("Failed to load transactions: {}", e),
                    );
                }
            }
        } else {
            ui.label("Vault not loaded");
        }
    });
}

// Note: Async data fetching is implemented using AsyncCommandHandler
// which uses block_on for quick operations (balance, address fetching)
// This is acceptable for egui's immediate mode UI
