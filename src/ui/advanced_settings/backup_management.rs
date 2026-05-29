//! Backup Management UI
//!
//! Comprehensive backup management interface

use crate::state::{AppState, Navigation};
use eframe::egui;

/// State for backup management
#[derive(Default)]
pub struct BackupManagementState {
    pub manual_backup_state: ManualBackupState,
    pub pcloud_backup_state: PcloudBackupState,
}

#[derive(Default)]
pub struct ManualBackupState {
    pub is_exporting: bool,
    pub error: Option<String>,
    pub success_path: Option<String>,
}

#[derive(Default)]
pub struct PcloudBackupState {
    pub is_uploading: bool,
    pub error: Option<String>,
    pub success: bool,
    pub email_input: String,
    pub show_dialog: bool,
}

/// Render backup management view
pub fn render_backup_management(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    _navigation: &mut Navigation,
    state: &mut BackupManagementState,
) {
    if !app_state.is_vault_loaded() {
        ui.label("No vault loaded");
        return;
    }

    ui.vertical(|ui| {
        ui.label(egui::RichText::new("Backup Management").heading());
        ui.add_space(10.0);

        ui.label("Create backups of your vault to protect against data loss.");
        ui.label("Backups are encrypted and can be restored on any device.");
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Manual Backup Section
        ui.label(egui::RichText::new("Manual Backup (ZIP)").strong());
        ui.add_space(5.0);
        ui.label("Download an encrypted ZIP file containing your vault data.");
        ui.add_space(5.0);

        let manual = &mut state.manual_backup_state;

        if manual.is_exporting {
            ui.label("Exporting backup...");
        } else if let Some(ref error) = manual.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            if ui.button("Retry").clicked() {
                manual.error = None;
            }
        } else if let Some(ref path) = manual.success_path {
            ui.colored_label(egui::Color32::GREEN, format!("✓ Backup saved to: {}", path));
            ui.label("You can now share or move this file to a safe location.");
            if ui.button("Clear").clicked() {
                manual.success_path = None;
            }
        } else if ui.button("Create Manual Backup").clicked() {
            manual.is_exporting = true;
            manual.error = None;

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
                        manual.success_path = Some(path);
                        manual.is_exporting = false;
                    }
                    Err(e) => {
                        manual.error = Some(format!("Failed to export backup: {}", crate::utils::sanitize_error_for_ui(&e)));
                        manual.is_exporting = false;
                    }
                }
            } else {
                manual.error = Some("Vault not loaded or runtime not available".to_string());
                manual.is_exporting = false;
            }
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // pCloud Backup Section
        ui.label(egui::RichText::new("pCloud Backup").strong());
        ui.add_space(5.0);
        ui.label("Upload an encrypted backup to pCloud and receive a link via email.");
        ui.add_space(5.0);

        let pcloud = &mut state.pcloud_backup_state;

        if pcloud.show_dialog {
            egui::Window::new("pCloud Backup")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Enter your email address to receive the pCloud backup link:");
                    ui.add_space(10.0);

                    ui.text_edit_singleline(&mut pcloud.email_input);
                    ui.add_space(10.0);

                    if let Some(ref error) = pcloud.error {
                        ui.colored_label(egui::Color32::RED, error);
                        ui.add_space(10.0);
                    }

                    if pcloud.success {
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
                        pcloud.show_dialog = false;
                        pcloud.email_input.clear();
                        pcloud.error = None;
                        pcloud.success = false;
                    }
                    button_ui.add_space(10.0);
                    if button_ui.button("Create Backup").clicked() && !pcloud.email_input.is_empty()
                    {
                        pcloud.is_uploading = true;
                        pcloud.error = None;
                        pcloud.success = false;

                        if let (Some(vault_service), Some(runtime)) =
                            (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
                        {
                            let email = pcloud.email_input.clone();
                            let result = runtime.block_on(async {
                                let vs = vault_service.read().await;
                                vs.initialize_pcloud_backup(&email).await
                            });

                            match result {
                                Ok(_) => {
                                    pcloud.success = true;
                                    pcloud.is_uploading = false;
                                }
                                Err(e) => {
                                    pcloud.error =
                                        Some(format!("Failed to create pCloud backup: {}", crate::utils::sanitize_error_for_ui(&e)));
                                    pcloud.is_uploading = false;
                                }
                            }
                        } else {
                            pcloud.error =
                                Some("Vault not loaded or runtime not available".to_string());
                            pcloud.is_uploading = false;
                        }
                    }

                    if pcloud.is_uploading {
                        ui.label("Uploading backup to pCloud...");
                    }
                });
        }

        if pcloud.is_uploading {
            ui.label("Uploading backup to pCloud...");
        } else if pcloud.success {
            ui.colored_label(egui::Color32::GREEN, "✓ Backup link sent to your email!");
        } else if ui.button("Create pCloud Backup").clicked() {
            pcloud.show_dialog = true;
            pcloud.email_input.clear();
            pcloud.error = None;
            pcloud.success = false;
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Backup information
        ui.label(egui::RichText::new("Backup Information").strong());
        ui.add_space(5.0);
        ui.label("• Backups are encrypted with your vault keys");
        ui.label("• Store backups in multiple secure locations");
        ui.label("• Regular backups are recommended");
        ui.label("• Backups can be restored on any device with the same vault");
    });
}
