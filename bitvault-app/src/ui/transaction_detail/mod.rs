//! Transaction Detail View
//!
//! Displays detailed information about a specific transaction

use crate::state::{AppState, Navigation};
use crate::ui::components::{card, badge, button, BadgeStyle, ButtonStyle, Colors, Spacing, Typography};
use chrono::{Local, TimeZone};
use eframe::egui;

/// Transaction detail state
struct TransactionDetailState {
    current_txid: Option<String>,
    transaction: Option<bitvault_common::types::TransactionInfo>,
    is_loading: bool,
    error: Option<String>,
    is_cancelling: bool,
    cancel_error: Option<String>,
    cancel_success: bool,
    pin_verification: crate::ui::pin::PinVerificationState,
}

impl Default for TransactionDetailState {
    fn default() -> Self {
        Self {
            current_txid: None,
            transaction: None,
            is_loading: false,
            error: None,
            is_cancelling: false,
            cancel_error: None,
            cancel_success: false,
            pin_verification: crate::ui::pin::PinVerificationState::new(),
        }
    }
}

// Thread-local state for transaction detail
thread_local! {
    static TX_DETAIL_STATE: std::cell::RefCell<TransactionDetailState> =
        std::cell::RefCell::new(TransactionDetailState::default());
}

pub fn render(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    txid: &str,
) {
    let ctx = ui.ctx().clone();
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(Spacing::XL);
            
            ui.label(
                Typography::heading("Transaction Details")
                    .color(Colors::text_primary(&ctx))
            );

            if !app_state.is_vault_loaded() {
                card(ui, |ui| {
                    ui.label(
                        Typography::body("No vault loaded")
                            .color(Colors::text_secondary(&ctx))
                    );
                });
                ui.add_space(Spacing::MD);
                if button(ui, "Back", ButtonStyle::Secondary).clicked() {
                    navigation.go_back();
                }
                return;
            }

            ui.add_space(Spacing::LG);

        TX_DETAIL_STATE.with(|state| {
            let mut state = state.borrow_mut();

            // Reset state if txid changed
            if state
                .current_txid
                .as_ref()
                .map(|id| id != txid)
                .unwrap_or(true)
            {
                state.current_txid = Some(txid.to_string());
                state.transaction = None;
                state.error = None;
                state.is_loading = false;
                state.is_cancelling = false;
                state.cancel_error = None;
                state.cancel_success = false;
            }

            // Load transaction if not loaded
            if state.transaction.is_none() && !state.is_loading {
                load_transaction(ui, app_state, &mut state, txid);
            }

                // Show error if any
                if let Some(ref error) = state.error {
                    card(ui, |ui| {
                        ui.label(
                            Typography::body(error)
                                .color(Colors::ERROR)
                        );
                    });
                    ui.add_space(Spacing::MD);
                }

                // Show loading indicator
                if state.is_loading {
                    ui.vertical_centered(|ui| {
                        ui.spinner();
                        ui.add_space(Spacing::MD);
                        ui.label(
                            Typography::body("Loading transaction...")
                                .color(Colors::text_secondary(&ctx))
                        );
                    });
                    return;
                }

                // Show transaction details
                if let Some(ref tx) = state.transaction {
                    let tx_id = tx.tx_id.clone();
                    let is_pending = tx.status == bitvault_common::types::TransactionStatus::Pending;
                    let is_outgoing = tx.is_outgoing();

                    render_transaction_details(ui, app_state, navigation, tx, &ctx);

                    // Show cancel button for pending outgoing transactions
                    if is_pending && is_outgoing {
                        ui.add_space(Spacing::LG);
                        render_cancel_section(ui, app_state, &mut state, &tx_id, &ctx);
                    }
                } else {
                    card(ui, |ui| {
                        ui.label(
                            Typography::body("Transaction not found")
                                .color(Colors::text_secondary(&ctx))
                        );
                    });
                }

                ui.add_space(Spacing::LG);

                // Back button - centered
                if button(ui, "Back", ButtonStyle::Secondary).clicked() {
                    navigation.go_back();
                }
                
                ui.add_space(Spacing::XL);
            });
        });
    });
}

