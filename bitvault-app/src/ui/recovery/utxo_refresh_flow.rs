//! UTXO Refresh Flow
//!
//! Flow for refreshing UTXOs older than 6 months

use super::utxo_selection::{render_utxo_selection, RecoveryMode, UtxoSelectionState};
use crate::state::{AppState, Navigation};
use eframe::egui;

/// UTXO refresh workflow steps
#[derive(Debug, Clone, PartialEq)]
enum UtxoRefreshStep {
    LoadingUtxos,
    SelectingUtxos,
    Signing,
    Sharing,
    Error,
}

/// UTXO refresh flow state
struct UtxoRefreshState {
    current_step: UtxoRefreshStep,
    step_history: Vec<UtxoRefreshStep>,
    // Step-specific data
    selection_state: Option<UtxoSelectionState>,
    psbt_base64: String,
    compressed_psbt: String,
    error: Option<String>,
}

impl Default for UtxoRefreshState {
    fn default() -> Self {
        Self {
            current_step: UtxoRefreshStep::LoadingUtxos,
            step_history: Vec::new(),
            selection_state: None,
            psbt_base64: String::new(),
            compressed_psbt: String::new(),
            error: None,
        }
    }
}

impl UtxoRefreshState {
    /// Advance to the next step in the workflow
    fn advance_to_step(&mut self, step: UtxoRefreshStep) {
        if step != self.current_step {
            self.step_history.push(self.current_step.clone());
            self.current_step = step;
        }
    }

    /// Go back to the previous step in the workflow
    /// Returns true if there was a previous step, false if at first step
    fn go_to_previous_step(&mut self) -> bool {
        if let Some(previous) = self.step_history.pop() {
            self.current_step = previous;
            true
        } else {
            false  // At first step
        }
    }

    /// Check if we can go back in the workflow
    fn can_go_back_in_workflow(&self) -> bool {
        !self.step_history.is_empty()
    }
}

// Thread-local state for UTXO refresh flow
thread_local! {
    static REFRESH_STATE: std::cell::RefCell<UtxoRefreshState> =
        std::cell::RefCell::new(UtxoRefreshState::default());
}

/// Check if we can go back in the UTXO refresh workflow
pub fn can_go_back_in_utxo_refresh_workflow() -> bool {
    REFRESH_STATE.with(|state| {
        state.borrow().can_go_back_in_workflow()
    })
}

/// Go back in the UTXO refresh workflow
/// Returns true if there was a previous step, false if at first step
pub fn go_back_in_utxo_refresh_workflow() -> bool {
    REFRESH_STATE.with(|state| {
        state.borrow_mut().go_to_previous_step()
    })
}

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical_centered(|ui| {
        ui.heading("UTXO Refresh");
        ui.add_space(20.0);

        if !app_state.is_vault_loaded() {
            ui.label("No vault loaded");
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button("Back").clicked() {
                    navigation.go_back();
                }
            });
            return;
        }

        REFRESH_STATE.with(|state_ref| {
            let mut state = state_ref.borrow_mut();

            match state.current_step {
                UtxoRefreshStep::LoadingUtxos => {
                    load_old_utxos(ui, app_state, &mut state, true);
                }
                UtxoRefreshStep::SelectingUtxos => {
                    let mut cancel_clicked = false;
                    let mut continue_clicked = false;
                    let mut has_selection = false;
                    
                    if let Some(ref mut selection_state) = state.selection_state {
                        has_selection = selection_state.has_selection();
                        render_utxo_selection(ui, selection_state, RecoveryMode::Refresh);
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            cancel_clicked = ui.button("Cancel").clicked();
                            continue_clicked = ui.button("Continue").clicked() && has_selection;
                        });
                    }
                    
                    // Handle navigation outside the borrow
                    if cancel_clicked {
                        // Use step-based navigation for consistency
                        if !state.go_to_previous_step() {
                            // At first step, exit workflow
                            navigation.go_back();
                        }
                    }
                    if continue_clicked {
                        build_refresh_transaction(ui, app_state, &mut state);
                    }
                }
                UtxoRefreshStep::Signing => {
                    sign_refresh_transaction(ui, app_state, &mut state);
                }
                UtxoRefreshStep::Sharing => {
                    render_sharing(ui, app_state, navigation, &state.compressed_psbt);
                }
                UtxoRefreshStep::Error => {
                    if let Some(ref error) = state.error {
                        ui.colored_label(egui::Color32::RED, error);
                        ui.add_space(10.0);
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            if ui.button("Back").clicked() {
                                state.go_to_previous_step();
                            }
                        });
                    }
                }
            }
        });
    });
}

fn load_old_utxos(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut UtxoRefreshState,
    is_refresh: bool,
) {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.get_old_utxos(is_refresh).await
        });

        match result {
            Ok(utxos) => {
                let mut selection_state = UtxoSelectionState::default();
                selection_state.utxos = utxos;
                state.selection_state = Some(selection_state);
                state.advance_to_step(UtxoRefreshStep::SelectingUtxos);
            }
            Err(e) => {
                state.error = Some(format!("Failed to load UTXOs: {}", e));
                state.advance_to_step(UtxoRefreshStep::Error);
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.advance_to_step(UtxoRefreshStep::Error);
    }
}

