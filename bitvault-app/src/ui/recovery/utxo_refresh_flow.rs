//! UTXO Refresh Flow
//!
//! Flow for refreshing UTXOs older than 6 months

use eframe::egui;
use crate::state::{AppState, Navigation};
use super::utxo_selection::{UtxoSelectionState, render_utxo_selection, RecoveryMode};

/// UTXO refresh flow state
#[derive(Default)]
enum UtxoRefreshState {
    #[default]
    LoadingUtxos,
    SelectingUtxos(UtxoSelectionState),
    Signing {
        psbt_base64: String,
    },
    Sharing {
        compressed_psbt: String,
    },
    Error(String),
}


// Thread-local state for UTXO refresh flow
thread_local! {
    static REFRESH_STATE: std::cell::RefCell<UtxoRefreshState> = 
        std::cell::RefCell::new(UtxoRefreshState::default());
}

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical_centered(|ui| {
        ui.heading("UTXO Refresh");
        ui.add_space(20.0);

        if !app_state.is_vault_loaded() {
            ui.label("No vault loaded");
            if ui.button("Back").clicked() {
                navigation.go_back();
            }
            return;
        }

        REFRESH_STATE.with(|state| {
            let mut state = state.borrow_mut();

            match &mut *state {
                UtxoRefreshState::LoadingUtxos => {
                    load_old_utxos(ui, app_state, &mut state, true);
                }
                UtxoRefreshState::SelectingUtxos(selection_state) => {
                    render_utxo_selection(ui, selection_state, RecoveryMode::Refresh);
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            navigation.go_back();
                        }
                        if ui.button("Continue").clicked() && selection_state.has_selection() {
                            // For refresh, we need to build preview and sign
                            // This is similar to recovery but simpler
                            ui.label("Building transaction...");
                            // TODO: Implement refresh transaction building
                        }
                    });
                }
                UtxoRefreshState::Signing { psbt_base64: _ } => {
                    ui.label("Signing refresh transaction...");
                    // TODO: Implement signing
                }
                UtxoRefreshState::Sharing { compressed_psbt } => {
                    render_sharing(ui, app_state, navigation, compressed_psbt);
                }
                UtxoRefreshState::Error(error) => {
                    ui.colored_label(egui::Color32::RED, error);
                    ui.add_space(10.0);
                    if ui.button("Back").clicked() {
                        navigation.go_back();
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
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref()) {
        
        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.get_old_utxos(is_refresh).await
        });

        match result {
            Ok(utxos) => {
                let mut selection_state = UtxoSelectionState::default();
                selection_state.utxos = utxos;
                *state = UtxoRefreshState::SelectingUtxos(selection_state);
            }
            Err(e) => {
                *state = UtxoRefreshState::Error(format!("Failed to load UTXOs: {}", e));
            }
        }
    } else {
        *state = UtxoRefreshState::Error("Vault not loaded or runtime not available".to_string());
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
        ui.label(format!("PSBT: {}...", &compressed_psbt[..compressed_psbt.len().min(50)]));
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
