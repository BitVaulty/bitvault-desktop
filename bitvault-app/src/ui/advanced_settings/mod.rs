//! Advanced Settings UI
//!
//! Provides advanced configuration options:
//! - UTXO selection
//! - Detailed fee rate setting
//! - Backup management

pub mod backup_management;
pub mod fee_rate_setting;
pub mod utxo_selection;

pub use backup_management::{render_backup_management, BackupManagementState};
pub use fee_rate_setting::{render_fee_rate_setting, FeeRateSettingState};
pub use utxo_selection::{render_utxo_selection_view, UtxoSelectionViewState};

/// Advanced settings main view
pub struct AdvancedSettingsState {
    pub current_tab: AdvancedSettingsTab,
    pub utxo_selection: UtxoSelectionViewState,
    pub fee_rate_setting: FeeRateSettingState,
    pub backup_management: BackupManagementState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancedSettingsTab {
    UtxoSelection,
    FeeRate,
    Backup,
    Security,
}

impl Default for AdvancedSettingsState {
    fn default() -> Self {
        Self {
            current_tab: AdvancedSettingsTab::UtxoSelection,
            utxo_selection: UtxoSelectionViewState::default(),
            fee_rate_setting: FeeRateSettingState::default(),
            backup_management: BackupManagementState::default(),
        }
    }
}

/// Render advanced settings view
pub fn render_advanced_settings(
    ui: &mut egui::Ui,
    app_state: &mut crate::state::AppState,
    navigation: &mut crate::state::Navigation,
    state: &mut AdvancedSettingsState,
) {
    ui.vertical(|ui| {
        ui.heading("Advanced Settings");
        ui.add_space(10.0);

        // Tab selection
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    state.current_tab == AdvancedSettingsTab::UtxoSelection,
                    "UTXO Selection",
                )
                .clicked()
            {
                state.current_tab = AdvancedSettingsTab::UtxoSelection;
            }

            if ui
                .selectable_label(
                    state.current_tab == AdvancedSettingsTab::FeeRate,
                    "Fee Rate",
                )
                .clicked()
            {
                state.current_tab = AdvancedSettingsTab::FeeRate;
            }

            if ui
                .selectable_label(state.current_tab == AdvancedSettingsTab::Backup, "Backup")
                .clicked()
            {
                state.current_tab = AdvancedSettingsTab::Backup;
            }

            if ui
                .selectable_label(state.current_tab == AdvancedSettingsTab::Security, "Security")
                .clicked()
            {
                state.current_tab = AdvancedSettingsTab::Security;
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Render current tab
        match state.current_tab {
            AdvancedSettingsTab::UtxoSelection => {
                render_utxo_selection_view(ui, app_state, navigation, &mut state.utxo_selection);
            }
            AdvancedSettingsTab::FeeRate => {
                render_fee_rate_setting(ui, app_state, &mut state.fee_rate_setting);
            }
            AdvancedSettingsTab::Backup => {
                render_backup_management(ui, app_state, navigation, &mut state.backup_management);
            }
            AdvancedSettingsTab::Security => {
                render_security_settings(ui);
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        if ui.button("Back").clicked() {
            navigation.go_back();
        }
    });
}

/// Render security settings (PIN management)
fn render_security_settings(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("Security Settings");
        ui.add_space(20.0);

        let pin_service = bitvault_common::PinService::new();
        let has_pin = pin_service.has_pin();

        if has_pin {
            ui.label("PIN is currently set.");
            ui.add_space(10.0);
            
            ui.colored_label(
                egui::Color32::YELLOW,
                "Warning: Resetting your PIN will remove it completely. You will need to set a new PIN if you want to use PIN protection again."
            );
            ui.add_space(20.0);

            if ui.button("Reset PIN").clicked() {
                match pin_service.delete_pin() {
                    Ok(_) => {
                        ui.colored_label(
                            egui::Color32::GREEN,
                            "✓ PIN has been reset. You can now use the app without PIN authentication."
                        );
                        eprintln!("[PIN_RESET] PIN successfully deleted");
                    }
                    Err(e) => {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("Failed to reset PIN: {}", e)
                        );
                        eprintln!("[PIN_RESET] Failed to delete PIN: {:?}", e);
                    }
                }
            }
        } else {
            ui.label("No PIN is currently set.");
            ui.add_space(10.0);
            ui.label("You can set a PIN from the vault creation flow or settings.");
        }
    });
}