fn build_refresh_transaction(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut UtxoRefreshState,
) {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        // Get vault address (destination for refresh - send to self)
        let vault_address = match app_state.vault_data.lock() {
            Ok(data) => data.receive_address.clone().unwrap_or_default(),
            Err(_) => {
                state.error = Some("Failed to get vault address".to_string());
                state.advance_to_step(UtxoRefreshStep::Error);
                return;
            }
        };

        if vault_address.is_empty() {
            state.error = Some("Vault address not available".to_string());
            state.advance_to_step(UtxoRefreshStep::Error);
            return;
        }

        // Get selected UTXOs from selection state
        let selected_utxos: Vec<String> = if let Some(ref selection_state) = state.selection_state {
            selection_state.selected.iter().cloned().collect()
        } else {
            state.error = Some("No UTXOs selected".to_string());
            state.advance_to_step(UtxoRefreshStep::Error);
            return;
        };

        // Convert selected outpoints from strings to OutPoint
        use bdk::bitcoin::{OutPoint, Txid};
        use std::str::FromStr;
        
        let utxos_to_spend: Result<Vec<OutPoint>, _> = selected_utxos
            .iter()
            .map(|outpoint_str| {
                // Format: "txid:vout"
                let parts: Vec<&str> = outpoint_str.split(':').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid outpoint format: {}", outpoint_str));
                }
                let txid = Txid::from_str(parts[0]).map_err(|e| format!("Invalid txid: {}", e))?;
                let vout: u32 = parts[1]
                    .parse()
                    .map_err(|e| format!("Invalid vout: {}", e))?;
                Ok(OutPoint { txid, vout })
            })
            .collect();

        let _utxos_to_spend = match utxos_to_spend {
            Ok(utxos) => utxos,
            Err(e) => {
                state.error = Some(e);
                state.advance_to_step(UtxoRefreshStep::Error);
                return;
            }
        };

        // Convert OutPoints to strings for the API
        let utxo_strings: Vec<String> = selected_utxos;

        // Build transaction preview (refresh = send max to self, with selected UTXOs)
        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.build_transaction_preview(
                &vault_address,
                0.0,                 // Amount (ignored when is_sending_max = true)
                3,                   // Fee rate (sat/vB) - low fee for refresh
                None,                // Description
                true,                // is_sending_max
                false,               // is_recovery (refresh is not recovery)
                Some(&utxo_strings), // Selected UTXOs as strings
            )
            .await
        });

        match result {
            Ok(preview) => {
                // Sign and share the refresh transaction
                state.psbt_base64 = preview.psbt.clone();
                state.advance_to_step(UtxoRefreshStep::Signing);
            }
            Err(e) => {
                state.error = Some(format!("Failed to build refresh transaction: {}", e));
                state.advance_to_step(UtxoRefreshStep::Error);
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.advance_to_step(UtxoRefreshStep::Error);
    }
}

fn sign_refresh_transaction(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut UtxoRefreshState,
) {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        // Get vault address (recipient for refresh)
        let vault_address = match app_state.vault_data.lock() {
            Ok(data) => data.receive_address.clone().unwrap_or_default(),
            Err(_) => {
                state.error = Some("Failed to get vault address".to_string());
                state.advance_to_step(UtxoRefreshStep::Error);
                return;
            }
        };

        if vault_address.is_empty() {
            state.error = Some("Vault address not available".to_string());
            state.advance_to_step(UtxoRefreshStep::Error);
            return;
        }

        let psbt_base64 = state.psbt_base64.clone();

        // Sign and compress PSBT for sharing
        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.sign_and_share_recovery_tx(&psbt_base64, &vault_address)
                .await
        });

        match result {
            Ok(compressed_psbt) => {
                state.compressed_psbt = compressed_psbt;
                state.advance_to_step(UtxoRefreshStep::Sharing);
            }
            Err(e) => {
                state.error = Some(format!("Failed to sign refresh transaction: {}", e));
                state.advance_to_step(UtxoRefreshStep::Error);
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.advance_to_step(UtxoRefreshStep::Error);
    }
}

fn render_sharing(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    navigation: &mut Navigation,
    compressed_psbt: &str,
) {
    ui.heading("Share Refresh Transaction");
    ui.add_space(10.0);
    ui.label("Scan this QR code with the second device to complete the refresh:");
    ui.add_space(10.0);

    // Generate and display QR code from compressed_psbt
    use crate::utils::qr::generate_qr_image;

    if let Some(qr_texture) = generate_qr_image(ui.ctx(), compressed_psbt) {
        ui.image((qr_texture.id(), egui::Vec2::new(300.0, 300.0)));
        ui.add_space(10.0);
    } else {
        ui.colored_label(egui::Color32::YELLOW, "Failed to generate QR code");
        ui.label(format!(
            "PSBT: {}...",
            &compressed_psbt[..compressed_psbt.len().min(50)]
        ));
    }

    ui.add_space(20.0);
    // Buttons - centered
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        if ui.button("Copy PSBT").clicked() {
            ui.output_mut(|o| {
                o.copied_text = compressed_psbt.to_string();
            });
        }
        ui.add_space(10.0);
        if ui.button("Done").clicked() {
            navigation.go_back();
        }
    });
}
