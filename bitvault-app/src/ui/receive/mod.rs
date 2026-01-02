//! Receive Address/QR Display
//!
//! Shows:
//! - QR code for receive address
//! - Address text
//! - Copy and share buttons

use eframe::egui;
use crate::state::{AppState, Navigation};
use crate::utils::qr::generate_qr_image;

/// Receive view state
#[derive(Default)]
struct ReceiveState {
    address: Option<String>,
    qr_image: Option<egui::TextureHandle>,
    is_loading: bool,
    error: Option<String>,
    copied: bool,
}


// Thread-local state for receive view
thread_local! {
    static RECEIVE_STATE: std::cell::RefCell<ReceiveState> = 
        std::cell::RefCell::new(ReceiveState::default());
}

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation, ctx: &egui::Context) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);

        if !app_state.is_vault_loaded() {
            ui.label("No vault loaded");
            if ui.button("Back").clicked() {
                navigation.go_back();
            }
            return;
        }

        ui.heading("Receive Bitcoin");
        ui.add_space(10.0);
        ui.label("Scan the QR code or copy the address to receive Bitcoin.");
        ui.add_space(20.0);

        RECEIVE_STATE.with(|state| {
            let mut state = state.borrow_mut();

            // Load address if not loaded
            if state.address.is_none() && !state.is_loading {
                load_address(ui, app_state, &mut state);
            }

            // Show error if any
            if let Some(ref error) = state.error {
                ui.colored_label(egui::Color32::RED, error);
                ui.add_space(10.0);
            }

            // Show loading indicator
            if state.is_loading {
                ui.label("Loading address...");
                return;
            }

            // Show address and QR code
            let address_opt = state.address.clone();
            let qr_image_opt = state.qr_image.clone();
            let copied = state.copied;
            
            if let Some(ref address) = address_opt {
                // Generate QR code if not already generated
                if qr_image_opt.is_none() {
                    if let Some(texture) = generate_qr_image(ctx, address) {
                        state.qr_image = Some(texture);
                    }
                }

                // Display QR code (re-read after potential update)
                let qr_texture_opt = state.qr_image.clone();
                if let Some(ref qr_texture) = qr_texture_opt {
                    ui.add_space(10.0);
                    ui.image((qr_texture.id(), egui::Vec2::new(300.0, 300.0)));
                    ui.add_space(20.0);
                }

                // Address section
                ui.label("BTC deposit address");
                ui.add_space(5.0);
                
                // Address text (selectable)
                ui.horizontal(|ui| {
                    ui.label(address);
                });
                
                ui.add_space(10.0);

                // Copy button
                if copied {
                    ui.label("✓ Copied to clipboard!");
                } else if ui.button("Copy Address").clicked() {
                    copy_to_clipboard(ui, address);
                    state.copied = true;
                }

                ui.add_space(10.0);
            } else {
                ui.label("Failed to load address");
            }

            // Back button
            ui.add_space(20.0);
            if ui.button("Back").clicked() {
                navigation.go_back();
            }
        });
    });
}

fn load_address(_ui: &mut egui::Ui, app_state: &mut AppState, state: &mut ReceiveState) {
    state.is_loading = true;
    state.error = None;

    // Try to get address from cached vault data first
    let vault_data = match app_state.vault_data.lock() {
        Ok(data) => data,
        Err(_) => {
            state.error = Some("Error: Mutex poisoned".to_string());
            state.is_loading = false;
            return;
        }
    };
    if let Some(ref address) = vault_data.receive_address {
        state.address = Some(address.clone());
        state.is_loading = false;
        return;
    }
    drop(vault_data);

    // If not cached, fetch from vault service
    if let (Some(vault_service), Some(runtime)) = 
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref()) {
        
        let result = runtime.block_on(async {
            let vs = vault_service.read().await;
            vs.get_new_address().await
        });

        match result {
            Ok(address) => {
                state.address = Some(address);
                state.is_loading = false;
                
                // Update cached vault data
                if let Ok(mut vault_data) = app_state.vault_data.lock() {
                    if let Some(ref addr) = state.address {
                        vault_data.update_address(addr.clone());
                    }
                }
            }
            Err(e) => {
                state.error = Some(format!("Failed to load address: {}", e));
                state.is_loading = false;
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.is_loading = false;
    }
}

fn copy_to_clipboard(ui: &mut egui::Ui, text: &str) {
    ui.output_mut(|o| {
        o.copied_text = text.to_string();
    });
}