fn load_transaction(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut TransactionDetailState,
    txid: &str,
) {
    state.is_loading = true;
    state.error = None;

    // Get vault service and runtime
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let txid_clone = txid.to_string();
        let result = runtime.block_on(async {
            let vs = vault_service.read().await;
            vs.get_transaction(&txid_clone).await
        });

        match result {
            Ok(Some(tx)) => {
                state.transaction = Some(tx);
                state.is_loading = false;
            }
            Ok(None) => {
                state.error = Some("Transaction not found".to_string());
                state.is_loading = false;
            }
            Err(e) => {
                state.error = Some(format!("Failed to load transaction: {}", e));
                state.is_loading = false;
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.is_loading = false;
    }
}

fn render_transaction_details(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    _navigation: &mut Navigation,
    tx: &bitvault_common::types::TransactionInfo,
    ctx: &egui::Context,
) {
    let amount = tx.total_amount_btc();
    let is_positive = amount >= 0.0;
    
    // Main transaction card
    card(ui, |ui| {
        ui.vertical(|ui| {
            ui.add_space(Spacing::MD);
            
            // Header: Status badge and amount
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
                    ui.label(
                        Typography::heading(amount_str)
                            .color(if is_positive { Colors::SUCCESS } else { Colors::ERROR })
                    );
                });
            });
            
            ui.add_space(Spacing::MD);
            
            // Transaction ID section
            ui.label(
                Typography::body("Transaction ID")
                    .color(Colors::text_secondary(ctx))
            );
            ui.add_space(Spacing::XS);
            ui.horizontal(|ui| {
                ui.label(
                    Typography::body(&tx.tx_id)
                        .color(Colors::text_primary(ctx))
                        .monospace()
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if button(ui, "Copy", ButtonStyle::Text).clicked() {
                        ui.output_mut(|o| {
                            o.copied_text = tx.tx_id.clone();
                        });
                    }
                });
            });
            ui.add_space(Spacing::MD);
            
            // Address section
            ui.label(
                Typography::body(if tx.is_outgoing() { "To Address" } else { "From Address" })
                    .color(Colors::text_secondary(ctx))
            );
            ui.add_space(Spacing::XS);
            ui.horizontal(|ui| {
                ui.label(
                    Typography::body(&tx.address)
                        .color(Colors::text_primary(ctx))
                        .monospace()
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if button(ui, "Copy", ButtonStyle::Text).clicked() {
                        ui.output_mut(|o| {
                            o.copied_text = tx.address.clone();
                        });
                    }
                });
            });
            ui.add_space(Spacing::MD);
            
            // Details grid
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // Fee
                    if let Some(fee) = tx.fee_sat {
                        ui.label(
                            Typography::body("Fee")
                                .color(Colors::text_secondary(ctx))
                        );
                        ui.label(
                            Typography::body(format!("{} sats", fee))
                                .color(Colors::text_primary(ctx))
                        );
                        ui.add_space(Spacing::SM);
                    }
                    
                    // Date/Time
                    if tx.timestamp > 0 {
                        if let Some(dt) = Local.timestamp_opt(tx.timestamp, 0).single() {
                            ui.label(
                                Typography::body("Date")
                                    .color(Colors::text_secondary(ctx))
                            );
                            ui.label(
                                Typography::body(dt.format("%B %d, %Y at %H:%M:%S").to_string())
                                    .color(Colors::text_primary(ctx))
                            );
                            ui.add_space(Spacing::SM);
                        }
                    } else {
                        ui.label(
                            Typography::body("Date")
                                .color(Colors::text_secondary(ctx))
                        );
                        ui.label(
                            Typography::body("Pending")
                                .color(Colors::WARNING)
                        );
                        ui.add_space(Spacing::SM);
                    }
                });
                
                ui.add_space(Spacing::LG);
                
                ui.vertical(|ui| {
                    // Locktime
                    if tx.locktime > 0 {
                        ui.label(
                            Typography::body("Locktime")
                                .color(Colors::text_secondary(ctx))
                        );
                        ui.label(
                            Typography::body(format!("{} blocks", tx.locktime))
                                .color(Colors::text_primary(ctx))
                        );
                        ui.add_space(Spacing::SM);
                    }
                    
                    // Execution date (if different from timestamp and pending)
                    if tx.status == bitvault_common::types::TransactionStatus::Pending
                        && tx.execution_date > 0
                        && tx.execution_date != tx.timestamp
                    {
                        if let Some(dt) = Local.timestamp_opt(tx.execution_date, 0).single() {
                            ui.label(
                                Typography::body("Execution Date")
                                    .color(Colors::text_secondary(ctx))
                            );
                            ui.label(
                                Typography::body(dt.format("%B %d, %Y at %H:%M:%S").to_string())
                                    .color(Colors::text_primary(ctx))
                            );
                        }
                    }
                });
            });
            
            // Description
            if let Some(ref desc) = tx.description {
                if !desc.is_empty() {
                    ui.add_space(Spacing::MD);
                    ui.label(
                        Typography::body("Description")
                            .color(Colors::text_secondary(ctx))
                    );
                    ui.add_space(Spacing::XS);
                    ui.label(
                        Typography::body(desc)
                            .color(Colors::text_primary(ctx))
                    );
                }
            }
            
            ui.add_space(Spacing::MD);
        });
    });
}

