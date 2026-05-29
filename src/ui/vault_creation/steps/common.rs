//! Shared helpers for vault creation steps

use crate::state::AppState;
use crate::ui::components::{button_large, Spacing};
use crate::ui::vault_creation::{DeviceRole, HardwareWalletType, VaultCreationState, VaultCreationStep};
use eframe::egui;

/// Render hardware wallet type selection UI consistently across all flows
///
/// This helper function ensures consistent UX/UI for hardware wallet type selection
/// in co-owner key scanning, view-only setup, and restore flows.
pub(crate) fn render_hardware_wallet_type_selection(
    ui: &mut egui::Ui,
    state: &mut VaultCreationState,
    grid_id: &str,
) {
    ui.add_space(Spacing::SM);
    ui.label("Select your hardware wallet type:");
    ui.add_space(Spacing::XS);

    // Create a grid of selectable hardware wallet type buttons
    let mut selected_type = state.selected_hw_type;
    egui::Grid::new(grid_id)
        .num_columns(2)
        .spacing([Spacing::SM, Spacing::SM])
        .show(ui, |ui| {
            for hw_type in HardwareWalletType::all_types() {
                let is_selected = selected_type == Some(hw_type);
                let button = egui::SelectableLabel::new(is_selected, hw_type.title());

                if ui.add(button).clicked() {
                    selected_type = Some(hw_type);
                }
            }
        });

    // Update state if selection changed
    if selected_type != state.selected_hw_type {
        state.selected_hw_type = selected_type;
        // Reset scanner when type changes
        if state.hw_batch_qr_scanner_state.success {
            state.hw_batch_qr_scanner_state.reset();
            if state.coowner_keys.is_some() {
                state.coowner_keys = None;
            }
            if !state.import_descriptors_qr.is_empty() {
                state.import_descriptors_qr.clear();
            }
        }
    }

    ui.add_space(Spacing::MD);

    // Show guidance based on selected type (consistent messaging)
    if let Some(hw_type) = state.selected_hw_type {
        if hw_type.uses_multi_part_ur() {
            ui.colored_label(
                egui::Color32::from_rgb(255, 200, 0),
                format!("⚠ {} uses multi-part UR codes. You'll need to scan multiple QR codes in sequence.", hw_type.title())
            );
        } else {
            ui.label(
                egui::RichText::new(format!("{} uses single-part UR codes.", hw_type.title()))
                    .weak(),
            );
        }
        ui.add_space(Spacing::SM);
    } else {
        ui.colored_label(
            egui::Color32::YELLOW,
            "⚠ Please select your hardware wallet type above before scanning.",
        );
        ui.add_space(Spacing::SM);
    }
}

/// Optional hardware wallet naming step (parity with iOS NameHardwareWalletView).
pub fn render_name_hardware_wallet(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut VaultCreationState,
) {
    ui.heading("Name Your Hardware Wallet");
    ui.add_space(Spacing::MD);
    ui.label(
        "Optionally give this hardware wallet a name to easily identify it in your vault.",
    );
    ui.add_space(Spacing::SM);
    ui.label(
        egui::RichText::new(
            "This name is private and visible only to you. You can change it later in settings.",
        )
        .weak(),
    );
    ui.add_space(Spacing::LG);

    let name_field = if state.hw_naming_index == 1 {
        &mut state.second_hw_display_name
    } else {
        &mut state.first_hw_display_name
    };

    ui.label("Hardware Wallet Name");
    ui.text_edit_singleline(name_field);
    ui.add_space(Spacing::MD);

    if let Err(e) = bitvault_common::validate_hardware_wallet_name(name_field) {
        ui.colored_label(egui::Color32::RED, e);
    }

    ui.add_space(Spacing::XL);

    let skip_label = if name_field.trim().is_empty() {
        "Skip"
    } else {
        "Continue"
    };

    if button_large(ui, skip_label).clicked() {
        if let Err(e) = bitvault_common::validate_hardware_wallet_name(name_field) {
            state.error = Some(e);
            return;
        }
        state.error = None;

        match state.device_role {
            DeviceRole::SingleDeviceHWHW if state.hw_naming_index == 0 => {
                state.scanning_second_hw = true;
                state.hw_batch_qr_scanner_state.reset();
                state.selected_hw_type = None;
                state.advance_to_step(VaultCreationStep::ScanCoownerKeys);
            }
            DeviceRole::SingleDeviceHWHW => {
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
            DeviceRole::Restore => {
                restore_vault_after_hw_naming(ui, app_state, state);
            }
            _ => {
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
        }
    }
}

fn restore_vault_after_hw_naming(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut VaultCreationState,
) {
    use bip39::{Language, Mnemonic};

    if state.is_importing {
        ui.spinner();
        ui.label("Restoring vault...");
        return;
    }

    let mnemonic = match Mnemonic::parse_in(Language::English, state.import_mnemonic_text.trim()) {
        Ok(m) => m,
        Err(e) => {
            state.error = Some(format!(
                "Invalid seed phrase: {}",
                crate::utils::sanitize_error_for_ui(&e)
            ));
            return;
        }
    };

    let Some(ref runtime) = app_state.runtime else {
        state.error = Some("Runtime not available".to_string());
        return;
    };

    state.is_importing = true;
    state.error = None;

    let descriptors_qr = state.import_descriptors_qr.clone();
    let vault_name = state.vault_name.clone();
    let network = app_state.network;
    let hw_type = state.selected_hw_type.map(|t| t.title().to_string());
    let display_names = state.compute_hardware_wallet_display_names();
    let runtime_handle = runtime.handle().clone();

    let result: Result<(bitvault_common::wallet::VaultService, String), String> =
        runtime.block_on(async {
            let mut vault_service = bitvault_common::wallet::VaultService::new(network);
            vault_service
                .import_vault(
                    &mnemonic,
                    &descriptors_qr,
                    &vault_name,
                    false,
                    hw_type.as_deref(),
                    display_names,
                )
                .await
                .map_err(|e| {
                    format!(
                        "Restore failed: {}",
                        crate::utils::sanitize_error_for_ui(&e)
                    )
                })?;

            let vault_address = vault_service.get_address().map_err(|e| {
                format!(
                    "Failed to get address: {}",
                    crate::utils::sanitize_error_for_ui(&e)
                )
            })?;
            Ok((vault_service, vault_address))
        });

    match result {
        Ok((vault_service, vault_address)) => {
            if let Err(e) = runtime_handle.block_on(async {
                app_state.initialize_vault_from_service(vault_service).await
            }) {
                state.error = Some(format!(
                    "Failed to initialize: {}",
                    crate::utils::sanitize_error_for_ui(&e)
                ));
                state.is_importing = false;
                return;
            }

            if let Some(ref mut handler) = app_state.async_handler {
                handler.fetch_balance();
                handler.fetch_address();
            }

            state.clear_sensitive_data();
            state.vault_address = Some(vault_address);
            state.is_importing = false;

            if let Some(next) = state.next_step_for_role() {
                state.advance_to_step(next);
            }
        }
        Err(e) => {
            state.error = Some(e);
            state.is_importing = false;
        }
    }
}
