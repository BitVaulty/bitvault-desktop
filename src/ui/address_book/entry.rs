//! Address Entry Dialog
//!
//! Dialog for adding a new address to the address book

use crate::services::address_book::AddressBookService;
use bdk::bitcoin::Address;
use eframe::egui;
use std::str::FromStr;

/// State for address entry dialog
#[derive(Default)]
pub struct AddressEntryState {
    address: String,
    label: String,
    error: Option<String>,
}

impl AddressEntryState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_address(address: String) -> Self {
        Self {
            address,
            label: String::new(),
            error: None,
        }
    }
}

/// Render address entry dialog
/// Returns true if address was saved
pub fn render_address_entry(
    ctx: &egui::Context,
    state: &mut AddressEntryState,
    vault_address: &str,
    _network: bdk::bitcoin::Network,
    on_save: impl FnOnce(String, Option<String>),
) -> bool {
    let mut should_save = false;
    let mut address = state.address.clone();
    let mut label = state.label.clone();

    egui::Window::new("Add Address")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Bitcoin Address:");
            ui.text_edit_singleline(&mut address);

            ui.add_space(10.0);

            ui.label("Label (optional):");
            ui.text_edit_singleline(&mut label);

            if let Some(ref error) = state.error {
                ui.add_space(10.0);
                ui.colored_label(egui::Color32::RED, error);
            }

            ui.add_space(10.0);

            // Buttons - centered
            let button_width = 100.0;
            let (rect, _) = ui.allocate_exact_size(
                egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
                egui::Sense::click(),
            );
            let mut button_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
            if button_ui.button("Cancel").clicked() {
                // Dialog will close automatically
            }
            button_ui.add_space(10.0);
            if button_ui.button("Save").clicked() {
                let address_trimmed = address.trim().to_string();
                if address_trimmed.is_empty() {
                    state.error = Some("Address cannot be empty".to_string());
                } else {
                    // Validate address using proper Bitcoin address parsing
                    match Address::from_str(&address_trimmed) {
                        Ok(_addr) => {
                            // Address is valid - proceed with saving
                            let label_opt = if label.trim().is_empty() {
                                None
                            } else {
                                Some(label.trim().to_string())
                            };

                            let service = AddressBookService::new().unwrap_or_default();
                            if let Err(e) = service.add_address(
                                vault_address,
                                address_trimmed.clone(),
                                label_opt.clone(),
                            ) {
                                state.error = Some(format!("Failed to save: {}", crate::utils::sanitize_error_for_ui(&e)));
                            } else {
                                state.address = address_trimmed.clone();
                                state.label = label.clone();
                                should_save = true;
                                on_save(address_trimmed, label_opt);
                            }
                        }
                        Err(e) => {
                            state.error = Some(format!("Invalid Bitcoin address: {}", crate::utils::sanitize_error_for_ui(&e)));
                        }
                    }
                }
            }
        });

    should_save
}
