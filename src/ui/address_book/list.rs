//! Address Book List View
//!
//! Displays a list of saved addresses with options to edit, delete, and use them

use crate::services::address_book::{AddressBookService, AddressEntry};
use crate::state::{AppState, Navigation};
use eframe::egui;

/// State for address book list view
pub struct AddressBookState {
    addresses: Vec<AddressEntry>,
    is_loading: bool,
    error: Option<String>,
    search_text: String,
    selected_index: Option<usize>,
    edit_dialog: Option<usize>, // Index of address being edited
    edit_label: String,
    add_dialog_open: bool,
    add_address_state: crate::ui::address_book::entry::AddressEntryState,
}

impl Default for AddressBookState {
    fn default() -> Self {
        Self {
            addresses: Vec::new(),
            is_loading: false,
            error: None,
            search_text: String::new(),
            selected_index: None,
            edit_dialog: None,
            edit_label: String::new(),
            add_dialog_open: false,
            add_address_state: crate::ui::address_book::entry::AddressEntryState::new(),
        }
    }
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
                self.error = Some(format!("Failed to load addresses: {}", crate::utils::sanitize_error_for_ui(&e)));
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
                entry.address.to_lowercase().contains(&search_lower)
                    || entry
                        .label
                        .as_ref()
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
    let vault_address = app_state
        .vault_data
        .lock()
        .ok()
        .and_then(|data| data.receive_address.clone());

    let vault_address = match vault_address {
        Some(addr) => addr,
        None => {
            ui.label("No vault loaded");
            return;
        }
    };

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
                                ui.label(format!(
                                    "Last used: {}",
                                    entry.last_used.format("%Y-%m-%d %H:%M")
                                ));
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Use").clicked() {
                                        // Navigate to send transaction with this address pre-filled
                                        navigation.navigate_to_with_data(
                                            crate::state::View::SendTransaction,
                                            Some(entry.address.clone()),
                                        );
                                    }

                                    if ui.button("Edit").clicked() {
                                        state.edit_dialog = Some(idx);
                                        state.edit_label = entry.label.clone().unwrap_or_default();
                                    }

                                    if ui.button("Delete").clicked() {
                                        let service = AddressBookService::new().unwrap_or_default();
                                        if let Err(e) =
                                            service.delete_address(&vault_address, &entry.address)
                                        {
                                            state.error = Some(format!("Failed to delete: {}", crate::utils::sanitize_error_for_ui(&e)));
                                        } else {
                                            state.refresh(&vault_address);
                                        }
                                    }
                                },
                            );
                        });

                        ui.separator();
                    }
                });
        }

        ui.add_space(10.0);

        // Action buttons - centered
        let button_width = 120.0;
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
            egui::Sense::click(),
        );
        let mut button_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        if button_ui.button("Refresh").clicked() {
            state.refresh(&vault_address);
        }
        button_ui.add_space(10.0);
        if button_ui.button("Add Address").clicked() {
            state.add_dialog_open = true;
            state.add_address_state = crate::ui::address_book::entry::AddressEntryState::new();
        }
    });

    // Add address dialog
    if state.add_dialog_open {
        use crate::ui::address_book::entry::render_address_entry;
        let mut should_refresh = false;
        let saved = render_address_entry(
            ctx,
            &mut state.add_address_state,
            &vault_address,
            app_state.network,
            |_address, _label| {
                // Address saved - mark for refresh
                should_refresh = true;
            },
        );
        if saved || should_refresh {
            state.refresh(&vault_address);
            state.add_dialog_open = false;
            state.add_address_state = crate::ui::address_book::entry::AddressEntryState::new();
        }
    }

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

                            if let Err(e) =
                                service.update_label(&vault_address, &entry_address, label)
                            {
                                state.error = Some(format!("Failed to update: {}", crate::utils::sanitize_error_for_ui(&e)));
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
