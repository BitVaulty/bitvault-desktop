//! QR Code Scanner Component
//!
//! Scans QR codes from hardware wallets (for signed PSBTs)

use crate::state::{AppState, Navigation};
use crate::utils::qr::decode_qr_from_file;
use eframe::egui;
use std::path::PathBuf;

/// State for QR code scanner
pub struct QrScannerState {
    pub scanned_parts: Vec<String>,
    pub is_scanning: bool,
    pub error: Option<String>,
    pub success: bool,
    pub decoded_psbt: Option<String>,
    pub selected_file: Option<PathBuf>,
    pub pending_file_selection: bool,
}

impl Default for QrScannerState {
    fn default() -> Self {
        Self {
            scanned_parts: Vec::new(),
            is_scanning: true,
            error: None,
            success: false,
            decoded_psbt: None,
            selected_file: None,
            pending_file_selection: false,
        }
    }
}

/// Render QR code scanner for hardware wallet
pub fn render_qr_scanner(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut QrScannerState,
    title: &str,
    description: &str,
) {
    ui.vertical_centered(|ui| {
        ui.heading(title);
        ui.add_space(10.0);
        ui.label(description);
        ui.add_space(20.0);

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }

        // Show success message
        if state.success {
            ui.colored_label(egui::Color32::GREEN, "✓ QR code scanned successfully!");
            if let Some(ref psbt) = state.decoded_psbt {
                ui.label(format!("PSBT: {}...", &psbt[..psbt.len().min(50)]));
            }
            ui.add_space(20.0);
            if ui.button("Continue").clicked() {
                navigation.go_back();
            }
            return;
        }

        // Scanner UI
        if state.is_scanning {
            ui.label("Scan QR code from image file:");
            ui.add_space(10.0);

            // File selection button
            if ui.button("Select Image File").clicked() {
                state.pending_file_selection = true;
            }

            // Show selected file
            if let Some(ref file_path) = state.selected_file {
                ui.label(format!("Selected: {}", file_path.display()));
                ui.add_space(5.0);

                if ui.button("Scan QR Code from File").clicked() {
                    match decode_qr_from_file(file_path) {
                        Ok(decoded) => {
                            state.scanned_parts.push(decoded);
                            decode_ur_parts(ui, app_state, state);
                        }
                        Err(e) => {
                            state.error = Some(e);
                        }
                    }
                }
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Manual input option (for testing/debugging)
            ui.label("Or enter UR string manually:");
            ui.add_space(5.0);

            let mut manual_input = String::new();
            ui.text_edit_singleline(&mut manual_input);

            if ui.button("Submit UR String").clicked() && !manual_input.is_empty() {
                state.scanned_parts.push(manual_input.clone());
                // Try to decode
                decode_ur_parts(ui, app_state, state);
            }
        }

        // Handle file selection (non-blocking)
        if state.pending_file_selection {
            state.pending_file_selection = false;
            // Use rfd for file dialog
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image files", &["png", "jpg", "jpeg", "gif", "bmp"])
                .pick_file()
            {
                state.selected_file = Some(path);
            }
        }

        // Show scanned parts progress
        if !state.scanned_parts.is_empty() {
            ui.add_space(10.0);
            ui.label(format!("Scanned {} part(s)", state.scanned_parts.len()));
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Cancel button
        if ui.button("Cancel").clicked() {
            navigation.go_back();
        }
    });
}

fn decode_ur_parts(_ui: &mut egui::Ui, app_state: &mut AppState, state: &mut QrScannerState) {
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        // Use all scanned parts (for multi-part UR)
        let ur_parts = state.scanned_parts.clone();

        let result = runtime.block_on(async {
            let vs = vault_service.read().await;
            vs.decode_ur_psbt_to_psbt_base64(&ur_parts)
        });

        match result {
            Ok(psbt_base64) => {
                state.decoded_psbt = Some(psbt_base64);
                state.success = true;
                state.is_scanning = false;
            }
            Err(e) => {
                state.error = Some(format!("Failed to decode UR: {}", e));
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
    }
}
