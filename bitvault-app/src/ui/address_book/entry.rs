//! Address Entry Dialog
//!
//! Dialog for adding a new address to the address book

use eframe::egui;
use crate::services::address_book::AddressBookService;

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
            
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    // Dialog will close automatically
                }
                
                if ui.button("Save").clicked() {
                    let address_trimmed = address.trim().to_string();
                    if address_trimmed.is_empty() {
                        state.error = Some("Address cannot be empty".to_string());
                    } else {
                        // Basic address validation (starts with bc1, 1, or 3)
                        if !address_trimmed.starts_with("bc1") && 
                           !address_trimmed.starts_with("1") && 
                           !address_trimmed.starts_with("3") &&
                           !address_trimmed.starts_with("tb1") &&
                           !address_trimmed.starts_with("bcrt1") {
                            state.error = Some("Invalid Bitcoin address format".to_string());
                        } else {
                            let label_opt = if label.trim().is_empty() {
                                None
                            } else {
                                Some(label.trim().to_string())
                            };
                            
                            let service = AddressBookService::new().unwrap_or_default();
                            if let Err(e) = service.add_address(vault_address, address_trimmed.clone(), label_opt.clone()) {
                                state.error = Some(format!("Failed to save: {}", e));
                            } else {
                                state.address = address_trimmed.clone();
                                state.label = label.clone();
                                should_save = true;
                                on_save(address_trimmed, label_opt);
                            }
                        }
                    }
                }
            });
        });
    
    should_save
}
