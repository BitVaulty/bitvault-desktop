//! Transaction Detail View
//!
//! Displays detailed information about a specific transaction

use eframe::egui;
use crate::state::{AppState, Navigation};
use chrono::{Local, TimeZone};

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

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation, txid: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.heading("Transaction Details");

        if !app_state.is_vault_loaded() {
            ui.label("No vault loaded");
            if ui.button("Back").clicked() {
                navigation.go_back();
            }
            return;
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        TX_DETAIL_STATE.with(|state| {
            let mut state = state.borrow_mut();

            // Reset state if txid changed
            if state.current_txid.as_ref().map(|id| id != txid).unwrap_or(true) {
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
                ui.colored_label(egui::Color32::RED, error);
                ui.add_space(10.0);
            }

            // Show loading indicator
            if state.is_loading {
                ui.label("Loading transaction...");
                return;
            }

            // Show transaction details
            if let Some(ref tx) = state.transaction {
                let tx_id = tx.tx_id.clone();
                let is_pending = tx.status == bitvault_common::types::TransactionStatus::Pending;
                let is_outgoing = tx.is_outgoing();
                
                render_transaction_details(ui, app_state, navigation, tx);
                
                // Show cancel button for pending outgoing transactions
                if is_pending && is_outgoing {
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);
                    render_cancel_section(ui, app_state, &mut state, &tx_id);
                }
            } else {
                ui.label("Transaction not found");
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Back button
            if ui.button("← Back").clicked() {
                navigation.go_back();
            }
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
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref()) {
        
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
) {
    // Transaction ID
    ui.horizontal(|ui| {
        ui.label("Transaction ID:");
        ui.label(&tx.tx_id);
        if ui.button("Copy").clicked() {
            ui.output_mut(|o| {
                o.copied_text = tx.tx_id.clone();
            });
        }
    });
    ui.add_space(10.0);

    // Status
    let status_color = match tx.status {
        bitvault_common::types::TransactionStatus::Pending => egui::Color32::YELLOW,
        bitvault_common::types::TransactionStatus::Sent => egui::Color32::RED,
        bitvault_common::types::TransactionStatus::Received => egui::Color32::GREEN,
    };
    ui.horizontal(|ui| {
        ui.label("Status:");
        ui.colored_label(status_color, tx.status.as_str());
    });
    ui.add_space(10.0);

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
    ui.horizontal(|ui| {
        ui.label(if tx.is_outgoing() { "Sent:" } else { "Received:" });
        ui.colored_label(amount_color, &amount_str);
    });
    ui.add_space(10.0);

    // Address
    ui.horizontal(|ui| {
        ui.label(if tx.is_outgoing() { "To:" } else { "From:" });
        ui.label(&tx.address);
        if ui.button("Copy").clicked() {
            ui.output_mut(|o| {
                o.copied_text = tx.address.clone();
            });
        }
    });
    ui.add_space(10.0);

    // Fee
    if let Some(fee) = tx.fee_sat {
        ui.horizontal(|ui| {
            ui.label("Fee:");
            ui.label(format!("{} sats", fee));
        });
        ui.add_space(10.0);
    }

    // Date/Time
    if tx.timestamp > 0 {
        if let Some(dt) = Local.timestamp_opt(tx.timestamp, 0).single() {
            ui.horizontal(|ui| {
                ui.label("Date:");
                ui.label(dt.format("%Y-%m-%d %H:%M:%S").to_string());
            });
            ui.add_space(10.0);
        }
    } else {
        ui.label("Date: Pending");
        ui.add_space(10.0);
    }

    // Description
    if let Some(ref desc) = tx.description {
        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.label(desc);
        });
        ui.add_space(10.0);
    }

    // Locktime
    if tx.locktime > 0 {
        ui.horizontal(|ui| {
            ui.label("Locktime:");
            ui.label(format!("{} blocks", tx.locktime));
        });
        ui.add_space(10.0);
    }

    // Execution date (if different from timestamp and pending)
    if tx.status == bitvault_common::types::TransactionStatus::Pending 
        && tx.execution_date > 0 
        && tx.execution_date != tx.timestamp {
        if let Some(dt) = Local.timestamp_opt(tx.execution_date, 0).single() {
            ui.horizontal(|ui| {
                ui.label("Execution Date:");
                ui.label(dt.format("%Y-%m-%d %H:%M:%S").to_string());
            });
        }
    }
}

fn render_cancel_section(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut TransactionDetailState,
    tx_id: &str,
) {
    ui.heading("Transaction Actions");
    ui.add_space(10.0);

    // Show cancel success message
    if state.cancel_success {
        ui.colored_label(egui::Color32::GREEN, "✓ Transaction cancellation sent successfully!");
        ui.label("The replacement transaction has been broadcast. The original transaction will be replaced once confirmed.");
        ui.add_space(10.0);
    }

    // Show cancel error
    if let Some(ref error) = state.cancel_error {
        ui.colored_label(egui::Color32::RED, format!("Cancel failed: {}", error));
        ui.add_space(10.0);
    }

    // Cancel button
    ui.horizontal(|ui| {
        let button_text = if state.is_cancelling {
            "Cancelling..."
        } else {
            "Cancel Transaction (RBF)"
        };

        let button_enabled = !state.is_cancelling && !state.cancel_success;

        if ui.add_enabled(button_enabled, egui::Button::new(button_text)).clicked() {
            cancel_transaction(ui, app_state, state, tx_id);
        }

        if ui.button("Refresh Status").clicked() {
            // Reload transaction to get updated status
            state.transaction = None;
            state.is_loading = false;
        }
    });

    ui.add_space(10.0);
    ui.label("⚠️ Canceling will create a replacement transaction with a higher fee. The original transaction will be replaced once the replacement is confirmed.");
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
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref()) {
        
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

