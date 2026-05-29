//! Full Settings View
//!
//! Comprehensive settings screen with all configuration options

use crate::settings::{AppTheme, Currency};
use crate::state::{AppState, Navigation};
use eframe::egui;

// Thread-local state for backup operation
thread_local! {
    static BACKUP_STATE: std::cell::RefCell<BackupState> =
        std::cell::RefCell::new(BackupState::default());
}

#[derive(Default)]
struct BackupState {
    is_exporting: bool,
    error: Option<String>,
    success_path: Option<String>,
}

fn export_manual_backup(ui: &mut egui::Ui, app_state: &mut AppState) {
    BACKUP_STATE.with(|state| {
        let mut state = state.borrow_mut();

        if state.is_exporting {
            ui.label("Exporting backup...");
            return;
        }

        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            if ui.button("Retry").clicked() {
                state.error = None;
            }
            return;
        }

        if let Some(ref path) = state.success_path {
            ui.colored_label(egui::Color32::GREEN, format!("✓ Backup saved to: {}", path));
            ui.label("You can now share or move this file to a safe location.");
            if ui.button("Clear").clicked() {
                state.success_path = None;
            }
            return;
        }

        // Start export
        state.is_exporting = true;
        state.error = None;

        if let (Some(vault_service), Some(runtime)) =
            (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
        {
            let backup_email = app_state.key_service.get_email().ok().flatten();
            let result = runtime.block_on(async {
                let vs = vault_service.read().await;
                vs.export_manual_backup(backup_email.as_deref()).await
            });

            match result {
                Ok(path) => {
                    state.success_path = Some(path);
                    state.is_exporting = false;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to export backup: {}", crate::utils::sanitize_error_for_ui(&e)));
                    state.is_exporting = false;
                }
            }
        } else {
            state.error = Some("Vault not loaded or runtime not available".to_string());
            state.is_exporting = false;
        }
    });
}

// Thread-local state for pCloud backup
thread_local! {
    static PCLOUD_BACKUP_STATE: std::cell::RefCell<PcloudBackupState> =
        std::cell::RefCell::new(PcloudBackupState::default());
}

#[derive(Default)]
struct PcloudBackupState {
    is_uploading: bool,
    error: Option<String>,
    success: bool,
    email_input: String,
    show_dialog: bool,
}

