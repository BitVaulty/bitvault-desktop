//! UTXO Refresh Flow
//!
//! Flow for refreshing UTXOs older than 6 months

use super::utxo_selection::{render_utxo_selection, RecoveryMode, UtxoSelectionState};
use crate::state::{AppState, Navigation};
use bdk::bitcoin::{OutPoint, Txid};
use eframe::egui;
use std::str::FromStr;

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

        REFRESH_STATE.with(|state_ref| {
            let mut state = state_ref.borrow_mut();

            match &mut *state {
                UtxoRefreshState::LoadingUtxos => {
                    load_old_utxos(ui, app_state, &mut state, true);
                }
                UtxoRefreshState::SelectingUtxos(selection_state) => {
                    let selection_state_clone = selection_state.clone();
                    render_utxo_selection(ui, selection_state, RecoveryMode::Refresh);
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            navigation.go_back();
                        }
                        if ui.button("Continue").clicked() && selection_state_clone.has_selection()
                        {
                            // Build refresh transaction with selected UTXOs
                            // Use a temporary variable to avoid borrow conflicts
                            let next_state =
                                build_refresh_transaction_state(app_state, &selection_state_clone);
                            *state = next_state;
                        }
                    });
                }
                UtxoRefreshState::Signing { psbt_base64 } => {
                    let psbt_clone = psbt_base64.clone();
                    // Use a temporary variable to avoid borrow conflicts
                    let next_state = sign_refresh_transaction_state(app_state, &psbt_clone);
                    *state = next_state;
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

fn build_refresh_transaction_state(
    app_state: &mut AppState,
    selection_state: &UtxoSelectionState,
) -> UtxoRefreshState {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        // Get vault address (destination for refresh - send to self)
        let vault_address = match app_state.vault_data.lock() {
            Ok(data) => data.receive_address.clone().unwrap_or_default(),
            Err(_) => {
                return UtxoRefreshState::Error("Failed to get vault address".to_string());
            }
        };

        if vault_address.is_empty() {
            return UtxoRefreshState::Error("Vault address not available".to_string());
        }

        // Convert selected outpoints from strings to OutPoint
        // Note: OutPoint and Txid are already imported at the top
        let utxos_to_spend: Result<Vec<OutPoint>, _> = selection_state
            .selected
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

        let utxos_to_spend = match utxos_to_spend {
            Ok(utxos) => utxos,
            Err(e) => {
                return UtxoRefreshState::Error(e);
            }
        };

        // Convert OutPoints to strings for the API
        let utxo_strings: Vec<String> = utxos_to_spend
            .iter()
            .map(|op| format!("{}:{}", op.txid, op.vout))
            .collect();

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
                let psbt_base64 = preview.psbt.clone();
                UtxoRefreshState::Signing { psbt_base64 }
            }
            Err(e) => {
                UtxoRefreshState::Error(format!("Failed to build refresh transaction: {}", e))
            }
        }
    } else {
        UtxoRefreshState::Error("Vault not loaded or runtime not available".to_string())
    }
}

fn sign_refresh_transaction_state(app_state: &mut AppState, psbt_base64: &str) -> UtxoRefreshState {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        // Get vault address (recipient for refresh)
        let vault_address = match app_state.vault_data.lock() {
            Ok(data) => data.receive_address.clone().unwrap_or_default(),
            Err(_) => {
                return UtxoRefreshState::Error("Failed to get vault address".to_string());
            }
        };

        if vault_address.is_empty() {
            return UtxoRefreshState::Error("Vault address not available".to_string());
        }

        // Sign and compress PSBT for sharing
        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.sign_and_share_recovery_tx(psbt_base64, &vault_address)
                .await
        });

        match result {
            Ok(compressed_psbt) => UtxoRefreshState::Sharing { compressed_psbt },
            Err(e) => UtxoRefreshState::Error(format!("Failed to sign refresh transaction: {}", e)),
        }
    } else {
        UtxoRefreshState::Error("Vault not loaded or runtime not available".to_string())
    }
}

fn sign_refresh_transaction(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut UtxoRefreshState,
    psbt_base64: &str,
) {
    ui.label("Signing refresh transaction...");
    ui.spinner();

    let next_state = sign_refresh_transaction_state(app_state, psbt_base64);
    *state = next_state;
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
