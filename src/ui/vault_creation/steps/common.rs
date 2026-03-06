//! Shared helpers for vault creation steps

use crate::ui::vault_creation::{HardwareWalletType, VaultCreationState};
use crate::ui::components::Spacing;
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
