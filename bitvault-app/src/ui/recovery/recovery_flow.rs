//! Recovery Transaction Flow
//!
//! Flow for recovering UTXOs older than 1 year

use super::utxo_selection::{render_utxo_selection, RecoveryMode, UtxoSelectionState};
use crate::state::{AppState, Navigation};
use eframe::egui;

/// Recovery workflow steps
#[derive(Debug, Clone, PartialEq)]
enum RecoveryStep {
    LoadingUtxos,
    SelectingUtxos,
    BuildingPreview,
    PreviewReady,
    Signing,
    Sharing,
    Error,
}

/// Recovery flow state
struct RecoveryState {
    current_step: RecoveryStep,
    step_history: Vec<RecoveryStep>,
    // Step-specific data
    selection_state: Option<UtxoSelectionState>,
    selected_utxos: Vec<String>,
    preview: Option<bitvault_common::types::TransactionPreview>,
    psbt_base64: String,
    recipient: String,
    compressed_psbt: String,
    error: Option<String>,
}

impl Default for RecoveryState {
    fn default() -> Self {
        Self {
            current_step: RecoveryStep::LoadingUtxos,
            step_history: Vec::new(),
            selection_state: None,
            selected_utxos: Vec::new(),
            preview: None,
            psbt_base64: String::new(),
            recipient: String::new(),
            compressed_psbt: String::new(),
            error: None,
        }
    }
}

impl RecoveryState {
    /// Advance to the next step in the workflow
    fn advance_to_step(&mut self, step: RecoveryStep) {
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
            false // At first step
        }
    }

    /// Check if we can go back in the workflow
    fn can_go_back_in_workflow(&self) -> bool {
        !self.step_history.is_empty()
    }
}

// Thread-local state for recovery flow
thread_local! {
    static RECOVERY_STATE: std::cell::RefCell<RecoveryState> =
        std::cell::RefCell::new(RecoveryState::default());
}

/// Check if we can go back in the recovery workflow
pub fn can_go_back_in_recovery_workflow() -> bool {
    RECOVERY_STATE.with(|state| state.borrow().can_go_back_in_workflow())
}

/// Go back in the recovery workflow
/// Returns true if there was a previous step, false if at first step
pub fn go_back_in_recovery_workflow() -> bool {
    RECOVERY_STATE.with(|state| state.borrow_mut().go_to_previous_step())
}

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical_centered(|ui| {
        ui.heading("Recovery Transaction");
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

        RECOVERY_STATE.with(|state| {
            let mut state = state.borrow_mut();

            match state.current_step {
                RecoveryStep::LoadingUtxos => {
                    load_old_utxos(ui, app_state, &mut state, false);
                }
                RecoveryStep::SelectingUtxos => {
                    let mut cancel_clicked = false;
                    let mut continue_clicked = false;
                    let mut _has_selection = false;

                    if let Some(ref mut selection_state) = state.selection_state {
                        render_utxo_selection(ui, selection_state, RecoveryMode::Recovery);
                        ui.add_space(20.0);
                        // Buttons - centered
                        let button_width = 120.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
                            egui::Sense::click(),
                        );
                        let mut button_ui =
                            ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                        cancel_clicked = button_ui.button("Cancel").clicked();
                        button_ui.add_space(10.0);
                        _has_selection = selection_state.has_selection();
                        continue_clicked = button_ui.button("Continue").clicked() && _has_selection;
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
                        if let Some(ref selection_state) = state.selection_state {
                            state.selected_utxos =
                                selection_state.selected.iter().cloned().collect();
                            state.advance_to_step(RecoveryStep::BuildingPreview);
                        }
                    }
                }
                RecoveryStep::BuildingPreview => {
                    ui.label("Building transaction preview...");
                    build_recovery_preview(ui, app_state, &mut state);
                }
                RecoveryStep::PreviewReady => {
                    if let Some(ref preview) = state.preview {
                        render_preview(ui, preview);
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                state.go_to_previous_step();
                            }
                            if ui.button("Sign & Share").clicked() {
                                // Get recipient address (for now, use vault address)
                                match app_state.vault_data.lock() {
                                    Ok(data) => {
                                        state.recipient =
                                            data.receive_address.clone().unwrap_or_default();
                                        state.advance_to_step(RecoveryStep::Signing);
                                    }
                                    Err(_) => {
                                        state.error = Some("Error: Mutex poisoned".to_string());
                                        state.advance_to_step(RecoveryStep::Error);
                                    }
                                }
                            }
                        });
                    }
                }
                RecoveryStep::Signing => {
                    ui.label("Signing transaction...");
                    sign_and_share(ui, app_state, &mut state);
                }
                RecoveryStep::Sharing => {
                    render_sharing(ui, app_state, navigation, &state.compressed_psbt);
                }
                RecoveryStep::Error => {
                    if let Some(ref error) = state.error {
                        ui.colored_label(egui::Color32::RED, error);
                        ui.add_space(10.0);
                        if ui.button("Back").clicked() {
                            state.go_to_previous_step();
                        }
                    }
                }
            }
        });
    });
}

