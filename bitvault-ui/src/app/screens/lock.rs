use crate::app::state::{View, WalletState};
use crate::app::BitVaultApp;
use eframe::egui::{self, Color32, Ui};

pub fn render(app: &BitVaultApp, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.heading("Unlock Your Wallet");
        ui.add_space(20.0);

        ui.label("Enter your PIN to unlock your wallet");
        ui.add_space(30.0);

        // Check if we need to load the wallet data from disk
        let wallet_loaded = if let Ok(mut state) = app.state.write() {
            if state.encrypted_wallet_data.is_none() {
                log::info!("Attempting to load wallet data from disk");
                match app.load_wallet_from_disk() {
                    Ok(encrypted_data) => {
                        state.encrypted_wallet_data = Some(encrypted_data);
                        log::info!("Wallet data loaded from disk successfully");
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to load wallet data: {}", e);
                        state.lock_error = Some(
                            "Failed to load wallet data. Please create a new wallet.".to_string(),
                        );
                        false
                    }
                }
            } else {
                true
            }
        } else {
            false
        };

        let mut pin_input = String::new();
        if let Ok(state) = app.state.read() {
            pin_input = state.pin_input.clone();
        }

        // PIN input field
        let pin_response = ui.add(
            egui::TextEdit::singleline(&mut pin_input)
                .password(true)
                .hint_text("Enter PIN")
                .desired_width(200.0),
        );

        if pin_response.changed() {
            if let Ok(mut state) = app.state.write() {
                state.pin_input = pin_input;
            }
        }

        // Check for Enter key press
        let enter_pressed =
            pin_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        // Display error message if there is one
        if let Ok(state) = app.state.read() {
            if let Some(error) = &state.lock_error {
                ui.add_space(10.0);
                ui.colored_label(Color32::RED, error);
            }
        }

        ui.add_space(20.0);

        let unlock_button = ui.add_enabled(wallet_loaded, egui::Button::new("Unlock"));
        if unlock_button.clicked() || (enter_pressed && wallet_loaded) {
            if let Ok(mut state) = app.state.write() {
                // Try to load and decrypt the wallet
                if let Some(encrypted_data) = &state.encrypted_wallet_data {
                    log::info!("Attempting to decrypt wallet");
                    match bitvault_core::crypto::decrypt_seed(encrypted_data, &state.pin_input) {
                        Ok(seed_phrase) => {
                            // Successfully decrypted
                            state.seed_phrase = Some(seed_phrase);
                            state.wallet_state = WalletState::Unlocked;
                            state.current_view = View::Wallet;
                            state.lock_error = None;
                            state.pin_input.clear(); // Clear PIN input for security
                            log::info!("Wallet unlocked successfully");
                        }
                        Err(e) => {
                            // Failed to decrypt
                            log::error!("Failed to decrypt wallet: {}", e);
                            state.lock_error = Some("Incorrect PIN. Please try again.".to_string());
                        }
                    }
                } else {
                    state.lock_error =
                        Some("No wallet data found. Please create a new wallet.".to_string());
                }
            }
        }

        ui.add_space(20.0);

        if ui.button("Back to Home").clicked() {
            if let Ok(mut state) = app.state.write() {
                state.current_view = View::Home;
                state.wallet_state = WalletState::New;
                state.pin_input.clear();
                state.lock_error = None;
                log::info!("Returning to home screen");
            }
        }
    });
}
