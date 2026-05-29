//! Main device and co-owner vault creation steps

use crate::state::{AppState, Navigation, View};
use crate::ui::components::{button, button_large, ButtonStyle, Spacing};
use crate::ui::vault_creation::{
    DeviceRole, VaultCreationState, VaultCreationStep,
};
use base64::{engine::general_purpose, Engine};
use bitvault_common::key_exchange;
use bitvault_common::utils::TimeDelay;
use eframe::egui;

pub fn render_scan_coowner_keys(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut VaultCreationState,
) {
    // Different heading and instructions based on device role
    match state.device_role {
        DeviceRole::SingleDeviceSeedHW => {
            ui.heading("Scan Hardware Wallet Keys");
            ui.add_space(Spacing::MD);
            ui.label("Scan your hardware wallet's account QR code to add it as co-owner.");
        }
        DeviceRole::SingleDeviceHWHW => {
            if !state.scanning_second_hw && state.first_hw_keys.is_none() {
                ui.heading("Scan First Hardware Wallet");
                ui.add_space(Spacing::MD);
                ui.label(
                    "Scan the first hardware wallet's account QR code (this will be the owner).",
                );
            } else {
                ui.heading("Scan Second Hardware Wallet");
                ui.add_space(Spacing::MD);
                ui.label("Scan the second hardware wallet's account QR code (this will be the co-owner).");
            }
        }
        _ => {
            ui.heading("Get Co-owner's Keys");
            ui.add_space(Spacing::MD);
            ui.label("First, have your co-owner complete their setup and share their keys.");
            ui.label("Then scan the QR code from their device or paste the key data.");
            egui::CollapsingHeader::new("How it works")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label("1. Generate Keys: Your co-owner generates their public keys on their device.");
                    ui.label("2. Display QR Code: The co-owner displays a QR code with their keys.");
                    ui.label("3. Scan QR Code: Scan this QR code with your main device.");
                    ui.label("4. Complete Setup: After scanning, you'll proceed to complete the vault setup.");
                });
        }
    }
    ui.add_space(Spacing::MD);

    // Check if hardware wallet mode is active
    let hw_mode_active = !state.hw_batch_qr_scanner_state.scanned_parts.is_empty()
        || state.hw_batch_qr_scanner_state.pending_file_selection
        || state.hw_batch_qr_scanner_state.is_scanning
        || state.selected_hw_type.is_some();

    // Toggle between seed phrase and hardware wallet modes (only for regular 2-device setup)
    if state.device_role != DeviceRole::SingleDeviceSeedHW
        && state.device_role != DeviceRole::SingleDeviceHWHW
    {
        ui.horizontal(|ui| {
            ui.label("Co-owner type:");
            let seed_selected = !hw_mode_active;
            let hw_selected = hw_mode_active;

            if ui.selectable_label(seed_selected, "Seed Phrase").clicked() && hw_mode_active {
                // Switch to seed phrase mode
                state.hw_batch_qr_scanner_state.reset();
                state.selected_hw_type = None;
                state.coowner_keys = None;
            }
            if ui
                .selectable_label(hw_selected, "Hardware Wallet")
                .clicked()
                && !hw_mode_active
            {
                // Switch to hardware wallet mode
                if state.is_scanning_qr {
                    if let Some(ref mut camera) = state.camera_capture {
                        camera.stop_capture();
                    }
                    state.is_scanning_qr = false;
                }
                state.hw_batch_qr_scanner_state =
                    crate::ui::hardware_wallet::BatchQrScannerState::default();
            }
        });
    }

    if hw_mode_active {
        // Hardware wallet scanning mode
        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::SM);

        ui.heading("Scan Hardware Wallet QR");

        // Hardware wallet type selection (using consistent helper)
        super::common::render_hardware_wallet_type_selection(ui, state, "hw_type_selection_coowner");

        // Device-specific instructions (expandable)
        if let Some(hw_type) = state.selected_hw_type {
            egui::CollapsingHeader::new("Instructions")
                .default_open(false)
                .show(ui, |ui| {
                    for line in hw_type.instructions() {
                        ui.label(*line);
                    }
                });
        }

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
        if state.hw_batch_qr_scanner_state.pending_file_selection && can_scan {
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
                                    // Hardware wallet QR scanned successfully
                                    // Parse the decoded UR as AccountDescriptor and convert to CoownerKeys
                                    if let Some(ref decoded_base64) =
                                        state.hw_batch_qr_scanner_state.decoded_psbt
                                    {
                                        // The decoded_psbt field contains base64-encoded CBOR bytes from multi-part UR
                                        // Decode base64 and parse AccountDescriptor from CBOR bytes
                                        use base64::{engine::general_purpose, Engine as _};
                                        match general_purpose::STANDARD.decode(decoded_base64) {
                                            Ok(cbor_bytes) => {
                                                match bitvault_common::ur::parse_account_descriptor_from_cbor_bytes(&cbor_bytes) {
                                                    Ok(account_desc) => {
                                                        match bitvault_common::ur::convert_account_descriptor_to_coowner_keys(&account_desc) {
                                                            Ok(coowner_keys) => {
                                                                // For HW+HW single device, handle first vs second HW
                                                                if state.device_role == DeviceRole::SingleDeviceHWHW && !state.scanning_second_hw {
                                                                    state.first_hw_keys = Some(coowner_keys);
                                                                    state.first_hw_type = state.selected_hw_type;
                                                                    state.hw_naming_index = 0;
                                                                    state.error = None;
                                                                    state.advance_to_step(VaultCreationStep::NameHardwareWallet);
                                                                } else if state.device_role == DeviceRole::SingleDeviceHWHW
                                                                    && state.scanning_second_hw
                                                                {
                                                                    state.coowner_keys = Some(coowner_keys);
                                                                    state.coowner_pubkeys = state.hw_batch_qr_scanner_state.scanned_parts.join("\n");
                                                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                                                    state.hw_naming_index = 1;
                                                                    state.error = None;
                                                                    state.advance_to_step(VaultCreationStep::NameHardwareWallet);
                                                                } else if state.device_role == DeviceRole::SingleDeviceSeedHW {
                                                                    state.coowner_keys = Some(coowner_keys);
                                                                    state.coowner_pubkeys = state.hw_batch_qr_scanner_state.scanned_parts.join("\n");
                                                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                                                    state.hw_naming_index = 0;
                                                                    state.error = None;
                                                                    state.advance_to_step(VaultCreationStep::NameHardwareWallet);
                                                                } else {
                                                                    // Regular case or second HW in HW+HW mode
                                                                    state.coowner_keys = Some(coowner_keys);
                                                                    state.coowner_pubkeys = state.hw_batch_qr_scanner_state.scanned_parts.join("\n");
                                                                    state.hw_batch_qr_scanner_state.selected_file = None;
                                                                    state.error = None;
                                                                    // Auto-advance to next step
                                                                    if let Some(next) = state.next_step_for_role() {
                                                                        state.advance_to_step(next);
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                let error_msg = format!(
                                                                    "Failed to convert hardware wallet keys: {}\n\nThis may happen if:\n- The hardware wallet keys don't include the expected account-level derivation paths\n- The QR code data is incomplete or corrupted\n\nPlease try scanning again or verify your hardware wallet is configured correctly.",
                                                                    e
                                                                );
                                                                state.error = Some(error_msg);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let error_msg = format!(
                                                            "Failed to parse hardware wallet AccountDescriptor: {}\n\nThe QR code may not be from a hardware wallet account export, or the data format is invalid.\n\nPlease ensure you're scanning the correct QR code from your hardware wallet.",
                                                            e
                                                        );
                                                        state.error = Some(error_msg);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                state.error = Some(format!(
                                                    "Failed to decode QR code data: {}\n\nPlease try scanning the QR code again.",
                                                    e
                                                ));
                                            }
                                        }
                                    } else {
                                        state.error = Some(
                                            "No decoded QR data available.\n\nThis may indicate the QR code scanning didn't complete properly. Please try scanning again.".to_string()
                                        );
                                    }
                                } else {
                                    // More parts needed - clear file selection for next scan
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

        // Show success message
        if state.hw_batch_qr_scanner_state.success {
            ui.add_space(Spacing::MD);
            ui.colored_label(
                egui::Color32::GREEN,
                "✓ Hardware wallet QR codes scanned successfully!",
            );
        }

        // Show errors (support multi-line)
        if let Some(ref error) = state.hw_batch_qr_scanner_state.error {
            ui.add_space(Spacing::SM);
            // Split error by newlines and display each line
            for line in error.lines() {
                ui.colored_label(egui::Color32::RED, line);
            }
        }

        // Show state.error as well (for conversion errors)
        if let Some(ref error) = state.error {
            ui.add_space(Spacing::SM);
            // Display multi-line error messages
            for line in error.lines() {
                ui.colored_label(egui::Color32::RED, line);
            }
        }

        ui.add_space(Spacing::MD);
        ui.separator();
        ui.add_space(Spacing::MD);

        // Continue button when hardware wallet QR is scanned
        if state.hw_batch_qr_scanner_state.success {
            // For HW+HW mode, check if we're waiting for second HW
            if state.device_role == DeviceRole::SingleDeviceHWHW
                && state.first_hw_keys.is_some()
                && state.coowner_keys.is_none()
            {
                // First HW scanned, waiting for second HW - scanner was reset
                ui.label(
                    egui::RichText::new(
                        "First hardware wallet scanned! Now scan the second hardware wallet above.",
                    )
                    .color(egui::Color32::GREEN),
                );
            } else {
                // Verify we have coowner keys from conversion (or first_hw_keys for HW+HW)
                let can_continue = state.coowner_keys.is_some()
                    || (state.device_role == DeviceRole::SingleDeviceHWHW
                        && state.first_hw_keys.is_some()
                        && state.coowner_keys.is_some());

                if can_continue {
                    let continue_response = button_large(ui, "Continue");
                    if continue_response.clicked() {
                        // Hardware wallet keys are ready
                        state.error = None;
                        if let Some(next) = state.next_step_for_role() {
                            state.advance_to_step(next);
                        }
                    }
                } else {
                    // Show processing state or error
                    ui.spinner();
                    ui.label("Processing hardware wallet keys...");
                    if state.error.is_some() {
                        ui.add_space(Spacing::SM);
                        ui.label("If this takes too long, try scanning again.");
                    }
                }
            }
        } else if state.device_role != DeviceRole::SingleDeviceSeedHW
            && state.device_role != DeviceRole::SingleDeviceHWHW
            && button(ui, "← Use Seed Phrase Co-owner Instead", ButtonStyle::Text).clicked()
        {
            // Option to switch back to seed phrase mode (only for regular 2-device setup)
            state.hw_batch_qr_scanner_state.reset();
            state.selected_hw_type = None;
            state.coowner_keys = None;
            state.error = None;
        }

        // Don't show seed phrase scanning UI in hardware wallet mode
        return;
    }

    ui.add_space(Spacing::LG);

    // Webcam scanning option - centered (seed phrase co-owner mode)
    ui.vertical_centered(|ui| {
        if state.is_scanning_qr {
            if button(ui, "Stop Scanning", ButtonStyle::Secondary).clicked() {
                if let Some(ref mut camera) = state.camera_capture {
                    camera.stop_capture();
                }
                state.is_scanning_qr = false;
            }
        } else if button(ui, "📷 Scan QR Code", ButtonStyle::Secondary).clicked() {
            let mut camera = crate::utils::camera::CameraCapture::new();
            match camera.start_capture() {
                Ok(()) => {
                    state.camera_capture = Some(camera);
                    state.is_scanning_qr = true;
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to start camera: {}", crate::utils::sanitize_error_for_ui(&e)));
                }
            }
        }
    });

    // Show camera preview and scan
    if state.is_scanning_qr {
        if let Some(ref mut camera) = state.camera_capture {
            if let Some(texture) = camera.capture_frame(ctx) {
                ui.add_space(Spacing::MD);
                ui.image((texture.id(), egui::Vec2::new(400.0, 300.0)));
                ui.add_space(Spacing::SM);
                ui.label("Point camera at QR code");

                // Try to scan QR from current frame
                match camera.scan_qr_from_frame() {
                    Ok(qr_data) => {
                        // Validate it's co-owner keys
                        match bitvault_common::ur::decode_qr_data::<
                            bitvault_common::derivation::CoownerKeys,
                        >(&qr_data)
                        {
                            Ok(keys) => {
                                state.coowner_pubkeys = qr_data;
                                state.coowner_keys = Some(keys);
                                camera.stop_capture();
                                state.camera_capture = None;
                                state.is_scanning_qr = false;
                                state.error = None;
                                // Auto-advance to next step
                                if let Some(next) = state.next_step_for_role() {
                                    state.advance_to_step(next);
                                }
                                return;
                            }
                            Err(_) => {
                                // Not valid co-owner keys, keep scanning
                            }
                        }
                    }
                    Err(_) => {
                        // No QR code detected yet, keep scanning
                    }
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    ui.separator();
    ui.add_space(Spacing::MD);

    ui.label("Or paste co-owner's key data:");
    ui.add_space(Spacing::SM);

    let coowner_keys_response = ui.add(
        egui::TextEdit::multiline(&mut state.coowner_pubkeys)
            .hint_text("Paste the key data here...")
            .desired_width(400.0)
            .desired_rows(4),
    );

    // Auto-focus on step change
    if state.step_just_changed(VaultCreationStep::ScanCoownerKeys) {
        coowner_keys_response.request_focus();
    }

    ui.add_space(Spacing::MD);

    // Or load from file (try encrypted first, then plain JSON for backward compatibility)
    if button(ui, "Load from File", ButtonStyle::Secondary).clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt", "json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    // Try to parse as encrypted file first
                    match serde_json::from_str::<key_exchange::EncryptedCoownerKeysFile>(&contents)
                    {
                        Ok(encrypted_file) => {
                            // Decrypt
                            match key_exchange::decrypt_coowner_keys(&encrypted_file) {
                                Ok(coowner_keys) => {
                                    // Extract signature public key for File 2 encryption
                                    match general_purpose::STANDARD
                                        .decode(&encrypted_file.sender_public_key)
                                    {
                                        Ok(pubkey_bytes) => {
                                            match secp256k1::PublicKey::from_slice(&pubkey_bytes) {
                                                Ok(pubkey) => {
                                                    state.recipient_public_key = Some(pubkey);
                                                }
                                                Err(e) => {
                                                    state.error = Some(format!(
                                                        "Invalid public key in file: {}",
                                                        e
                                                    ));
                                                    return;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            state.error =
                                                Some(format!("Failed to decode public key: {}", crate::utils::sanitize_error_for_ui(&e)));
                                            return;
                                        }
                                    }

                                    // Encode decrypted keys back to JSON for display/QR
                                    match bitvault_common::ur::encode_qr_data(&coowner_keys) {
                                        Ok(keys_text) => {
                                            state.coowner_pubkeys = keys_text;
                                            state.coowner_keys = Some(coowner_keys);
                                            state.error = None;
                                            state.saved_key_file = Some(path);
                                        }
                                        Err(e) => {
                                            state.error = Some(format!(
                                                "Failed to encode decrypted keys: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    state.error = Some(format!("Failed to decrypt file: {}", crate::utils::sanitize_error_for_ui(&e)));
                                }
                            }
                        }
                        Err(_) => {
                            // Not encrypted, try plain JSON (backward compatibility)
                            state.coowner_pubkeys = contents.trim().to_string();
                            state.error = None;
                            state.saved_key_file = Some(path);
                        }
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Failed to read file: {}", crate::utils::sanitize_error_for_ui(&e)));
                }
            }
        }
    }

    // Show warning if file was loaded
    if state.saved_key_file.is_some() && !state.coowner_pubkeys.is_empty() {
        ui.add_space(Spacing::SM);
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 0),
            "⚠ Security: Delete the key file after successful vault creation.",
        );
    }

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    let continue_response = button_large(ui, "Continue");
    let continue_keyboard = continue_response.has_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
    if continue_response.clicked() || continue_keyboard {
        if state.coowner_pubkeys.trim().is_empty() {
            state.error = Some("Please scan, paste, or load the co-owner's key data".to_string());
        } else {
            // Try to decode as CoownerKeys (seed phrase co-owner)
            match bitvault_common::ur::decode_qr_data::<bitvault_common::derivation::CoownerKeys>(
                &state.coowner_pubkeys,
            ) {
                Ok(keys) => {
                    // Valid CoownerKeys format (seed phrase co-owner)
                    state.coowner_keys = Some(keys);
                    state.error = None;
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(_) => {
                    // Not CoownerKeys format - might be hardware wallet UR format
                    // Check if it starts with "ur:" (UR format)
                    if state.coowner_pubkeys.trim().starts_with("ur:") {
                        // Hardware wallet UR format - backend will parse AccountDescriptor
                        // For now, mark coowner_keys as None - backend will handle conversion
                        state.coowner_keys = None;
                        state.error = None;
                        if let Some(next) = state.next_step_for_role() {
                            state.advance_to_step(next);
                        }
                    } else {
                        state.error = Some("Invalid key data format. Expected CoownerKeys JSON or Hardware Wallet UR format.".to_string());
                    }
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        // Stop camera if scanning
        if state.is_scanning_qr {
            if let Some(ref mut camera) = state.camera_capture {
                camera.stop_capture();
            }
            state.is_scanning_qr = false;
        }
        state.go_to_previous_step();
    }
}

/// Co-owner device: Display own keys for main device
pub fn render_display_own_keys(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut VaultCreationState,
) {
    ui.heading("Share Your Keys");
    ui.add_space(Spacing::SM);

    ui.label("Share this with the main device to link as co-owner.");
    ui.add_space(Spacing::MD);

    // Generate keys text if not already done
    if state.my_keys_text.is_none() {
        if let Some(ref mnemonic) = state.mnemonic {
            match bitvault_common::derivation::get_owner_keys(mnemonic) {
                Ok(owner_keys) => match bitvault_common::ur::encode_qr_data(&owner_keys) {
                    Ok(keys_text) => {
                        state.my_keys_text = Some(keys_text);
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to encode keys: {}", crate::utils::sanitize_error_for_ui(&e)));
                    }
                },
                Err(e) => {
                    state.error = Some(format!("Failed to derive keys: {}", crate::utils::sanitize_error_for_ui(&e)));
                }
            }
        }
    }

    if let Some(ref keys_text) = state.my_keys_text {
        // Show QR code
        if let Some(qr_texture) = crate::utils::qr::generate_qr_image(ctx, keys_text) {
            ui.image((qr_texture.id(), egui::Vec2::new(280.0, 280.0)));
            ui.add_space(Spacing::SM);
        }

        // Copy and Save buttons on same row, centered
        let mut save_clicked = false;
        ui.horizontal(|ui| {
            // Calculate centering offset
            let button_width = 140.0 * 2.0 + Spacing::XS; // Two buttons + spacing
            let available = ui.available_width();
            if available > button_width {
                ui.add_space((available - button_width) / 2.0);
            }
            if button(ui, "Copy", ButtonStyle::Secondary).clicked() {
                ui.ctx().copy_text(keys_text.clone());
            }
            ui.add_space(Spacing::XS);
            if button(ui, "Save to File", ButtonStyle::Secondary).clicked() {
                save_clicked = true;
            }
        });

        // Handle save outside the horizontal block
        if save_clicked {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("coowner_keys.txt")
                .save_file()
            {
                // Parse keys from text
                match bitvault_common::ur::decode_qr_data::<bitvault_common::derivation::CoownerKeys>(
                    keys_text,
                ) {
                    Ok(coowner_keys) => {
                        // Encrypt and sign (generate signing key if not already stored)
                        let signing_key_opt = state.signing_secret_key.as_ref();
                        match key_exchange::encrypt_coowner_keys(&coowner_keys, signing_key_opt) {
                            Ok((encrypted_file, signing_key)) => {
                                // Store signing key for File 2 decryption
                                state.signing_secret_key = Some(signing_key);

                                // Serialize encrypted file to JSON
                                match serde_json::to_string_pretty(&encrypted_file) {
                                    Ok(json) => match std::fs::write(&path, json) {
                                        Ok(()) => {
                                            state.saved_key_file = Some(path.clone());
                                            state.error = None;
                                        }
                                        Err(e) => {
                                            state.error =
                                                Some(format!("Failed to save file: {}", crate::utils::sanitize_error_for_ui(&e)));
                                        }
                                    },
                                    Err(e) => {
                                        state.error = Some(format!(
                                            "Failed to serialize encrypted file: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                state.error = Some(format!("Failed to encrypt keys: {}", crate::utils::sanitize_error_for_ui(&e)));
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to parse keys: {}", crate::utils::sanitize_error_for_ui(&e)));
                    }
                }
            }
        }

        // Offer secure deletion if file was saved (compact inline)
        if let Some(ref file_path) = state.saved_key_file {
            if file_path.exists() {
                ui.add_space(Spacing::XS);
                let file_path_clone = file_path.clone();
                ui.horizontal(|ui| {
                    ui.label("✓ Saved");
                    if button(ui, "Delete", ButtonStyle::Danger).clicked() {
                        match bitvault_common::secure_delete_file(&file_path_clone)
                            .map_err(|e| e.to_string())
                        {
                            Ok(()) => {
                                state.saved_key_file = None;
                            }
                            Err(e) => {
                                state.error = Some(format!("Failed to delete: {}", crate::utils::sanitize_error_for_ui(&e)));
                            }
                        }
                    }
                });
            }
        }

        // Security warning (more compact)
        ui.add_space(Spacing::XS);
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 0),
            "⚠ Delete file after sharing",
        );
    }

    ui.add_space(Spacing::MD);

    if button_large(ui, "I've Shared My Keys").clicked() {
        if let Some(next) = state.next_step_for_role() {
            state.advance_to_step(next);
        }
    }

    ui.add_space(Spacing::SM);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// Co-owner device: Enter exchange data from main device
pub fn render_enter_exchange_data(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut VaultCreationState,
) {
    ui.heading("Enter Vault Configuration");
    ui.add_space(Spacing::MD);

    ui.label("The main device owner will share the vault configuration with you");
    ui.label("after they create the vault. Scan the QR code or paste it below.");
    ui.add_space(Spacing::LG);

    // Webcam scanning option
    ui.horizontal(|ui| {
        if state.is_scanning_qr {
            if button(ui, "Stop Scanning", ButtonStyle::Secondary).clicked() {
                if let Some(ref mut camera) = state.camera_capture {
                    camera.stop_capture();
                }
                state.is_scanning_qr = false;
            }
        } else if button(ui, "📷 Scan QR Code", ButtonStyle::Secondary).clicked() {
            let mut camera = crate::utils::camera::CameraCapture::new();
            match camera.start_capture() {
                Ok(()) => {
                    state.camera_capture = Some(camera);
                    state.is_scanning_qr = true;
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to start camera: {}", crate::utils::sanitize_error_for_ui(&e)));
                }
            }
        }
    });

    // Show camera preview and scan
    if state.is_scanning_qr {
        if let Some(ref mut camera) = state.camera_capture {
            if let Some(texture) = camera.capture_frame(ctx) {
                ui.add_space(Spacing::MD);
                ui.image((texture.id(), egui::Vec2::new(400.0, 300.0)));
                ui.add_space(Spacing::SM);
                ui.label("Point camera at QR code");

                // Try to scan QR from current frame
                match camera.scan_qr_from_frame() {
                    Ok(qr_data) => {
                        // Validate it's exchange data
                        match bitvault_common::ur::decode_qr_data::<
                            bitvault_common::ur::QrExchangeData,
                        >(&qr_data)
                        {
                            Ok(_) => {
                                state.exchange_data_input = qr_data;
                                camera.stop_capture();
                                state.camera_capture = None;
                                state.is_scanning_qr = false;
                                state.error = None;
                                // Auto-validate and continue
                                if let Some(next) = state.next_step_for_role() {
                                    state.advance_to_step(next);
                                }
                                return;
                            }
                            Err(_) => {
                                // Not valid exchange data, keep scanning
                            }
                        }
                    }
                    Err(_) => {
                        // No QR code detected yet, keep scanning
                    }
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    ui.separator();
    ui.add_space(Spacing::MD);

    ui.label("Or paste vault configuration:");
    ui.add_space(Spacing::SM);

    let exchange_data_response = ui.add(
        egui::TextEdit::multiline(&mut state.exchange_data_input)
            .hint_text("Paste the configuration data here...")
            .desired_width(400.0)
            .desired_rows(4),
    );

    // Auto-focus on step change
    if state.step_just_changed(VaultCreationStep::EnterExchangeData) {
        exchange_data_response.request_focus();
    }

    ui.add_space(Spacing::MD);

    if button(ui, "Load from File", ButtonStyle::Secondary).clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt", "json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    // Try to parse as encrypted file first
                    match serde_json::from_str::<key_exchange::EncryptedExchangeDataFile>(&contents)
                    {
                        Ok(encrypted_file) => {
                            // Decrypt using co-owner's signing private key
                            if let Some(ref signing_key) = state.signing_secret_key {
                                match key_exchange::decrypt_exchange_data(
                                    &encrypted_file,
                                    signing_key,
                                ) {
                                    Ok(exchange_data) => {
                                        // Encode decrypted data back to JSON for display
                                        match bitvault_common::ur::encode_qr_data(&exchange_data) {
                                            Ok(exchange_data_text) => {
                                                state.exchange_data_input = exchange_data_text;
                                                state.error = None;
                                                state.saved_exchange_file = Some(path);
                                            }
                                            Err(e) => {
                                                state.error = Some(format!(
                                                    "Failed to encode decrypted data: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        state.error =
                                            Some(format!("Failed to decrypt file: {}", crate::utils::sanitize_error_for_ui(&e)));
                                    }
                                }
                            } else {
                                state.error = Some("Missing signing key - cannot decrypt file. Please restart the workflow.".to_string());
                            }
                        }
                        Err(_) => {
                            // Not encrypted, try plain JSON (backward compatibility)
                            state.exchange_data_input = contents.trim().to_string();
                            state.error = None;
                            state.saved_exchange_file = Some(path);
                        }
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Failed to read file: {}", crate::utils::sanitize_error_for_ui(&e)));
                }
            }
        }
    }

    // Show warning if file was loaded
    if state.saved_exchange_file.is_some() && !state.exchange_data_input.is_empty() {
        ui.add_space(Spacing::SM);
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 0),
            "⚠ Security: Delete the config file after successful vault creation.",
        );
    }

    ui.add_space(Spacing::XL);

    let continue_response = button_large(ui, "Continue");
    let continue_keyboard = continue_response.has_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
    if continue_response.clicked() || continue_keyboard {
        if state.exchange_data_input.trim().is_empty() {
            state.error = Some("Please paste or load the vault configuration".to_string());
        } else {
            // Validate the exchange data
            match bitvault_common::ur::decode_qr_data::<bitvault_common::ur::QrExchangeData>(
                &state.exchange_data_input,
            ) {
                Ok(exchange_data) => {
                    // Store the main device's keys
                    state.coowner_keys = Some(exchange_data.coowner_public_keys);
                    // Extract time delay from exchange data
                    let time_delay = bitvault_common::utils::blocks_to_time_delay(
                        exchange_data.timelock_in_blocks,
                    );
                    state.time_delay_days = time_delay.days;
                    state.time_delay_hours = time_delay.hours;
                    state.error = None;
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Invalid configuration data: {}", crate::utils::sanitize_error_for_ui(&e)));
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        // Stop camera if scanning
        if state.is_scanning_qr {
            if let Some(ref mut camera) = state.camera_capture {
                camera.stop_capture();
            }
            state.is_scanning_qr = false;
        }
        state.go_to_previous_step();
    }
}

/// Email authentication step
pub fn render_email_auth(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut VaultCreationState,
) {
    ui.heading("Email Verification");
    ui.add_space(Spacing::MD);

    ui.label("Enter your email address to verify your identity:");
    ui.add_space(Spacing::MD);

    let email_response = ui.add(
        egui::TextEdit::singleline(&mut state.email)
            .hint_text("you@example.com")
            .desired_width(300.0)
            .margin(egui::vec2(8.0, 6.0)),
    );

    // Handle Enter key to send code
    let should_send_code = email_response.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && !state.code_sent;

    ui.add_space(Spacing::MD);

    if !state.code_sent {
        if button(ui, "Send Verification Code", ButtonStyle::Secondary).clicked()
            || should_send_code
        {
            if state.email.trim().is_empty() || !state.email.contains('@') {
                state.error = Some("Please enter a valid email address".to_string());
            } else if let Some(ref runtime) = app_state.runtime {
                // Check connectivity before network-dependent operation
                let is_online =
                    runtime.block_on(crate::services::network_check::check_connectivity());
                if !is_online {
                    state.error = Some(
                        "No internet connection. Please check your network and try again."
                            .to_string(),
                    );
                } else {
                    state.is_sending_code = true;
                    state.error = None;

                    let email = state.email.clone();
                    let network = app_state.network;
                    let result = runtime.block_on(async {
                        let temp_service = bitvault_common::wallet::VaultService::new(network);
                        temp_service.send_email_auth_code(&email).await
                    });

                    match result {
                        Ok(_) => {
                            state.code_sent = true;
                            state.is_sending_code = false;
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to send code: {}", crate::utils::sanitize_error_for_ui(&e)));
                            state.is_sending_code = false;
                        }
                    }
                }
            }
        }

        if state.is_sending_code {
            ui.spinner();
            ui.label("Sending...");
        }
    } else {
        ui.colored_label(egui::Color32::GREEN, "✓ Code sent! Check your email.");
        ui.add_space(Spacing::MD);

        ui.label("Enter the verification code:");
        ui.add_space(Spacing::SM);

        let auth_code_response = ui.add(
            egui::TextEdit::singleline(&mut state.auth_code)
                .hint_text("123456")
                .desired_width(150.0)
                .margin(egui::vec2(8.0, 6.0)),
        );

        // Auto-focus on step change (when code is sent)
        if state.step_just_changed(VaultCreationStep::EmailAuth) && state.code_sent {
            auth_code_response.request_focus();
        }

        // Handle Enter key to verify
        let should_verify =
            auth_code_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        ui.add_space(Spacing::XL);

        let verify_response = button_large(ui, "Verify & Continue");
        let verify_keyboard = verify_response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
        if verify_response.clicked() || should_verify || verify_keyboard {
            if state.auth_code.trim().is_empty() {
                state.error = Some("Please enter the verification code".to_string());
            } else {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.code_sent = false;
        state.auth_code.clear();
        state.go_to_previous_step();
    }
}

/// Create vault step
pub fn render_create_vault(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    let action_text = match state.device_role {
        DeviceRole::Main => "Create Vault",
        DeviceRole::Coowner => "Join Vault",
        DeviceRole::ViewOnly => "Set Up View-Only",
        DeviceRole::Restore => "Restore Vault",
        DeviceRole::SingleDeviceSeedHW => "Create Single Device Vault",
        DeviceRole::SingleDeviceHWHW => "Create Single Device Vault",
    };

    ui.heading(action_text);
    ui.add_space(Spacing::MD);

    if state.is_creating {
        ui.spinner();
        let status_text = match state.device_role {
            DeviceRole::Main => "Creating vault",
            DeviceRole::SingleDeviceSeedHW => "Creating single device vault",
            DeviceRole::SingleDeviceHWHW => "Creating single device vault",
            _ => "Joining vault",
        };
        ui.label(format!("{}...", status_text));
        return;
    }

    // Summary
    ui.label(format!("Vault Name: {}", state.vault_name));
    ui.label(format!(
        "Time Delay: {} days, {} hours",
        state.time_delay_days, state.time_delay_hours
    ));
    ui.label(format!("Email: {}", state.email));
    let role_display = match state.device_role {
        DeviceRole::Main => "Main Device",
        DeviceRole::Coowner => "Co-owner",
        DeviceRole::SingleDeviceSeedHW => "Single Device (Seed + HW)",
        DeviceRole::SingleDeviceHWHW => "Single Device (HW + HW)",
        _ => "Unknown",
    };
    ui.label(format!("Role: {}", role_display));

    ui.add_space(Spacing::XL);

    let create_response = button_large(ui, action_text);
    let create_keyboard = create_response.has_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
    if create_response.clicked() || create_keyboard {
        state.error = None;

        // Validation
        if state.vault_name.trim().is_empty() {
            state.error = Some("Vault name cannot be empty".to_string());
            state.is_creating = false;
            return;
        }

        // Validate email format (basic validation)
        let email = state.email.trim();
        if email.is_empty() {
            state.error = Some("Please enter an email address".to_string());
            state.is_creating = false;
            return;
        }
        if !email.contains('@') || !email.contains('.') || email.len() < 5 {
            state.error =
                Some("Please enter a valid email address (e.g., name@example.com)".to_string());
            state.is_creating = false;
            return;
        }
        // Check that @ is not at the start or end
        let at_pos = email.find('@').unwrap();
        if at_pos == 0 || at_pos == email.len() - 1 {
            state.error = Some("Please enter a valid email address".to_string());
            state.is_creating = false;
            return;
        }

        if state.auth_code.trim().is_empty() {
            state.error = Some("Please enter an authentication code".to_string());
            state.is_creating = false;
            return;
        }

        // Validate based on role
        match state.device_role {
            DeviceRole::SingleDeviceSeedHW => {
                // Need mnemonic and hardware wallet keys
                if state.mnemonic.is_none() {
                    state.error = Some("Seed phrase is required".to_string());
                    state.is_creating = false;
                    return;
                }
                if state.coowner_keys.is_none() && state.coowner_pubkeys.trim().is_empty() {
                    state.error = Some("Hardware wallet keys are required".to_string());
                    state.is_creating = false;
                    return;
                }
            }
            DeviceRole::SingleDeviceHWHW => {
                // Need both hardware wallets
                if state.first_hw_keys.is_none() {
                    state.error = Some("First hardware wallet keys are required".to_string());
                    state.is_creating = false;
                    return;
                }
                if state.coowner_keys.is_none() && state.coowner_pubkeys.trim().is_empty() {
                    state.error = Some("Second hardware wallet keys are required".to_string());
                    state.is_creating = false;
                    return;
                }
            }
            _ => {
                // Regular 2-device setup
                if state.coowner_pubkeys.trim().is_empty() && state.coowner_keys.is_none() {
                    state.error = Some("Co-owner keys are required".to_string());
                    state.is_creating = false;
                    return;
                }
            }
        }

        // Create/join vault
        if let Some(runtime) = app_state.runtime.as_ref() {
            // Check connectivity before vault creation (network-dependent)
            let is_online = runtime.block_on(crate::services::network_check::check_connectivity());
            if !is_online {
                state.error = Some(
                    "No internet connection. Please check your network and try again.".to_string(),
                );
            } else {
                state.is_creating = true;
                let time_delay = TimeDelay {
                    days: state.time_delay_days,
                    hours: state.time_delay_hours,
                };
                let coowner_pubkeys = state.coowner_pubkeys.clone();
                let vault_name = state.vault_name.clone();
                let network = app_state.network;
                let email = state.email.clone();
                let auth_code = state.auth_code.clone();
                let runtime_handle = runtime.handle().clone();

                // Prepare data for single device vaults (validation outside async block)
                let hw_keys_string_opt = match state.device_role {
                    DeviceRole::SingleDeviceSeedHW => {
                        // Convert hardware wallet keys to JSON string
                        if let Some(ref hw_keys) = state.coowner_keys {
                            match serde_json::to_string(hw_keys) {
                                Ok(s) => Some(s),
                                Err(e) => {
                                    state.error =
                                        Some(format!("Failed to serialize HW keys: {}", crate::utils::sanitize_error_for_ui(&e)));
                                    state.is_creating = false;
                                    return;
                                }
                            }
                        } else if !coowner_pubkeys.trim().is_empty() {
                            Some(coowner_pubkeys.clone())
                        } else {
                            None
                        }
                    }
                    DeviceRole::SingleDeviceHWHW => {
                        // Will be handled separately
                        None
                    }
                    _ => None,
                };

                let first_hw_string_opt = match state.device_role {
                    DeviceRole::SingleDeviceHWHW => {
                        if let Some(ref first_hw_keys) = state.first_hw_keys {
                            match serde_json::to_string(first_hw_keys) {
                                Ok(s) => Some(s),
                                Err(e) => {
                                    state.error =
                                        Some(format!("Failed to serialize first HW keys: {}", crate::utils::sanitize_error_for_ui(&e)));
                                    state.is_creating = false;
                                    return;
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let second_hw_string_opt = match state.device_role {
                    DeviceRole::SingleDeviceHWHW => {
                        if let Some(ref second_hw_keys) = state.coowner_keys {
                            match serde_json::to_string(second_hw_keys) {
                                Ok(s) => Some(s),
                                Err(e) => {
                                    state.error =
                                        Some(format!("Failed to serialize second HW keys: {}", crate::utils::sanitize_error_for_ui(&e)));
                                    state.is_creating = false;
                                    return;
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let result = runtime.block_on(async {
                    let mut vault_service = bitvault_common::wallet::VaultService::new(network);

                    let display_names = state.compute_hardware_wallet_display_names();

                    match state.device_role {
                        DeviceRole::SingleDeviceSeedHW => {
                            let mnemonic = state.mnemonic.as_ref().unwrap(); // Already validated
                            let hw_keys_string = hw_keys_string_opt.as_ref().unwrap(); // Already validated
                            let hw_type_str = state
                                .selected_hw_type
                                .map(|t| t.title().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            vault_service
                                .setup_single_device_vault_seed_hw(
                                    mnemonic,
                                    hw_keys_string,
                                    time_delay,
                                    &vault_name,
                                    &email,
                                    &auth_code,
                                    &hw_type_str,
                                    display_names,
                                )
                                .await
                                .map(|_| (None, vault_service)) // Single device doesn't return QR data
                        }
                        DeviceRole::SingleDeviceHWHW => {
                            let first_hw_string = first_hw_string_opt.as_ref().unwrap(); // Already validated
                            let second_hw_string = second_hw_string_opt.as_ref().unwrap(); // Already validated
                            let first_hw_type_str = state
                                .first_hw_type
                                .map(|t| t.title().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());
                            let second_hw_type_str = state
                                .selected_hw_type
                                .map(|t| t.title().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            vault_service
                                .setup_single_device_vault_hw_hw(
                                    first_hw_string,
                                    second_hw_string,
                                    time_delay,
                                    &vault_name,
                                    &email,
                                    &auth_code,
                                    &first_hw_type_str,
                                    &second_hw_type_str,
                                    display_names,
                                )
                                .await
                                .map(|_| (None, vault_service)) // Single device doesn't return QR data
                        }
                        _ => {
                            // Regular 2-device setup
                            let mnemonic = state.mnemonic.as_ref().unwrap(); // Already validated

                            let qr_result = vault_service
                                .setup_vault(
                                    mnemonic,
                                    &coowner_pubkeys,
                                    time_delay,
                                    &vault_name,
                                    &email,
                                    &auth_code,
                                )
                                .await;

                            match qr_result {
                                Ok(qr) => Ok((Some(qr), vault_service)),
                                Err(e) => Err(e),
                            }
                        }
                    }
                });

                match result {
                    Ok((exchange_data, vault_service)) => {
                        // Exchange data is only returned for 2-device setups
                        if let Some(qr_data) = exchange_data {
                            state.exchange_data_output = Some(qr_data);
                        }

                        if let Err(e) = runtime_handle.block_on(async {
                            app_state.initialize_vault_from_service(vault_service).await
                        }) {
                            state.error = Some(format!("Failed to initialize vault: {}", crate::utils::sanitize_error_for_ui(&e)));
                            state.is_creating = false;
                            return;
                        }

                        if let Some(ref mut handler) = app_state.async_handler {
                            handler.fetch_balance();
                            handler.fetch_address();
                        }

                        // Clear sensitive data (mnemonic) from memory now that vault is created
                        state.clear_sensitive_data();

                        state.is_creating = false;

                        // Single device vaults go directly to completed (no exchange data needed)
                        // Main device shows exchange data for 2-device setups, co-owner goes to completed
                        match state.device_role {
                            DeviceRole::Main => {
                                state.advance_to_step(VaultCreationStep::DisplayExchangeData);
                            }
                            DeviceRole::SingleDeviceSeedHW | DeviceRole::SingleDeviceHWHW => {
                                state.advance_to_step(VaultCreationStep::Completed);
                                navigation.navigate_to(View::Dashboard { tab: 0 });
                            }
                            _ => {
                                state.advance_to_step(VaultCreationStep::Completed);
                                navigation.navigate_to(View::Dashboard { tab: 0 });
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to create vault: {}", crate::utils::sanitize_error_for_ui(&e)));
                        state.is_creating = false;
                    }
                }
            }
        } else {
            state.error = Some("Missing mnemonic or runtime".to_string());
            state.is_creating = false;
        }
    }

    // Show Retry button when vault creation failed (e.g. network blip)
    if state.error.is_some() {
        ui.add_space(Spacing::MD);
        if button_large(ui, "Retry").clicked() {
            state.error = None;
        }
    }

    ui.add_space(Spacing::MD);
    if button(ui, "← Back", ButtonStyle::Text).clicked() {
        state.go_to_previous_step();
    }
}

/// Main device: Display exchange data for co-owner
pub fn render_display_exchange_data(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut VaultCreationState,
) {
    ui.heading("Share with Co-owner");
    ui.add_space(Spacing::MD);

    ui.colored_label(egui::Color32::GREEN, "✓ Vault created successfully!");
    ui.add_space(Spacing::MD);

    ui.label("Share this configuration with your co-owner.");
    ui.label("They will enter it on their device to join the vault.");
    ui.add_space(Spacing::LG);

    if let Some(ref exchange_data) = state.exchange_data_output {
        // Show QR code
        if let Some(qr_texture) = crate::utils::qr::generate_qr_image(ctx, exchange_data) {
            ui.image((qr_texture.id(), egui::Vec2::new(200.0, 200.0)));
            ui.add_space(Spacing::MD);
        }

        if button(ui, "Copy Configuration", ButtonStyle::Secondary).clicked() {
            ui.ctx().copy_text(exchange_data.clone());
        }

        ui.add_space(Spacing::SM);

        if button(ui, "Save to File", ButtonStyle::Secondary).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("vault_config.txt")
                .save_file()
            {
                // Parse exchange data from JSON
                match bitvault_common::ur::decode_qr_data::<bitvault_common::ur::QrExchangeData>(
                    exchange_data,
                ) {
                    Ok(exchange_data_struct) => {
                        // Encrypt with ECDH using co-owner's public key from File 1
                        if let Some(ref recipient_pubkey) = state.recipient_public_key {
                            match key_exchange::encrypt_exchange_data(
                                &exchange_data_struct,
                                recipient_pubkey,
                            ) {
                                Ok(encrypted_file) => {
                                    // Serialize encrypted file to JSON
                                    match serde_json::to_string_pretty(&encrypted_file) {
                                        Ok(json) => match std::fs::write(&path, json) {
                                            Ok(()) => {
                                                state.saved_exchange_file = Some(path.clone());
                                                state.error = None;
                                                ui.ctx().output_mut(|o| {
                                                    o.copied_text =
                                                        format!("Saved to: {}", path.display());
                                                });
                                            }
                                            Err(e) => {
                                                state.error =
                                                    Some(format!("Failed to save file: {}", crate::utils::sanitize_error_for_ui(&e)));
                                            }
                                        },
                                        Err(e) => {
                                            state.error = Some(format!(
                                                "Failed to serialize encrypted file: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    state.error =
                                        Some(format!("Failed to encrypt exchange data: {}", crate::utils::sanitize_error_for_ui(&e)));
                                }
                            }
                        } else {
                            // No recipient public key - save as plain JSON (backward compatibility)
                            match std::fs::write(&path, exchange_data) {
                                Ok(()) => {
                                    state.saved_exchange_file = Some(path.clone());
                                    state.error = None;
                                    ui.ctx().output_mut(|o| {
                                        o.copied_text = format!("Saved to: {}", path.display());
                                    });
                                }
                                Err(e) => {
                                    state.error = Some(format!("Failed to save file: {}", crate::utils::sanitize_error_for_ui(&e)));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to parse exchange data: {}", crate::utils::sanitize_error_for_ui(&e)));
                    }
                }
            }
        }

        // Security warning
        ui.add_space(Spacing::SM);
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 0),
            "⚠ Security: This file contains sensitive vault configuration. Delete it after use.",
        );

        // Offer secure deletion if file was saved
        if let Some(ref file_path) = state.saved_exchange_file {
            if file_path.exists() {
                ui.add_space(Spacing::SM);
                if button(ui, "🗑️ Delete Saved File", ButtonStyle::Danger).clicked() {
                    match bitvault_common::secure_delete_file(file_path).map_err(|e| e.to_string()) {
                        Ok(()) => {
                            state.saved_exchange_file = None;
                            ui.ctx().output_mut(|o| {
                                o.copied_text = "File securely deleted".to_string();
                            });
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to delete file: {}", crate::utils::sanitize_error_for_ui(&e)));
                        }
                    }
                }
            }
        }

        ui.add_space(Spacing::MD);

        ui.collapsing("Show Configuration Data", |ui| {
            ui.monospace(exchange_data);
        });
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Go to Dashboard").clicked() {
        state.advance_to_step(VaultCreationStep::Completed);
    }
}

