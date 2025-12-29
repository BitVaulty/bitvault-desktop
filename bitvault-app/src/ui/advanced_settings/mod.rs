//! Advanced Settings UI
//!
//! Provides advanced configuration options:
//! - UTXO selection
//! - Detailed fee rate setting
//! - Backup management

pub mod utxo_selection;
pub mod fee_rate_setting;
pub mod backup_management;

pub use utxo_selection::{render_utxo_selection_view, UtxoSelectionViewState};
pub use fee_rate_setting::{render_fee_rate_setting, FeeRateSettingState};
pub use backup_management::{render_backup_management, BackupManagementState};

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
            if ui.selectable_label(
                state.current_tab == AdvancedSettingsTab::UtxoSelection,
                "UTXO Selection"
            ).clicked() {
                state.current_tab = AdvancedSettingsTab::UtxoSelection;
            }
            
            if ui.selectable_label(
                state.current_tab == AdvancedSettingsTab::FeeRate,
                "Fee Rate"
            ).clicked() {
                state.current_tab = AdvancedSettingsTab::FeeRate;
            }
            
            if ui.selectable_label(
                state.current_tab == AdvancedSettingsTab::Backup,
                "Backup"
            ).clicked() {
                state.current_tab = AdvancedSettingsTab::Backup;
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
        }
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        if ui.button("← Back").clicked() {
            navigation.go_back();
        }
    });
}
