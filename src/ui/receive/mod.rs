//! Receive Address/QR Display
//!
//! Shows:
//! - QR code for receive address
//! - Address text
//! - Copy and share buttons

use crate::state::{AppState, Navigation};
use crate::ui::components::{
    badge, button, button_large, card, BadgeStyle, ButtonStyle, Colors, Spacing, Typography,
};
use crate::utils::qr::generate_qr_image;
use eframe::egui;

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

pub fn render(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    ctx: &egui::Context,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(Spacing::XL);

            if !app_state.is_vault_loaded() {
                card(ui, |ui| {
                    ui.label(
                        Typography::body("No vault loaded").color(Colors::text_secondary(ctx)),
                    );
                });
                ui.add_space(Spacing::MD);
                if button(ui, "Back", ButtonStyle::Secondary).clicked() {
                    navigation.go_back();
                }
                return;
            }

            ui.label(Typography::heading("Receive Bitcoin").color(Colors::text_primary(ctx)));
            ui.add_space(Spacing::SM);
            ui.label(
                Typography::body("Scan the QR code or copy the address to receive Bitcoin")
                    .color(Colors::text_secondary(ctx)),
            );
            ui.add_space(Spacing::LG);

            RECEIVE_STATE.with(|state| {
                let mut state = state.borrow_mut();

                // Load address if not loaded
                if state.address.is_none() && !state.is_loading {
                    load_address(ui, app_state, &mut state);
                }

                // Show error if any
                if let Some(ref error) = state.error {
                    card(ui, |ui| {
                        ui.label(Typography::body(error).color(Colors::ERROR));
                    });
                    ui.add_space(Spacing::MD);
                }

                // Show loading indicator
                if state.is_loading {
                    ui.vertical_centered(|ui| {
                        ui.spinner();
                        ui.add_space(Spacing::MD);
                        ui.label(
                            Typography::body("Loading address...")
                                .color(Colors::text_secondary(ctx)),
                        );
                    });
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

                    // QR code card
                    card(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(Spacing::MD);

                            // Display QR code (re-read after potential update)
                            let qr_texture_opt = state.qr_image.clone();
                            if let Some(ref qr_texture) = qr_texture_opt {
                                ui.image((qr_texture.id(), egui::Vec2::new(300.0, 300.0)));
                            }

                            ui.add_space(Spacing::MD);
                        });
                    });

                    ui.add_space(Spacing::LG);

                    // Address card
                    card(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add_space(Spacing::MD);

                            ui.label(
                                Typography::body("BTC Deposit Address")
                                    .color(Colors::text_secondary(ctx)),
                            );
                            ui.add_space(Spacing::SM);

                            // Address text (selectable, monospace)
                            ui.label(
                                Typography::body(address)
                                    .color(Colors::text_primary(ctx))
                                    .monospace(),
                            );

                            ui.add_space(Spacing::MD);

                            // Copy button
                            if copied {
                                ui.vertical_centered(|ui| {
                                    badge(ui, "✓ Copied to clipboard!", BadgeStyle::Success);
                                });
                            } else {
                                ui.vertical_centered(|ui| {
                                    if button_large(ui, "Copy Address").clicked() {
                                        copy_to_clipboard(ui, address);
                                        state.copied = true;
                                    }
                                });
                            }

                            ui.add_space(Spacing::MD);
                        });
                    });

                    ui.add_space(Spacing::LG);
                } else {
                    card(ui, |ui| {
                        ui.label(Typography::body("Failed to load address").color(Colors::ERROR));
                    });
                }

                // Back button - centered
                ui.add_space(Spacing::MD);
                if button(ui, "Back", ButtonStyle::Secondary).clicked() {
                    navigation.go_back();
                }

                ui.add_space(Spacing::XL);
            });
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
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
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
                state.error = Some(format!("Failed to load address: {}", crate::utils::sanitize_error_for_ui(&e)));
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