fn load_old_utxos(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut RecoveryState,
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
                state.advance_to_step(RecoveryStep::SelectingUtxos);
            }
            Err(e) => {
                state.error = Some(format!("Failed to load UTXOs: {}", e));
                state.advance_to_step(RecoveryStep::Error);
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.advance_to_step(RecoveryStep::Error);
    }
}

fn build_recovery_preview(_ui: &mut egui::Ui, app_state: &mut AppState, state: &mut RecoveryState) {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        // Get recipient address (use vault address for recovery)
        let recipient = match app_state.vault_data.lock() {
            Ok(data) => data.receive_address.clone().unwrap_or_default(),
            Err(_) => {
                state.error = Some("Error: Mutex poisoned".to_string());
                state.advance_to_step(RecoveryStep::Error);
                return;
            }
        };

        if recipient.is_empty() {
            state.error = Some("No recipient address available".to_string());
            state.advance_to_step(RecoveryStep::Error);
            return;
        }

        // Convert OutPoint strings to the format expected by build_transaction_preview
        let utxos_to_spend: Option<&[String]> = if state.selected_utxos.is_empty() {
            None
        } else {
            Some(&state.selected_utxos)
        };

        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.build_transaction_preview(
                &recipient,
                0.0,  // Amount will be determined by selected UTXOs
                1,    // Fee rate
                None, // No description
                true, // Send max (drain selected UTXOs)
                true, // is_recovery
                utxos_to_spend,
            )
            .await
        });

        match result {
            Ok(preview) => {
                state.preview = Some(preview.clone());
                state.psbt_base64 = preview.psbt.clone();
                state.recipient = recipient;
                state.advance_to_step(RecoveryStep::PreviewReady);
            }
            Err(e) => {
                state.error = Some(format!("Failed to build preview: {}", e));
                state.advance_to_step(RecoveryStep::Error);
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.advance_to_step(RecoveryStep::Error);
    }
}

fn render_preview(ui: &mut egui::Ui, preview: &bitvault_common::types::TransactionPreview) {
    ui.separator();
    ui.add_space(10.0);
    ui.heading("Transaction Preview");
    ui.label(format!("Amount: {:.8} BTC", preview.amount));
    ui.label(format!("Fee: {} sats", preview.fee));
    ui.label(format!("Recipient: {}", preview.recipient));
    if let Some(ref desc) = preview.description {
        ui.label(format!("Description: {}", desc));
    }
}

fn sign_and_share(_ui: &mut egui::Ui, app_state: &mut AppState, state: &mut RecoveryState) {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let psbt_base64 = state.psbt_base64.clone();
        let recipient = state.recipient.clone();

        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.sign_and_share_recovery_tx(&psbt_base64, &recipient)
                .await
        });

        match result {
            Ok(compressed_psbt) => {
                state.compressed_psbt = compressed_psbt;
                state.advance_to_step(RecoveryStep::Sharing);
            }
            Err(e) => {
                state.error = Some(format!("Failed to sign and share: {}", e));
                state.advance_to_step(RecoveryStep::Error);
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.advance_to_step(RecoveryStep::Error);
    }
}

fn render_sharing(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    navigation: &mut Navigation,
    compressed_psbt: &str,
) {
    ui.heading("Share Recovery Transaction");
    ui.add_space(10.0);
    ui.label("Scan this QR code with the second device to complete the recovery:");
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
    if ui.button("Copy PSBT").clicked() {
        ui.output_mut(|o| {
            o.copied_text = compressed_psbt.to_string();
        });
    }

    ui.add_space(20.0);
    if ui.button("Done").clicked() {
        navigation.go_back();
    }
}
