//! View-only vault creation steps

use crate::state::{AppState, Navigation, View};
use crate::ui::components::{button, button_large, ButtonStyle, Spacing};
use crate::ui::vault_creation::{VaultCreationState, VaultCreationStep};
use eframe::egui;

pub fn render_scan_descriptor_view_only(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.heading("View-Only Setup");
    ui.add_space(Spacing::MD);

    ui.label("Scan or paste the descriptor from your mobile device or hardware wallet.");
    ui.add_space(Spacing::SM);

    ui.colored_label(
        egui::Color32::from_rgb(100, 149, 237),
        "This will let you monitor your vault without signing capability.",
    );
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
        super::common::render_hardware_wallet_type_selection(ui, state, "hw_type_selection_view_only");

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
                                    // Decode UR to get descriptor data
                                    if let Ok(Some(_message_bytes)) =
                                        bitvault_common::ur::decode_ur_part(
                                            &state
                                                .hw_batch_qr_scanner_state
                                                .scanned_parts
                                                .join("\n"),
                                        )
                                    {
                                        // Try to parse as AccountDescriptor
                                        match bitvault_common::ur::parse_crypto_account(
                                            &state.hw_batch_qr_scanner_state.scanned_parts[0],
                                        ) {
                                            Ok(_account_desc) => {
                                                // Store UR parts - backend will handle conversion
                                                state.import_descriptors_qr = state
                                                    .hw_batch_qr_scanner_state
                                                    .scanned_parts
                                                    .join("\n");
                                                state.hw_batch_qr_scanner_state.selected_file =
                                                    None;
                                                state.error = None;
                                            }
                                            Err(e) => {
                                                state.error = Some(format!("Failed to parse hardware wallet descriptor: {}", crate::utils::sanitize_error_for_ui(&e)));
                                            }
                                        }
                                    } else {
                                        // Store raw UR parts for backend processing
                                        state.import_descriptors_qr = state
                                            .hw_batch_qr_scanner_state
                                            .scanned_parts
                                            .join("\n");
                                        state.hw_batch_qr_scanner_state.selected_file = None;
                                        state.error = None;
                                    }
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

    if button_large(ui, "Continue").clicked() {
        if state.import_descriptors_qr.trim().is_empty() {
            state.error = Some("Please enter the descriptor configuration".to_string());
            return;
        }
        state.error = None;
        if let Some(next) = state.next_step_for_role() {
            state.advance_to_step(next);
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// View-only setup complete
pub fn render_view_only_complete(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.heading("View-Only Setup");
    ui.add_space(Spacing::MD);

    if state.is_importing {
        ui.spinner();
        ui.label("Setting up view-only wallet...");
        return;
    }

    if state.vault_address.is_some() {
        // Success
        ui.colored_label(egui::Color32::GREEN, "✓ View-only wallet created!");
        ui.add_space(Spacing::MD);
        ui.label("You can now monitor your vault balance and transactions.");
        ui.label("Signing transactions will require your mobile device.");
        ui.add_space(Spacing::XL);

        if button_large(ui, "Open Wallet").clicked() {
            navigation.navigate_to(View::Dashboard { tab: 0 });
        }
    } else {
        // Setup button
        ui.label("Ready to set up view-only wallet?");
        ui.add_space(Spacing::LG);

        if button_large(ui, "Create View-Only Wallet").clicked() {
            state.is_importing = true;
            state.error = None;

            if let Some(ref runtime) = app_state.runtime {
                let descriptors_qr = state.import_descriptors_qr.clone();
                let vault_name = state.vault_name.clone();
                let network = app_state.network;
                let runtime_handle = runtime.handle().clone();

                // For view-only, we use a dummy mnemonic since we don't need signing
                let dummy_mnemonic = bitvault_common::generate_mnemonic(12)
                    .expect("Failed to generate dummy mnemonic");

                let result: Result<(bitvault_common::wallet::VaultService, String), String> =
                    runtime.block_on(async {
                        let mut vault_service = bitvault_common::wallet::VaultService::new(network);
                        vault_service
                            .import_vault(
                                &dummy_mnemonic,
                                &descriptors_qr,
                                &vault_name,
                                false,
                                None,
                            )
                            .await
                            .map_err(|e| format!("Setup failed: {}", crate::utils::sanitize_error_for_ui(&e)))?;

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

                        // Clear any sensitive data from memory
                        state.clear_sensitive_data();

                        state.vault_address = Some(vault_address);
                        state.is_importing = false;

                        // Navigate to dashboard
                        state.advance_to_step(VaultCreationStep::Completed);
                        navigation.navigate_to(View::Dashboard { tab: 0 });
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
    }

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}
