//! Restore from backup vault creation steps

use crate::state::{AppState, Navigation};
use crate::ui::components::{button, button_large, ButtonStyle, Spacing};
use crate::ui::vault_creation::VaultCreationState;
use bip39::{Language, Mnemonic};
use eframe::egui;

pub fn render_select_seed_phrase_size(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.heading("Restore from Backup");
    ui.add_space(Spacing::MD);

    ui.label("How many words is your seed phrase?");
    ui.add_space(Spacing::LG);

    ui.vertical_centered(|ui| {
        if button_large(ui, "12 words").clicked() {
            state.import_seed_phrase_size = Some(12);
            state.error = None;
            if let Some(next) = state.next_step_for_role() {
                state.advance_to_step(next);
            }
        }
        ui.add_space(Spacing::MD);
        if button_large(ui, "24 words").clicked() {
            state.import_seed_phrase_size = Some(24);
            state.error = None;
            if let Some(next) = state.next_step_for_role() {
                state.advance_to_step(next);
            }
        }
    });

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// Enter seed phrase from paper backup
pub fn render_enter_seed_phrase(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.heading("Restore from Backup");
    ui.add_space(Spacing::MD);

    // Warning banner
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(80, 60, 0))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::YELLOW, "⚠");
                ui.colored_label(
                    egui::Color32::WHITE,
                    "Only use your PAPER BACKUP - the seed phrase you wrote down during vault creation."
                );
            });
        });

    ui.add_space(Spacing::LG);

    let size = state.import_seed_phrase_size.unwrap_or(12);
    ui.label(format!("Enter your {} word seed phrase:", size));
    ui.add_space(Spacing::SM);

    ui.add(
        egui::TextEdit::multiline(&mut state.import_mnemonic_text)
            .hint_text("word1 word2 word3 word4 ...")
            .desired_width(400.0)
            .desired_rows(4)
            .password(true),
    ); // Hide for security

    ui.add_space(Spacing::SM);
    ui.label(
        egui::RichText::new("Your seed phrase is never transmitted and stays on this device.")
            .small()
            .color(egui::Color32::GRAY),
    );

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Continue").clicked() {
        let words: Vec<&str> = state.import_mnemonic_text.split_whitespace().collect();

        if words.is_empty() {
            state.error = Some("Please enter your seed phrase".to_string());
            return;
        }

        let expected = state.import_seed_phrase_size.unwrap_or(12);
        if words.len() != expected as usize {
            state.error = Some(format!(
                "Seed phrase should be {} words (you entered {})",
                expected,
                words.len()
            ));
            return;
        }

        // Validate mnemonic
        match Mnemonic::parse_in(Language::English, state.import_mnemonic_text.trim()) {
            Ok(_) => {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
            Err(e) => {
                state.error = Some(format!("Invalid seed phrase: {}", crate::utils::sanitize_error_for_ui(&e)));
            }
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// Scan descriptor QR for restore flow
pub fn render_scan_descriptor_restore(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    _navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.heading("Restore from Backup");
    ui.add_space(Spacing::MD);

    ui.label("Now enter the descriptor configuration from your mobile device or hardware wallet.");
    ui.add_space(Spacing::SM);
    ui.label("On your mobile, go to Settings → Export Vault Descriptor.");
    ui.add_space(Spacing::MD);

    // Option: Hardware Wallet or Mobile Device
    ui.horizontal(|ui| {
        ui.label("Source:");
        let hw_mode = !state.hw_batch_qr_scanner_state.scanned_parts.is_empty()
            || state.hw_batch_qr_scanner_state.pending_file_selection
            || state.hw_batch_qr_scanner_state.is_scanning;
        if ui.selectable_label(hw_mode, "Hardware Wallet").clicked() && !hw_mode {
            state.hw_batch_qr_scanner_state =
                crate::ui::hardware_wallet::BatchQrScannerState::default();
            state.import_descriptors_qr.clear();
        }
        if ui.selectable_label(!hw_mode, "Mobile Device").clicked() && hw_mode {
            state.hw_batch_qr_scanner_state.reset();
        }
    });

    if !state.hw_batch_qr_scanner_state.scanned_parts.is_empty()
        || state.hw_batch_qr_scanner_state.pending_file_selection
        || state.hw_batch_qr_scanner_state.is_scanning
    {
        // Hardware wallet QR scanning mode
        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::SM);

        ui.heading("Scan Hardware Wallet Descriptor");

        // Hardware wallet type selection (using consistent helper)
        super::common::render_hardware_wallet_type_selection(ui, state, "hw_type_selection_restore");

        ui.label("Scan QR code(s) from your hardware wallet:");
        ui.add_space(Spacing::SM);

        // Show progress
        if !state.hw_batch_qr_scanner_state.scanned_parts.is_empty() {
            ui.label(format!(
                "Scanned {} part(s)",
                state.hw_batch_qr_scanner_state.scanned_parts.len()
            ));
            ui.add_space(Spacing::SM);
        }

        // File selection for QR code image (only enabled if hardware wallet type is selected)
        let can_scan = state.selected_hw_type.is_some();
        if can_scan {
            if button(ui, "Select QR Code Image File", ButtonStyle::Secondary).clicked() {
                state.hw_batch_qr_scanner_state.pending_file_selection = true;
            }
        } else {
            // Show disabled button appearance
            ui.add_enabled(false, egui::Button::new("Select QR Code Image File"));
            ui.label(
                egui::RichText::new("Select a hardware wallet type above to enable scanning")
                    .weak(),
            );
        }

        // Handle file selection
        if state.hw_batch_qr_scanner_state.pending_file_selection {
            state.hw_batch_qr_scanner_state.pending_file_selection = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image files", &["png", "jpg", "jpeg", "gif", "bmp"])
                .pick_file()
            {
                state.hw_batch_qr_scanner_state.selected_file = Some(path);
            }
        }

        // Show selected file and scan button
        if let Some(ref file_path) = state.hw_batch_qr_scanner_state.selected_file {
            ui.add_space(Spacing::SM);
            ui.label(format!("Selected: {}", file_path.display()));
            ui.add_space(Spacing::XS);

            let file_path_clone = file_path.clone();
            if button(ui, "Scan QR Code from File", ButtonStyle::Primary).clicked() {
                match crate::utils::qr::decode_qr_from_file(&file_path_clone) {
                    Ok(decoded) => {
                        match state
                            .hw_batch_qr_scanner_state
                            .process_scanned_part(decoded)
                        {
                            Ok(is_complete) => {
                                if is_complete {
                                    // Hardware wallet descriptor QR scanned successfully
                                    // Store UR parts for backend processing
                                    state.import_descriptors_qr =
                                        state.hw_batch_qr_scanner_state.scanned_parts.join("\n");
                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                    state.error = None;
                                } else {
                                    // More parts needed
                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                    state.error = None;
                                }
                            }
                            Err(e) => {
                                state.error = Some(e);
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to decode QR code: {}", crate::utils::sanitize_error_for_ui(&e)));
                    }
                }
            }
        }

        // Show success/error
        if state.hw_batch_qr_scanner_state.success {
            ui.add_space(Spacing::SM);
            ui.colored_label(
                egui::Color32::GREEN,
                "✓ Hardware wallet descriptor scanned!",
            );

            // If hardware wallet descriptor was scanned, ask if this is a single device vault
            ui.add_space(Spacing::MD);
            ui.separator();
            ui.add_space(Spacing::SM);
            ui.label("Is this a single device vault (Seed + Hardware Wallet)?");
            ui.label(egui::RichText::new("If your vault uses a seed phrase on this device plus a hardware wallet, select the hardware wallet type below.").small().weak());
            ui.add_space(Spacing::SM);

            // Hardware wallet type selection for single device detection
            super::common::render_hardware_wallet_type_selection(
                ui,
                state,
                "hw_type_selection_restore_single_device",
            );
        }

        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::MD);
    }

    ui.add_space(Spacing::LG);

    ui.label("Paste the descriptor configuration:");
    ui.add_space(Spacing::SM);
    ui.add(
        egui::TextEdit::multiline(&mut state.import_descriptors_qr)
            .hint_text("Paste configuration from mobile app...")
            .desired_width(400.0)
            .desired_rows(3),
    );

    ui.add_space(Spacing::MD);

    // File load option
    ui.horizontal(|ui| {
        if button(ui, "Load from File", ButtonStyle::Secondary).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Text/JSON", &["txt", "json"])
                .pick_file()
            {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    state.import_descriptors_qr = contents;
                } else {
                    state.error = Some("Failed to read file".to_string());
                }
            }
        }
    });

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    if state.is_importing {
        ui.spinner();
        ui.label("Restoring vault...");
        return;
    }

    if button_large(ui, "Restore Vault").clicked() {
        if state.import_descriptors_qr.trim().is_empty() {
            state.error = Some("Please enter the descriptor configuration".to_string());
            return;
        }

        // Parse mnemonic
        let mnemonic =
            match Mnemonic::parse_in(Language::English, state.import_mnemonic_text.trim()) {
                Ok(m) => m,
                Err(e) => {
                    state.error = Some(format!("Invalid seed phrase: {}", crate::utils::sanitize_error_for_ui(&e)));
                    return;
                }
            };

        state.is_importing = true;
        state.error = None;

        if let Some(ref runtime) = app_state.runtime {
            let descriptors_qr = state.import_descriptors_qr.clone();
            let vault_name = state.vault_name.clone();
            let network = app_state.network;
            let runtime_handle = runtime.handle().clone();

            // Determine if this is a single device vault (seed+HW) based on hardware wallet type selection
            let hw_type_opt =
                if state.hw_batch_qr_scanner_state.success && state.selected_hw_type.is_some() {
                    // Hardware wallet descriptor was scanned and type selected - likely single device vault
                    state.selected_hw_type.map(|t| t.title().to_string())
                } else {
                    None
                };

            let result: Result<(bitvault_common::wallet::VaultService, String), String> = runtime
                .block_on(async {
                    let mut vault_service = bitvault_common::wallet::VaultService::new(network);
                    vault_service
                        .import_vault(
                            &mnemonic,
                            &descriptors_qr,
                            &vault_name,
                            false,
                            hw_type_opt.as_deref(),
                        )
                        .await
                        .map_err(|e| format!("Restore failed: {}", crate::utils::sanitize_error_for_ui(&e)))?;

                    let vault_address = vault_service
                        .get_address()
                        .map_err(|e| format!("Failed to get address: {}", crate::utils::sanitize_error_for_ui(&e)))?;
                    Ok((vault_service, vault_address))
                });

            match result {
                Ok((vault_service, vault_address)) => {
                    if let Err(e) = runtime_handle.block_on(async {
                        app_state.initialize_vault_from_service(vault_service).await
                    }) {
                        state.error = Some(format!("Failed to initialize: {}", crate::utils::sanitize_error_for_ui(&e)));
                        state.is_importing = false;
                        return;
                    }

                    if let Some(ref mut handler) = app_state.async_handler {
                        handler.fetch_balance();
                        handler.fetch_address();
                    }

                    // Clear sensitive data (seed phrase) from memory now that vault is restored
                    state.clear_sensitive_data();

                    state.vault_address = Some(vault_address);
                    state.is_importing = false;

                    // Go to PIN setup
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(e) => {
                    state.error = Some(e);
                    state.is_importing = false;
                }
            }
        } else {
            state.error = Some("Runtime not available".to_string());
            state.is_importing = false;
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}
