use crate::app::state::{View, WalletState};
use crate::app::BitVaultApp;
use bitvault_core::crypto;
use eframe::egui::{Color32, RichText, Ui};

pub fn render(app: &BitVaultApp, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("BitVault Wallet");
        ui.add_space(20.0);

        ui.label("Your wallet is now set up and ready to use!");
        ui.add_space(10.0);

        // This would be replaced with actual wallet functionality
        ui.label("Balance: 0.00000000 BTC");

        ui.add_space(20.0);

        if ui.button("Receive").clicked() {
            // Show receive address
        }

        if ui.button("Send").clicked() {
            // Show send interface
        }

        if ui.button("Transactions").clicked() {
            // Show transaction history
        }

        ui.add_space(20.0);

        // Ensure the wallet is encrypted and saved when first created
        let mut wallet_saved = false;
        let mut encryption_error = false;

        if let Ok(mut state) = app.state.write() {
            // Check if we need to encrypt and save the wallet
            if state.encrypted_wallet_data.is_none()
                && state.seed_phrase.is_some()
                && state.user_pin.is_some()
            {
                if let (Some(seed), Some(pin)) = (&state.seed_phrase, &state.user_pin) {
                    log::info!("Attempting to encrypt and save wallet");
                    match crypto::encrypt_seed(seed, pin) {
                        Ok(encrypted_data) => {
                            // Save the encrypted wallet data to memory
                            state.encrypted_wallet_data = Some(encrypted_data.clone());
                            log::info!("Wallet encrypted successfully");

                            // Save the encrypted wallet data to disk
                            match app.save_wallet_to_disk(&encrypted_data) {
                                Ok(_) => {
                                    log::info!("Wallet successfully saved to disk");
                                    wallet_saved = true;
                                }
                                Err(e) => {
                                    log::error!("Failed to save wallet: {}", e);
                                    encryption_error = true;
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to encrypt wallet: {}", e);
                            encryption_error = true;
                        }
                    }
                }
            } else if state.encrypted_wallet_data.is_some() {
                wallet_saved = true;
            }
        }

        if wallet_saved {
            ui.add_space(10.0);
            ui.label(RichText::new("✓ Wallet saved to disk").color(Color32::GREEN));
        } else if encryption_error {
            ui.add_space(10.0);
            ui.label(RichText::new("⚠ Failed to save wallet").color(Color32::RED));
        }

        ui.add_space(20.0);

        if ui.button("Lock Wallet").clicked() {
            if let Ok(mut state) = app.state.write() {
                // Set wallet state to locked
                state.wallet_state = WalletState::Locked;
                state.current_view = View::LockScreen;

                // Clear sensitive data from memory
                state.user_pin = None;
                state.pin_input.clear();
                state.pin_confirm.clear();
                state.seed_phrase = None;

                // Keep the encrypted data for later decryption
                // state.encrypted_wallet_data remains intact
                log::info!("Wallet locked");
            }
        }
    });
}
