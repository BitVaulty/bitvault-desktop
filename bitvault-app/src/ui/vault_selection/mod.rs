//! Vault selection UI
//!
//! Displays a list of available vaults and allows the user to:
//! - Select an existing vault to load
//! - Create a new vault
//! - Import a vault

use eframe::egui;
use crate::state::{AppState, Navigation};
use crate::ui::pin::render_pin_verification;
use serde_json;

/// Vault selection state
pub struct VaultSelectionState {
    pub vaults: Vec<bitvault_common::wallet::VaultMetadata>,
    pub loading: bool,
    pub error: Option<String>,
    pub selected_index: Option<usize>,
    pub rename_dialog: Option<usize>, // Index of vault being renamed, None if dialog closed
    pub rename_text: String, // Text for rename input
    pub import_dialog_open: bool, // Whether import dialog is open
    pub import_text: String, // Text for import (JSON metadata)
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
pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation, state: &mut VaultSelectionState, ctx: &egui::Context) {
    // Handle PIN verification for delete operation
    if let Some(index) = state.pending_delete_index {
        if render_pin_verification(ctx, &mut state.pin_verification) {
            // PIN verified - proceed with deletion
            if index < state.vaults.len() {
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
            state.pending_delete_index = None;
            state.pin_verification.reset();
        } else if !state.pin_verification.is_visible() {
            // Show PIN verification modal
            state.pin_verification.show();
        }
    }
    
    ui.vertical_centered(|ui| {
        ui.heading("Select Vault");
        ui.add_space(20.0);
        
        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }
        
        // Load vaults on first render or when refresh is needed
        if state.vaults.is_empty() && !state.loading {
            state.loading = true;
            match AppState::list_vaults() {
                Ok(vaults) => {
                    state.vaults = vaults;
                    state.loading = false;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to load vaults: {}", e));
                    state.loading = false;
                }
            }
        }
        
        // Refresh button and validation info
        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh").clicked() {
                state.loading = true;
                state.error = None;
                match AppState::list_vaults() {
                    Ok(vaults) => {
                        state.vaults = vaults;
                        state.loading = false;
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to load vaults: {}", e));
                        state.loading = false;
                    }
                }
            }
            
            // Show vault count and validation summary
            if !state.vaults.is_empty() {
                let valid_count = state.vaults.iter().filter(|v| v.validate().is_ok()).count();
                let total_count = state.vaults.len();
                if valid_count == total_count {
                    ui.label(format!("✓ {} vault(s) valid", valid_count));
                } else {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("⚠ {} of {} vault(s) valid", valid_count, total_count)
                    );
                }
            }
        });
        
        ui.add_space(10.0);
        
        if state.loading {
            ui.label("Loading vaults...");
            return;
        }
        
        // List of vaults
        if state.vaults.is_empty() {
            ui.label("No vaults found.");
            ui.add_space(20.0);
            ui.label("Create a new vault to get started.");
            ui.add_space(20.0);
            
            if ui.button("Create New Vault").clicked() {
                navigation.navigate_to(crate::state::View::VaultCreation);
            }
        } else {
            ui.label("Select a vault to load:");
            ui.add_space(10.0);
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, vault) in state.vaults.iter().enumerate() {
                    let is_selected = state.selected_index == Some(index);
                    
                    // Validate vault
                    let validation_result = vault.validate();
                    let is_valid = validation_result.is_ok();
                    let status_icon = if is_valid {
                        "✓"
                    } else if vault.database_exists() {
                        "⚠" // Database exists but other issues
                    } else {
                        "✗" // Database missing
                    };
                    
                    // Make entire row clickable using a selectable label
                    let vault_text = format!(
                        "{} {} {} - {} ({})",
                        if is_selected { "✓" } else { " " },
                        status_icon,
                        vault.name,
                        vault.network,
                        &vault.address[0..std::cmp::min(20, vault.address.len())]
                    );
                    
                    if ui.selectable_label(is_selected, vault_text).clicked() {
                        state.selected_index = Some(index);
                    }
                    
                    // Show full details in a collapsible section
                    if is_selected {
                        ui.indent("vault_details", |ui| {
                            ui.label(format!("Full Address: {}", vault.address));
                            if let Some(ref created) = vault.created_at.get(0..10) {
                                ui.label(format!("Created: {}", created));
                            }
                            ui.label(format!("Database: {}", vault.database_path));
                            
                            // Show validation status
                            match validation_result {
                                Ok(_) => {
                                    ui.colored_label(egui::Color32::GREEN, "✓ Vault is valid");
                                }
                                Err(e) => {
                                    ui.colored_label(egui::Color32::RED, format!("✗ {}", e));
                                }
                            }
                        });
                    }
                    
                    ui.add_space(5.0);
                }
            });
            
            ui.add_space(20.0);
            
            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Load Selected Vault").clicked() {
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
                
                if ui.button("Create New Vault").clicked() {
                    navigation.navigate_to(crate::state::View::VaultCreation);
                }
                
                if ui.button("Import Vault").clicked() {
                    state.import_dialog_open = true;
                    state.import_text.clear();
                }
            });
            
            ui.add_space(10.0);
            
            // Vault management buttons (only if a vault is selected)
            if let Some(index) = state.selected_index {
                if index < state.vaults.len() {
                    ui.horizontal(|ui| {
                        if ui.button("Rename Vault").clicked() {
                            state.rename_dialog = Some(index);
                            state.rename_text = state.vaults[index].name.clone();
                        }
                        
                        if ui.button("Export Vault").clicked() {
                            let metadata = &state.vaults[index];
                            match serde_json::to_string_pretty(metadata) {
                                Ok(json) => {
                                    // Copy to clipboard
                                    ui.output_mut(|o| {
                                        o.copied_text = json.clone();
                                    });
                                    state.error = None;
                                    // Show success message (could use a toast notification in the future)
                                    ui.label("✓ Vault metadata copied to clipboard!");
                                }
                                Err(e) => {
                                    state.error = Some(format!("Failed to export vault: {}", e));
                                }
                            }
                        }
                        
                        if ui.button("Delete Vault").clicked() {
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
                }
            }
            
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
                            
                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    state.rename_dialog = None;
                                    state.rename_text.clear();
                                }
                                
                                if ui.button("Save").clicked() {
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
                        
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                state.import_dialog_open = false;
                                state.import_text.clear();
                            }
                            
                            if ui.button("Import").clicked() {
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
                    });
            }
        }
    });
}