fn show_pcloud_backup_dialog(ui: &mut egui::Ui, app_state: &mut AppState) {
    PCLOUD_BACKUP_STATE.with(|state| {
        let mut state = state.borrow_mut();

        if !state.show_dialog {
            state.show_dialog = true;
            state.email_input.clear();
            state.error = None;
            state.success = false;
        }

        // Show dialog
        egui::Window::new("pCloud Backup")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Enter your email address to receive the pCloud backup link:");
                ui.add_space(10.0);

                ui.text_edit_singleline(&mut state.email_input);
                ui.add_space(10.0);

                if let Some(ref error) = state.error {
                    ui.colored_label(egui::Color32::RED, error);
                    ui.add_space(10.0);
                }

                if state.success {
                    ui.colored_label(egui::Color32::GREEN, "✓ Backup link sent to your email!");
                    ui.add_space(10.0);
                }

                // Buttons - centered
                let button_width = 140.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
                    egui::Sense::click(),
                );
                let mut button_ui =
                    ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                if button_ui.button("Cancel").clicked() {
                    state.show_dialog = false;
                    state.email_input.clear();
                    state.error = None;
                    state.success = false;
                }
                button_ui.add_space(10.0);
                if button_ui.button("Create Backup").clicked() && !state.email_input.is_empty() {
                    state.is_uploading = true;
                    state.error = None;
                    state.success = false;

                    if let (Some(vault_service), Some(runtime)) =
                        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
                    {
                        let email = state.email_input.clone();
                        let result = runtime.block_on(async {
                            let vs = vault_service.read().await;
                            vs.initialize_pcloud_backup(&email).await
                        });

                        match result {
                            Ok(_) => {
                                state.success = true;
                                state.is_uploading = false;
                            }
                            Err(e) => {
                                state.error =
                                    Some(format!("Failed to create pCloud backup: {}", crate::utils::sanitize_error_for_ui(&e)));
                                state.is_uploading = false;
                            }
                        }
                    } else {
                        state.error = Some("Vault not loaded or runtime not available".to_string());
                        state.is_uploading = false;
                    }
                }

                if state.is_uploading {
                    ui.label("Uploading backup to pCloud...");
                }
            });
    });
}

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Settings");

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Preferences section
            ui.label(egui::RichText::new("Preferences").heading());
            ui.add_space(10.0);

            // Currency selection
            ui.horizontal(|ui| {
                ui.label("Local Currency:");
                ui.add_space(10.0);

                let current_currency = app_state.currency.clone();
                egui::ComboBox::from_id_source("currency")
                    .selected_text(current_currency.code())
                    .show_ui(ui, |ui| {
                        for currency in Currency::all() {
                            let is_selected = current_currency == currency;
                            if ui
                                .selectable_label(
                                    is_selected,
                                    format!("{} ({})", currency.name(), currency.code()),
                                )
                                .clicked()
                            {
                                if let Err(e) =
                                    app_state.settings_manager.set_currency(currency.clone())
                                {
                                    eprintln!("Failed to save currency: {}", e);
                                } else {
                                    app_state.currency = currency;
                                }
                            }
                        }
                    });
            });

            ui.add_space(10.0);

            // Theme selection
            ui.horizontal(|ui| {
                ui.label("Appearance:");
                ui.add_space(10.0);

                let current_theme = app_state.theme.clone();
                egui::ComboBox::from_id_source("theme")
                    .selected_text(current_theme.display_name())
                    .show_ui(ui, |ui| {
                        for theme in AppTheme::all() {
                            let is_selected = current_theme == theme;
                            if ui
                                .selectable_label(is_selected, theme.display_name())
                                .clicked()
                            {
                                if let Err(e) = app_state.settings_manager.set_theme(theme.clone())
                                {
                                    eprintln!("Failed to save theme: {}", e);
                                } else {
                                    app_state.theme = theme;
                                }
                            }
                        }
                    });
            });

            ui.add_space(10.0);

            // Biometrics
            ui.label(egui::RichText::new("Biometrics").heading());
            ui.add_space(5.0);

            let biometric_service = crate::services::biometric_service::BiometricService::new();
            if let Some(runtime) = app_state.get_runtime() {
                let is_available = runtime.block_on(biometric_service.is_available());
                let biometric_type = runtime.block_on(biometric_service.get_biometric_type());
                let is_enabled = runtime.block_on(biometric_service.is_enabled());

                if is_available {
                    let mut enabled = is_enabled;
                    if ui
                        .checkbox(
                            &mut enabled,
                            format!("Enable {}", biometric_type.display_name()),
                        )
                        .changed()
                    {
                        runtime.block_on(biometric_service.set_enabled(enabled));
                        // Also save to settings
                        if let Err(e) = app_state.settings_manager.set_biometrics_enabled(enabled) {
                            eprintln!("Failed to save biometrics setting: {}", e);
                        }
                    }
                    ui.label(format!(
                        "{} is available on this device",
                        biometric_type.display_name()
                    ));
                    ui.label("Use biometric authentication as an alternative to PIN entry");
                } else {
                    ui.label(format!(
                        "{} is not available on this platform",
                        biometric_type.display_name()
                    ));
                }
            } else {
                ui.label("Runtime not available - biometric authentication unavailable");
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Network section
            ui.label(egui::RichText::new("Network Configuration").heading());
            ui.add_space(10.0);

            let current_network = app_state.network;
            let networks = [
                (bdk::bitcoin::Network::Bitcoin, "Mainnet"),
                (bdk::bitcoin::Network::Testnet, "Testnet"),
                (bdk::bitcoin::Network::Signet, "Signet"),
                (bdk::bitcoin::Network::Regtest, "Regtest"),
            ];

            for (network, name) in networks.iter() {
                let is_selected = current_network == *network;
                if ui.selectable_label(is_selected, *name).clicked() && !is_selected {
                    // Update network and reload vault
                    match app_state.update_network(*network) {
                        Ok(()) => {
                            // If no vault loaded, navigate to vault selection
                            if !app_state.is_vault_loaded() {
                                navigation.navigate_to(crate::state::View::VaultSelection);
                            }
                        }
                        Err(e) => {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("Failed to change network: {}", crate::utils::sanitize_error_for_ui(&e)),
                            );
                        }
                    }
                }
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Vault management section
            ui.label(egui::RichText::new("Vault Management").heading());
            ui.add_space(10.0);

            if app_state.is_vault_loaded() {
                ui.label("Current vault is loaded");
                ui.add_space(5.0);

                // Buttons - centered
                let button_width = 160.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
                    egui::Sense::click(),
                );
                let mut button_ui =
                    ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                if button_ui.button("Manual Backup (ZIP)").clicked() {
                    export_manual_backup(ui, app_state);
                }
                button_ui.add_space(10.0);
                if button_ui.button("pCloud Backup").clicked() {
                    show_pcloud_backup_dialog(ui, app_state);
                }

                ui.add_space(5.0);
                ui.label("Manual Backup: Download ZIP file with vault information");
                ui.label("pCloud Backup: Upload encrypted backup to pCloud");

                ui.add_space(5.0);

                if ui.button("Export Vault").clicked() {
                    // Export vault backup (manual backup ZIP)
                    if let (Some(runtime), Some(vault_service)) =
                        (app_state.runtime.as_ref(), app_state.vault_service.as_ref())
                    {
                        // Clone the Arc (cheap operation) - explicit type to help inference
                        let vault_service_clone: std::sync::Arc<
                            tokio::sync::RwLock<bitvault_common::wallet::VaultService>,
                        > = vault_service.clone();
                        let rt: &tokio::runtime::Runtime = runtime;

                        // Use block_on to export synchronously (acceptable for one-time operation)
                        let backup_email = app_state.key_service.get_email().ok().flatten();
                        let result: std::result::Result<String, bitvault_common::BitVaultError> =
                            rt.block_on(async {
                                let vault_guard = vault_service_clone.read().await;
                                vault_guard
                                    .export_manual_backup(backup_email.as_deref())
                                    .await
                            });

                        match result {
                            Ok(file_path) => {
                                // Show success message
                                ui.colored_label(
                                    egui::Color32::GREEN,
                                    format!("✓ Backup exported to: {}", file_path),
                                );
                                // Also copy path to clipboard for convenience
                                ui.output_mut(|o| {
                                    o.copied_text = file_path.clone();
                                });
                            }
                            Err(e) => {
                                ui.colored_label(
                                    egui::Color32::RED,
                                    format!("Failed to export backup: {}", crate::utils::sanitize_error_for_ui(&e)),
                                );
                            }
                        }
                    } else {
                        ui.colored_label(
                            egui::Color32::RED,
                            "No vault loaded or runtime not available",
                        );
                    }
                }
            } else {
                ui.label("No vault loaded");
                ui.add_space(10.0);
                if ui.button("Create New Vault").clicked() {
                    navigation.navigate_to(crate::state::View::VaultCreation);
                }
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Advanced section
            ui.label(egui::RichText::new("Advanced").heading());
            ui.add_space(10.0);

            if ui.button("Advanced Settings").clicked() {
                navigation.navigate_to(crate::state::View::AdvancedSettings);
            }
            ui.label("UTXO selection, fee rate configuration, and backup management");

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Blockchain Backend:");
                ui.label("Electrum (default)");
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Database:");
                ui.label("SQLite (persistent)");
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Recovery and UTXO Refresh section
            ui.label(egui::RichText::new("Recovery & Maintenance").heading());
            ui.add_space(10.0);

            if app_state.is_vault_loaded() {
                ui.horizontal(|ui| {
                    if ui.button("Recovery Transaction").clicked() {
                        navigation.navigate_to(crate::state::View::Recovery);
                    }
                    if ui.button("UTXO Refresh").clicked() {
                        navigation.navigate_to(crate::state::View::UtxoRefresh);
                    }
                });
                ui.add_space(5.0);
                ui.label("Recovery: Move UTXOs older than 1 year");
                ui.label("Refresh: Move UTXOs older than 6 months");
            } else {
                ui.label("Vault must be loaded to use recovery features");
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Secret Notifications section
            ui.label(egui::RichText::new("Secret Notifications").heading());
            ui.add_space(10.0);

            if app_state.is_vault_loaded() {
                if ui.button("Configure Telegram Notifications").clicked() {
                    navigation.navigate_to(crate::state::View::SecretNotification);
                }
                ui.label("Enable Telegram notifications for vault activity");
            } else {
                ui.label("Load a vault to configure notifications");
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Subscription section
            ui.label(egui::RichText::new("Subscription").heading());
            ui.add_space(10.0);

            if app_state.is_vault_loaded() {
                if ui.button("View Subscription").clicked() {
                    navigation.navigate_to(crate::state::View::Subscription);
                }
            } else {
                ui.label("Vault must be loaded to view subscription");
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Legal / Documents section
            ui.label(egui::RichText::new("Legal").heading());
            ui.add_space(10.0);

            if ui.button("Terms of Service").clicked() {
                ui.output_mut(|o| {
                    o.open_url = Some(egui::OpenUrl {
                        url: "https://bitvault.sv/terms".to_string(),
                        new_tab: true,
                    });
                });
            }
            if ui.button("Privacy Policy").clicked() {
                ui.output_mut(|o| {
                    o.open_url = Some(egui::OpenUrl {
                        url: "https://bitvault.sv/privacy".to_string(),
                        new_tab: true,
                    });
                });
            }
            ui.label("View Terms and Privacy Policy in your browser");

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Back button
            if ui.button("Back").clicked() {
                navigation.go_back();
            }
        });
    });
}