fn render_cancel_section(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut TransactionDetailState,
    tx_id: &str,
    ctx: &egui::Context,
) {
    card(ui, |ui| {
        ui.vertical(|ui| {
            ui.add_space(Spacing::MD);
            
            ui.label(
                Typography::heading_small("Transaction Actions")
                    .color(Colors::text_primary(ctx))
            );
            ui.add_space(Spacing::MD);

            // Show cancel success message
            if state.cancel_success {
                ui.vertical_centered(|ui| {
                    badge(ui, "✓ Cancellation sent successfully!", BadgeStyle::Success);
                    ui.add_space(Spacing::SM);
                    ui.label(
                        Typography::body("The replacement transaction has been broadcast. The original transaction will be replaced once confirmed.")
                            .color(Colors::text_secondary(ctx))
                    );
                });
                ui.add_space(Spacing::MD);
            }

            // Show cancel error
            if let Some(ref error) = state.cancel_error {
                ui.label(
                    Typography::body(format!("Cancel failed: {}", error))
                        .color(Colors::ERROR)
                );
                ui.add_space(Spacing::MD);
            }

            // Action buttons
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let button_text = if state.is_cancelling {
                        "Cancelling..."
                    } else {
                        "Cancel Transaction (RBF)"
                    };

                    let button_enabled = !state.is_cancelling && !state.cancel_success;

                    if button_enabled {
                        if button(ui, button_text, ButtonStyle::Danger).clicked() {
                            cancel_transaction(ui, app_state, state, tx_id);
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new(button_text));
                    }

                    ui.add_space(Spacing::MD);
                    if button(ui, "Refresh Status", ButtonStyle::Secondary).clicked() {
                        // Reload transaction to get updated status
                        state.transaction = None;
                        state.is_loading = false;
                    }
                });
            });

            ui.add_space(Spacing::MD);
            ui.label(
                Typography::caption("⚠ Canceling will create a replacement transaction with a higher fee. The original transaction will be replaced once the replacement is confirmed.")
                    .color(Colors::WARNING)
            );
            
            ui.add_space(Spacing::MD);
        });
    });
}

fn cancel_transaction(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut TransactionDetailState,
    tx_id: &str,
) {
    // Check PIN before canceling (if PIN is set)
    let pin_service = bitvault_common::PinService::new();
    if pin_service.has_pin() && !state.pin_verification.is_verified() {
        // Show PIN verification modal
        if !state.pin_verification.is_visible() {
            state.pin_verification.show();
        }
        if crate::ui::pin::render_pin_verification(ui.ctx(), &mut state.pin_verification) {
            // PIN verified, continue with cancellation
        } else {
            return; // Wait for PIN verification
        }
    }

    state.is_cancelling = true;
    state.cancel_error = None;
    state.cancel_success = false;

    // Get vault service and runtime
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let tx_id_clone = tx_id.to_string();
        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.sign_and_send_cancel_transaction(&tx_id_clone).await
        });

        match result {
            Ok(_response) => {
                state.cancel_success = true;
                state.is_cancelling = false;
                // Reload transaction to get updated status
                state.transaction = None;
                state.is_loading = false;
                // Reset PIN verification after successful cancellation
                state.pin_verification.reset();
            }
            Err(e) => {
                state.cancel_error = Some(format!("Failed to cancel transaction: {}", e));
                state.is_cancelling = false;
                // Reset PIN verification on error
                state.pin_verification.reset();
            }
        }
    } else {
        state.cancel_error = Some("Vault not loaded or runtime not available".to_string());
        state.is_cancelling = false;
    }
}
