//! Vault selection UI
//!
//! Displays a list of available vaults and allows the user to:
//! - Select an existing vault to load
//! - Create a new vault
//! - Import a vault

use crate::state::{AppState, Navigation};
use crate::ui::components::{
    badge, button, button_large, card, BadgeStyle, ButtonStyle, Colors, Spacing, Typography,
};
use crate::ui::pin::render_pin_verification;
use eframe::egui;
use serde_json;

/// Vault selection state
pub struct VaultSelectionState {
    pub vaults: Vec<bitvault_common::wallet::VaultMetadata>,
    pub loading: bool,
    pub error: Option<String>,
    pub selected_index: Option<usize>,
    pub rename_dialog: Option<usize>, // Index of vault being renamed, None if dialog closed
    pub rename_text: String,          // Text for rename input
    pub import_dialog_open: bool,     // Whether import dialog is open
    pub import_text: String,          // Text for import (JSON metadata)
    pub pin_verification: crate::ui::pin::PinVerificationState,
    pub pending_delete_index: Option<usize>, // Index of vault pending deletion after PIN verification
}

impl Default for VaultSelectionState {
    fn default() -> Self {
        Self {
            vaults: Vec::new(),
            loading: false,
            error: None,
            selected_index: None,
            rename_dialog: None,
            rename_text: String::new(),
            import_dialog_open: false,
            import_text: String::new(),
            pin_verification: crate::ui::pin::PinVerificationState::new(),
            pending_delete_index: None,
        }
    }
}

