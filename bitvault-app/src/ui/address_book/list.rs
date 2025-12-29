//! Address Book List View
//!
//! Displays a list of saved addresses with options to edit, delete, and use them

use eframe::egui;
use crate::state::{AppState, Navigation};
use crate::services::address_book::{AddressBookService, AddressEntry};

/// State for address book list view
#[derive(Default)]
pub struct AddressBookState {
    addresses: Vec<AddressEntry>,
    is_loading: bool,
    error: Option<String>,
    search_text: String,
    selected_index: Option<usize>,
    edit_dialog: Option<usize>, // Index of address being edited
    edit_label: String,
}


impl AddressBookState {
    /// Refresh the address list
    pub fn refresh(&mut self, vault_address: &str) {
        self.is_loading = true;
        self.error = None;
        
        let service = AddressBookService::new().unwrap_or_default();
        match service.get_addresses(vault_address) {
            Ok(addresses) => {
                self.addresses = addresses;
                self.is_loading = false;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load addresses: {}", e));
                self.is_loading = false;
            }
        }
    }
    
    /// Get filtered addresses based on search text
    fn filtered_addresses(&self) -> Vec<&AddressEntry> {
        if self.search_text.is_empty() {
            return self.addresses.iter().collect();
        }
        
        let search_lower = self.search_text.to_lowercase();
        self.addresses
            .iter()
            .filter(|entry| {
                entry.address.to_lowercase().contains(&search_lower) ||
                entry.label.as_ref()
                    .map(|l| l.to_lowercase().contains(&search_lower))
                    .unwrap_or(false)
            })
            .collect()
    }
}

/// Render address book list view
pub fn render_address_book(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut AddressBookState,
    ctx: &egui::Context,
) {
    // Get current vault address
    let vault_address = app_state.vault_data.lock()
        .ok()
        .and_then(|data| data.receive_address.clone());
    
    if vault_address.is_none() {
        ui.label("No vault loaded");
        return;
    }
    let vault_address = vault_address.unwrap();
    
    // Refresh on first render
    if state.addresses.is_empty() && !state.is_loading {
        state.refresh(&vault_address);
    }
    
    ui.vertical(|ui| {
        ui.heading("Address Book");
        ui.add_space(10.0);
        
        // Search bar
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut state.search_text);
            if ui.button("Clear").clicked() {
                state.search_text.clear();
            }
        });
        
        ui.add_space(10.0);
        
        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }
        
        // Show loading state
        if state.is_loading {
            ui.label("Loading addresses...");
            return;
        }
        
        // Address list
        let filtered = state.filtered_addresses();
        
        if filtered.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("No saved addresses");
                ui.label("Addresses you send to will be saved here");
                ui.add_space(20.0);
            });
        } else {
            // Clone filtered entries to avoid borrow issues
            let filtered_entries: Vec<AddressEntry> = filtered.into_iter().cloned().collect();
            
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for (idx, entry) in filtered_entries.iter().enumerate() {
                        let is_selected = state.selected_index == Some(idx);
                        
                        // Display address entry
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_selected, "").clicked() {
                                state.selected_index = Some(idx);
                            }
                            
                            ui.vertical(|ui| {
                                // Label or address
                                if let Some(ref label) = entry.label {
                                    ui.label(egui::RichText::new(label).strong());
                                    ui.label(entry.address.to_string());
                                } else {
                                    ui.label(entry.address.clone());
                                }
                                
                                // Last used timestamp
                                ui.label(format!("Last used: {}", entry.last_used.format("%Y-%m-%d %H:%M")));
                            });
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Use").clicked() {
                                    // Navigate to send transaction with this address
                                    navigation.navigate_to(crate::state::View::SendTransaction);
                                    // TODO: Pre-fill address in send transaction
                                }
                                
                                if ui.button("Edit").clicked() {
                                    state.edit_dialog = Some(idx);
                                    state.edit_label = entry.label.clone().unwrap_or_default();
                                }
                                
                                if ui.button("Delete").clicked() {
                                    let service = AddressBookService::new().unwrap_or_default();
                                    if let Err(e) = service.delete_address(&vault_address, &entry.address) {
                                        state.error = Some(format!("Failed to delete: {}", e));
                                    } else {
                                        state.refresh(&vault_address);
                                    }
                                }
                            });
                        });
                        
                        ui.separator();
                    }
                });
        }
        
        ui.add_space(10.0);
        
        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                state.refresh(&vault_address);
            }
            
            if ui.button("Add Address").clicked() {
                // TODO: Show add address dialog
            }
        });
    });
    
    // Edit dialog
    if let Some(idx) = state.edit_dialog {
        // Get filtered addresses and clone the entry
        let filtered = state.filtered_addresses();
        if idx < filtered.len() {
            let entry_address = filtered[idx].address.clone();
            egui::Window::new("Edit Address")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Address: {}", entry_address));
                    ui.add_space(10.0);
                    
                    ui.label("Label:");
                    ui.text_edit_singleline(&mut state.edit_label);
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            state.edit_dialog = None;
                            state.edit_label.clear();
                        }
                        
                        if ui.button("Save").clicked() {
                            let service = AddressBookService::new().unwrap_or_default();
                            let label = if state.edit_label.trim().is_empty() {
                                None
                            } else {
                                Some(state.edit_label.trim().to_string())
                            };
                            
                            if let Err(e) = service.update_label(&vault_address, &entry_address, label) {
                                state.error = Some(format!("Failed to update: {}", e));
                            } else {
                                state.refresh(&vault_address);
                                state.edit_dialog = None;
                                state.edit_label.clear();
                            }
                        }
                    });
                });
        }
    }
}
