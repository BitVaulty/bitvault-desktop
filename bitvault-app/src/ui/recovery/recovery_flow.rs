//! Recovery Transaction Flow
//!
//! Flow for recovering UTXOs older than 1 year

use super::utxo_selection::{render_utxo_selection, RecoveryMode, UtxoSelectionState};
use crate::state::{AppState, Navigation};
use eframe::egui;

/// Recovery flow state
#[derive(Default)]
enum RecoveryState {
    #[default]
    LoadingUtxos,
    SelectingUtxos(UtxoSelectionState),
    BuildingPreview {
        selected_utxos: Vec<String>, // OutPoint as string
    },
    PreviewReady {
        preview: bitvault_common::types::TransactionPreview,
        psbt_base64: String,
    },
    Signing {
        psbt_base64: String,
        recipient: String,
    },
    Sharing {
        compressed_psbt: String,
    },
    Error(String),
}

// Thread-local state for recovery flow
thread_local! {
    static RECOVERY_STATE: std::cell::RefCell<RecoveryState> =
        std::cell::RefCell::new(RecoveryState::default());
}

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical_centered(|ui| {
        ui.heading("Recovery Transaction");
        ui.add_space(20.0);

        if !app_state.is_vault_loaded() {
            ui.label("No vault loaded");
            if ui.button("Back").clicked() {
                navigation.go_back();
            }
            return;
        }

        RECOVERY_STATE.with(|state| {
            let mut state = state.borrow_mut();

            // Use a flag to defer state mutations
            let mut next_state: Option<RecoveryState> = None;

            match &mut *state {
                RecoveryState::LoadingUtxos => {
                    // Defer the load to avoid borrow conflict
                    next_state = Some(RecoveryState::LoadingUtxos);
                }
                RecoveryState::SelectingUtxos(selection_state) => {
                    render_utxo_selection(ui, selection_state, RecoveryMode::Recovery);
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            navigation.go_back();
                        }
                        if ui.button("Continue").clicked() && selection_state.has_selection() {
                            let selected: Vec<String> =
                                selection_state.selected.iter().cloned().collect();
                            next_state = Some(RecoveryState::BuildingPreview {
                                selected_utxos: selected,
                            });
                        }
                    });
                }
                RecoveryState::BuildingPreview { selected_utxos } => {
                    ui.label("Building transaction preview...");
                    let selected = selected_utxos.clone();
                    // Will handle after match
                    next_state = Some(RecoveryState::BuildingPreview {
                        selected_utxos: selected,
                    });
                }
                RecoveryState::PreviewReady {
                    preview,
                    psbt_base64,
                } => {
                    render_preview(ui, preview);
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            navigation.go_back();
                        }
                        if ui.button("Sign & Share").clicked() {
                            // Get recipient address (for now, use vault address)
                            match app_state.vault_data.lock() {
                                Ok(data) => {
                                    let recipient =
                                        data.receive_address.clone().unwrap_or_default();
                                    next_state = Some(RecoveryState::Signing {
                                        psbt_base64: psbt_base64.clone(),
                                        recipient: recipient.clone(),
                                    });
                                }
                                Err(_) => {
                                    next_state = Some(RecoveryState::Error(
                                        "Error: Mutex poisoned".to_string(),
                                    ));
                                }
                            }
                        }
                    });
                }
                RecoveryState::Signing {
                    psbt_base64,
                    recipient,
                } => {
                    ui.label("Signing transaction...");
                    let psbt = psbt_base64.clone();
                    let recip = recipient.clone();
                    // Will handle after match
                    next_state = Some(RecoveryState::Signing {
                        psbt_base64: psbt,
                        recipient: recip,
                    });
                }
                RecoveryState::Sharing { compressed_psbt } => {
                    render_sharing(ui, app_state, navigation, compressed_psbt);
                }
                RecoveryState::Error(error) => {
                    ui.colored_label(egui::Color32::RED, error);
                    ui.add_space(10.0);
                    if ui.button("Back").clicked() {
                        navigation.go_back();
                    }
                }
            }

            // Handle deferred state changes and async operations
            if let Some(new_state) = next_state {
                match new_state {
                    RecoveryState::LoadingUtxos => {
                        load_old_utxos(ui, app_state, &mut state, false);
                    }
                    RecoveryState::BuildingPreview { selected_utxos } => {
                        *state = RecoveryState::BuildingPreview {
                            selected_utxos: selected_utxos.clone(),
                        };
                        build_recovery_preview(ui, app_state, &mut state, selected_utxos);
                    }
                    RecoveryState::Signing {
                        psbt_base64,
                        recipient,
                    } => {
                        *state = RecoveryState::Signing {
                            psbt_base64: psbt_base64.clone(),
                            recipient: recipient.clone(),
                        };
                        sign_and_share(ui, app_state, &mut state, psbt_base64, recipient);
                    }
                    _ => {
                        *state = new_state;
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
                *state = RecoveryState::SelectingUtxos(selection_state);
            }
            Err(e) => {
                *state = RecoveryState::Error(format!("Failed to load UTXOs: {}", e));
            }
        }
    } else {
        *state = RecoveryState::Error("Vault not loaded or runtime not available".to_string());
    }
}

fn build_recovery_preview(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut RecoveryState,
    selected_utxos: Vec<String>,
) {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        // Get recipient address (use vault address for recovery)
        let recipient = match app_state.vault_data.lock() {
            Ok(data) => data.receive_address.clone().unwrap_or_default(),
            Err(_) => {
                *state = RecoveryState::Error("Error: Mutex poisoned".to_string());
                return;
            }
        };

        if recipient.is_empty() {
            *state = RecoveryState::Error("No recipient address available".to_string());
            return;
        }

        // Convert OutPoint strings to the format expected by build_transaction_preview
        // We need to keep the vector alive for the reference
        let utxos_to_spend: Option<&[String]> = if selected_utxos.is_empty() {
            None
        } else {
            Some(&selected_utxos)
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
                // Extract PSBT from preview (it should be in the preview)
                let psbt_base64 = preview.psbt.clone();
                *state = RecoveryState::PreviewReady {
                    preview,
                    psbt_base64,
                };
            }
            Err(e) => {
                *state = RecoveryState::Error(format!("Failed to build preview: {}", e));
            }
        }
    } else {
        *state = RecoveryState::Error("Vault not loaded or runtime not available".to_string());
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

fn sign_and_share(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut RecoveryState,
    psbt_base64: String,
    recipient: String,
) {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.sign_and_share_recovery_tx(&psbt_base64, &recipient)
                .await
        });

        match result {
            Ok(compressed_psbt) => {
                *state = RecoveryState::Sharing { compressed_psbt };
            }
            Err(e) => {
                *state = RecoveryState::Error(format!("Failed to sign and share: {}", e));
            }
        }
    } else {
        *state = RecoveryState::Error("Vault not loaded or runtime not available".to_string());
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