/// Render vault selection screen
pub fn render(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultSelectionState,
    ctx: &egui::Context,
) {
    // Handle PIN verification for delete operation
    if let Some(index) = state.pending_delete_index {
        if render_pin_verification(ctx, &mut state.pin_verification) {
            // PIN verified - proceed with deletion
            if index < state.vaults.len() {
                let vault_address = state.vaults[index].address.clone();

                // Delete vault completely (metadata and database)
                match bitvault_common::wallet::VaultMetadata::delete_vault_complete(&vault_address)
                {
                    Ok(_) => {
                        // Remove from list
                        state.vaults.remove(index);
                        state.selected_index = None;
                        state.error = None;
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to delete vault: {}", e));
                    }
                }
            }
            state.pending_delete_index = None;
            state.pin_verification.reset();
        } else if !state.pin_verification.is_visible() {
            // Show PIN verification modal
            state.pin_verification.show();
        }
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(Spacing::XL);

            ui.label(
                Typography::heading("Select Vault")
                    .color(Colors::text_primary(ctx))
            );
            ui.add_space(Spacing::LG);

            // Show error if any
            if let Some(ref error) = state.error {
                card(ui, |ui| {
                    ui.label(
                        Typography::body(error)
                            .color(Colors::ERROR)
                    );
                });
                ui.add_space(Spacing::MD);
            }

        // Load vaults on first render or when refresh is needed
        if state.vaults.is_empty() && !state.loading {
            state.loading = true;
            match AppState::list_vaults() {
                Ok(mut vaults) => {
                    // Filter out vaults with missing databases (orphaned metadata)
                    let initial_count = vaults.len();
                    vaults.retain(|v| {
                        let db_exists = std::path::Path::new(&v.database_path).exists();
                        if !db_exists {
                            eprintln!("Removing orphaned vault metadata: {} (database not found: {})", v.name, v.database_path);
                            // Optionally delete the orphaned metadata file
                            if let Err(e) = bitvault_common::wallet::VaultMetadata::delete(&v.address) {
                                eprintln!("Failed to delete orphaned metadata: {}", e);
                            }
                        }
                        db_exists
                    });
                    if initial_count != vaults.len() {
                        eprintln!("Removed {} orphaned vault metadata file(s)", initial_count - vaults.len());
                    }
                    state.vaults = vaults;
                    state.loading = false;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to load vaults: {}", e));
                    state.loading = false;
                }
            }
        }

            // Refresh button - centered
            ui.vertical_centered(|ui| {
                if button(ui, "🔄 Refresh", ButtonStyle::Secondary).clicked() {
                    state.loading = true;
                    state.error = None;
                    match AppState::list_vaults() {
                        Ok(mut vaults) => {
                            // Filter out vaults with missing databases (orphaned metadata)
                            let initial_count = vaults.len();
                            vaults.retain(|v| {
                                let db_exists = std::path::Path::new(&v.database_path).exists();
                                if !db_exists {
                                    eprintln!("Removing orphaned vault metadata: {} (database not found: {})", v.name, v.database_path);
                                    if let Err(e) = bitvault_common::wallet::VaultMetadata::delete(&v.address) {
                                        eprintln!("Failed to delete orphaned metadata: {}", e);
                                    }
                                }
                                db_exists
                            });
                            if initial_count != vaults.len() {
                                eprintln!("Removed {} orphaned vault metadata file(s)", initial_count - vaults.len());
                            }
                            state.vaults = vaults;
                            state.loading = false;
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to load vaults: {}", e));
                            state.loading = false;
                        }
                    }
                }
            });

            ui.add_space(Spacing::MD);

            if state.loading {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.add_space(Spacing::MD);
                    ui.label(
                        Typography::body("Loading vaults...")
                            .color(Colors::text_secondary(ctx))
                    );
                });
                return;
            }

            // List of vaults
            if state.vaults.is_empty() {
                // Empty state - properly centered
                ui.add_space(Spacing::XXL);
                card(ui, |ui| {
                    // Use vertical layout and center content
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        // Get available height and center content vertically
                        let available_height = ui.available_height();
                        let min_content_height = 150.0; // Minimum space for content

                        // Add top spacing to center vertically
                        if available_height > min_content_height {
                            ui.add_space((available_height - min_content_height) / 2.0);
                        }

                        ui.label(
                            Typography::heading_small("No vaults found")
                                .color(Colors::text_primary(ctx))
                        );
                        ui.add_space(Spacing::MD);
                        ui.label(
                            Typography::body("Create a new vault to get started")
                                .color(Colors::text_secondary(ctx))
                        );
                        ui.add_space(Spacing::XXL); // More space before button
                        if button_large(ui, "Create New Vault").clicked() {
                            navigation.navigate_to(crate::state::View::VaultCreation);
                        }
                    });
                });
                ui.add_space(Spacing::XXL);
            } else {
                // Show vault count summary
                let valid_count = state.vaults.iter().filter(|v| v.validate().is_ok()).count();
                let total_count = state.vaults.len();
                if valid_count == total_count {
                    ui.label(
                        Typography::caption(format!("✓ {} vault(s) available", valid_count))
                            .color(Colors::SUCCESS)
                    );
                } else {
                    ui.label(
                        Typography::caption(format!("⚠ {} of {} vault(s) valid", valid_count, total_count))
                            .color(Colors::WARNING)
                    );
                }
                ui.add_space(Spacing::XL); // More space before vault cards

                // Vault cards
                for (index, vault) in state.vaults.iter().enumerate() {
                    // Add spacing BEFORE each card (except first)
                    if index > 0 {
                        ui.add_space(Spacing::MD);
                    }

                    let is_selected = state.selected_index == Some(index);
                    let validation_result = vault.validate();
                    let is_valid = validation_result.is_ok();

                    // Card with hover and selection effects
                    let _card_response = card(ui, |ui| {
                        let response = ui.interact(
                            ui.available_rect_before_wrap(),
                            ui.id().with(index),
                            egui::Sense::click(),
                        );

                        // Selection border
                        if is_selected {
                            ui.painter().rect_stroke(
                                response.rect,
                                12.0,
                                egui::Stroke::new(2.0, Colors::PRIMARY),
                            );
                        }

                        // Hover effect
                        if response.hovered() && !is_selected {
                            ui.painter().rect_filled(
                                response.rect,
                                12.0,
                                if ctx.style().visuals.dark_mode {
                                    Colors::GRAY_700
                                } else {
                                    Colors::GRAY_100
                                },
                            );
                        }

                        ui.vertical(|ui| {
                            ui.add_space(Spacing::MD);

                            // Header row
                            ui.horizontal(|ui| {
                                // Vault name
                                ui.label(
                                    Typography::heading_small(&vault.name)
                                        .color(Colors::text_primary(ctx))
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Status badge
                                    let status_badge = if is_valid {
                                        BadgeStyle::Success
                                    } else if vault.database_exists() {
                                        BadgeStyle::Warning
                                    } else {
                                        BadgeStyle::Error
                                    };
                                    let status_text = if is_valid {
                                        "Valid"
                                    } else if vault.database_exists() {
                                        "Warning"
                                    } else {
                                        "Error"
                                    };
                                    badge(ui, status_text, status_badge);

                                    ui.add_space(Spacing::SM);

                                    // Network badge
                                    let network_badge = match vault.network.as_str() {
                                        "mainnet" => BadgeStyle::Success,
                                        "testnet" => BadgeStyle::Warning,
                                        "signet" => BadgeStyle::Info,
                                        _ => BadgeStyle::Neutral,
                                    };
                                    badge(ui, &vault.network, network_badge);
                                });
                            });

                            ui.add_space(Spacing::SM);

                            // Address (truncated)
                            let address_display = if vault.address.len() > 30 {
                                format!("{}...", &vault.address[0..30])
                            } else {
                                vault.address.clone()
                            };
                            ui.label(
                                Typography::body(address_display)
                                    .color(Colors::text_secondary(ctx))
                                    .monospace()
                            );

                            // Created date
                            if let Some(ref created) = vault.created_at.get(0..10) {
                                ui.add_space(Spacing::XS);
                                ui.label(
                                    Typography::caption(format!("Created: {}", created))
                                        .color(Colors::text_muted(ctx))
                                );
                            }

                            // Show validation error if invalid
                            if !is_valid {
                                ui.add_space(Spacing::SM);
                                match &validation_result {
                                    Ok(_) => {}
                                    Err(e) => {
                                        ui.label(
                                            Typography::caption(format!("⚠ {}", e))
                                                .color(Colors::ERROR)
                                        );
                                    }
                                }
                            }

                            ui.add_space(Spacing::MD);
                        });

                        if response.clicked() {
                            state.selected_index = Some(index);
                        }
                    });

                    // Add spacing AFTER each card
                    if index < state.vaults.len() - 1 {
                        ui.add_space(Spacing::MD); // Between cards
                    } else {
                        // After LAST card - add large spacing
                        ui.add_space(100.0);
                        ui.separator();
                        ui.add_space(20.0);
                    }
                }

                // Primary action buttons - only show when there are vaults
                if !state.vaults.is_empty() {
                    ui.horizontal_centered(|ui| {
                        if button_large(ui, "Load Selected Vault").clicked() {
                            if let Some(index) = state.selected_index {
                                if index < state.vaults.len() {
                                    let metadata = state.vaults[index].clone();

                                    // Validate vault before loading
                                    match metadata.validate() {
                                        Ok(_) => {
                                            // Unload current vault if one is loaded
                                            if app_state.is_vault_loaded() {
                                                app_state.unload_vault();
                                            }

                                            // Load vault using runtime
                                            if let Some(ref runtime) = app_state.runtime {
                                                let handle = runtime.handle().clone();
                                                match handle.block_on(app_state.load_vault_from_metadata(&metadata)) {
                                                    Ok(_) => {
                                                        // Navigate to dashboard
                                                        navigation.navigate_to(crate::state::View::Dashboard { tab: 0 });
                                                        // Vault data will be refreshed automatically by async handler
                                                    }
                                                    Err(e) => {
                                                        state.error = Some(format!("Failed to load vault: {}", e));
                                                    }
                                                }
                                            } else {
                                                state.error = Some("Runtime not available".to_string());
                                            }
                                        }
                                        Err(e) => {
                                            state.error = Some(format!("Cannot load vault: {}", e));
                                        }
                                    }
                                }
                            } else {
                                state.error = Some("Please select a vault".to_string());
                            }
                            }
                            ui.add_space(Spacing::MD);
                            if button_large(ui, "Create New Vault").clicked() {
                                navigation.navigate_to(crate::state::View::VaultCreation);
                            }
                            ui.add_space(Spacing::MD);
                            if button_large(ui, "Import Vault").clicked() {
                                state.import_dialog_open = true;
                                state.import_text.clear();
                            }
                        });
                }
            }

            ui.add_space(Spacing::MD);

            // Vault management buttons (only if a vault is selected)
            if let Some(index) = state.selected_index {
                if index < state.vaults.len() {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            if button(ui, "Rename", ButtonStyle::Text).clicked() {
                                state.rename_dialog = Some(index);
                                state.rename_text = state.vaults[index].name.clone();
                            }
                            ui.add_space(Spacing::SM);
                            if button(ui, "Export", ButtonStyle::Text).clicked() {
                                let metadata = &state.vaults[index];
                                match serde_json::to_string_pretty(metadata) {
                                    Ok(json) => {
                                        // Copy to clipboard
                                        ui.output_mut(|o| {
                                            o.copied_text = json.clone();
                                        });
                                        state.error = None;
                                    }
                                    Err(e) => {
                                        state.error = Some(format!("Failed to export vault: {}", e));
                                    }
                                }
                            }
                            ui.add_space(Spacing::SM);
                            if button(ui, "Delete", ButtonStyle::Danger).clicked() {
                                // Check if PIN is required
                                let pin_service = bitvault_common::PinService::new();
                                if pin_service.has_pin() {
                                    // Require PIN verification before deletion
                                    state.pending_delete_index = Some(index);
                                    state.pin_verification.show();
                                } else {
                                    // No PIN set - delete directly
                                    let vault_address = state.vaults[index].address.clone();

                                    // Delete vault completely (metadata and database)
                                    match bitvault_common::wallet::VaultMetadata::delete_vault_complete(&vault_address) {
                                        Ok(_) => {
                                            // Remove from list
                                            state.vaults.remove(index);
                                            state.selected_index = None;
                                            state.error = None;
                                        }
                                        Err(e) => {
                                            state.error = Some(format!("Failed to delete vault: {}", e));
                                        }
                                    }
                                }
                            }
                        });
                    });
                }
            } // closes else block for !vaults.is_empty()
        }); // closes vertical_centered

        // Dialogs (outside ScrollArea since they're windows)
        // Rename dialog
        if let Some(index) = state.rename_dialog {
                if index < state.vaults.len() {
                    egui::Window::new("Rename Vault")
                        .collapsible(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.label("Enter new vault name:");
                            ui.text_edit_singleline(&mut state.rename_text);

                            ui.add_space(10.0);

                            // Buttons - centered
                            let button_width = 100.0;
                            let (rect, _) = ui.allocate_exact_size(
                                egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
                                egui::Sense::click()
                            );
                            let mut button_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                            if button_ui.button("Cancel").clicked() {
                                state.rename_dialog = None;
                                state.rename_text.clear();
                            }
                            button_ui.add_space(10.0);
                            if button_ui.button("Save").clicked() {
                                    let mut metadata = state.vaults[index].clone();
                                    match metadata.update_name(state.rename_text.clone()) {
                                        Ok(_) => {
                                            state.vaults[index] = metadata;
                                            state.rename_dialog = None;
                                            state.rename_text.clear();
                                            state.error = None;
                                        }
                                        Err(e) => {
                                            state.error = Some(format!("Failed to rename vault: {}", e));
                                        }
                                    }
                                }
                            });
                }
            }

            // Import dialog
            if state.import_dialog_open {
                egui::Window::new("Import Vault")
                    .collapsible(false)
                    .resizable(true)
                    .default_size([400.0, 300.0])
                    .show(ctx, |ui| {
                        ui.label("Paste vault metadata JSON:");
                        ui.add_space(5.0);

                        // Multi-line text input for JSON
                        ui.text_edit_multiline(&mut state.import_text);

                        ui.add_space(10.0);

                        ui.label("Note: The vault database must be manually copied to the correct location.");
                        ui.label("Database path will be shown after import.");

                        ui.add_space(10.0);

                        // Buttons - centered
                        let button_width = 100.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
                            egui::Sense::click()
                        );
                        let mut button_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                        if button_ui.button("Cancel").clicked() {
                            state.import_dialog_open = false;
                            state.import_text.clear();
                        }
                        button_ui.add_space(10.0);
                        if button_ui.button("Import").clicked() {
                                match serde_json::from_str::<bitvault_common::wallet::VaultMetadata>(&state.import_text) {
                                    Ok(metadata) => {
                                        // Validate required fields
                                        if metadata.address.is_empty() {
                                            state.error = Some("Vault metadata missing address".to_string());
                                        } else if metadata.database_path.is_empty() {
                                            state.error = Some("Vault metadata missing database path".to_string());
                                        } else {
                                            // Check if vault already exists
                                            if state.vaults.iter().any(|v| v.address == metadata.address) {
                                                state.error = Some("Vault with this address already exists".to_string());
                                            } else {
                                                // Save metadata
                                                match metadata.save() {
                                                    Ok(_) => {
                                                        // Add to list
                                                        state.vaults.push(metadata.clone());
                                                        state.import_dialog_open = false;
                                                        state.import_text.clear();
                                                        state.error = None;
                                                    }
                                                    Err(e) => {
                                                        state.error = Some(format!("Failed to save vault metadata: {}", e));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        state.error = Some(format!("Invalid JSON: {}", e));
                                    }
                                }
                            }
                        });
        }
    }); // closes ScrollArea
}
