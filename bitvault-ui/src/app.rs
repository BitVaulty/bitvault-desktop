#![allow(non_snake_case)]
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use eframe::{
    egui::{self, Color32, Context, Id, Rect, RichText, Ui},
    CreationContext,
};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::wallet;

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum WalletState {
    #[default]
    New,
    Creating,
    Restoring,
    Unlocked,
    Locked,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum View {
    #[default]
    Home,
    Disclaimer,
    PinChoice,
    Seed,
    SeedVerify,
    Wallet,
    LockScreen,
}

// Define a struct to hold the global state
#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub user_pin: Option<String>,
    pub wallet_state: WalletState,
    pub current_view: View,
    pub pin_input: String,
    pub pin_confirm: String,
    pub seed_phrase: Option<String>,
    pub seed_words: Vec<String>,
    pub verification_input: String,
    pub copied_feedback: Option<f32>, // Timer for showing copy feedback (in seconds)
    pub encrypted_wallet_data: Option<String>, // Encrypted wallet data stored on disk
    pub lock_error: Option<String>,   // Error message when unlocking fails
}

// Create a type alias for a thread-safe, shared reference to the state
pub type SharedAppState = Arc<RwLock<AppState>>;

pub struct BitVaultApp {
    state: SharedAppState,
}

impl BitVaultApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        // Create the app with default state
        let app = Self {
            state: Arc::new(RwLock::new(AppState::default())),
        };

        // Check if a wallet file exists and load it
        match app.load_wallet_from_disk() {
            Ok(encrypted_data) => {
                if let Ok(mut state) = app.state.write() {
                    state.encrypted_wallet_data = Some(encrypted_data);
                    state.wallet_state = WalletState::Locked;
                    state.current_view = View::LockScreen;
                    log::info!("Existing wallet found. Starting in locked mode.");
                }
            }
            Err(e) => {
                // No wallet file found or error loading it - this is normal for first run
                log::info!(
                    "No existing wallet found: {}. Starting in new wallet mode.",
                    e
                );
            }
        }

        app
    }

    fn render_home(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Welcome to BitVault");
            ui.add_space(20.0);

            ui.label("Your secure Bitcoin wallet");
            ui.add_space(30.0);

            if ui.button("Create New Wallet").clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.wallet_state = WalletState::Creating;
                    state.current_view = View::Disclaimer;
                }
            }

            ui.add_space(10.0);

            if ui.button("Restore Existing Wallet").clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.wallet_state = WalletState::Restoring;
                    state.current_view = View::Disclaimer;
                }
            }

            let back_button_response = ui.add(egui::Button::new("Go Back"));
            if back_button_response.clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::Home;
                }
            }
            crate::icons::draw_caret_left(ui, back_button_response.rect, Color32::WHITE);
        });
    }

    fn render_disclaimer(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Important Disclaimer");
            ui.add_space(20.0);

            ui.label(RichText::new("Please read carefully before proceeding:").strong());
            ui.add_space(10.0);

            let disclaimer_text = "
            1. BitVault is a self-custody wallet. You are solely responsible for your funds.

            2. Your recovery phrase (seed) is the ONLY way to recover your wallet if you lose access.

            3. Never share your recovery phrase or PIN with anyone.

            4. Always back up your recovery phrase in a secure location.

            5. If you lose your recovery phrase, you will permanently lose access to your funds.

            6. BitVault cannot recover your wallet or funds if you lose your recovery phrase.
            ";

            ui.label(disclaimer_text);
            ui.add_space(20.0);

            if ui.button("I Understand and Accept").clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::PinChoice;
                }
            }

            let back_button_response = ui.add(egui::Button::new("Go Back"));
            if back_button_response.clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.wallet_state = WalletState::New;
                    state.current_view = View::Home;
                }
            }
            crate::icons::draw_caret_left(ui, back_button_response.rect, Color32::WHITE);
        });
    }

    fn render_pin_choice(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Create a PIN");
            ui.add_space(20.0);

            ui.label("Choose a secure PIN to protect your wallet");
            ui.add_space(10.0);

            let mut pin_input = String::new();
            let mut pin_confirm = String::new();
            let mut back_button_clicked = false;

            // Read current state values
            if let Ok(state) = self.state.read() {
                pin_input = state.pin_input.clone();
                pin_confirm = state.pin_confirm.clone();
            }

            // PIN input fields
            ui.horizontal(|ui| {
                ui.label("PIN: ");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut pin_input)
                        .password(true)
                        .hint_text("Enter PIN")
                        .desired_width(200.0),
                );

                // Update state with new input
                if response.changed() {
                    if let Ok(mut state) = self.state.write() {
                        state.pin_input = pin_input.clone();
                    }
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Confirm PIN: ");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut pin_confirm)
                        .password(true)
                        .hint_text("Confirm PIN")
                        .desired_width(200.0),
                );

                // Update state with new input
                if response.changed() {
                    if let Ok(mut state) = self.state.write() {
                        state.pin_confirm = pin_confirm.clone();
                    }
                }
            });

            ui.add_space(20.0);

            // Calculate pin_valid based on current values
            let pin_valid = !pin_input.is_empty() && pin_input == pin_confirm;

            // Set PIN button
            if ui
                .add_enabled(pin_valid, egui::Button::new("Set PIN"))
                .clicked()
            {
                log::info!("Set PIN button clicked, PIN is valid: {}", pin_valid);

                if pin_valid {
                    if let Ok(mut state) = self.state.write() {
                        // Store the PIN
                        state.user_pin = Some(pin_input.clone());
                        log::info!("PIN set successfully");

                        // Clear the input fields for security
                        state.pin_input.clear();
                        state.pin_confirm.clear();

                        // Move to the next step
                        if state.wallet_state == WalletState::Creating {
                            log::info!("Moving to Seed view for new wallet creation");
                            // For creating a new wallet, move to the seed view
                            state.current_view = View::Seed;
                        } else if state.wallet_state == WalletState::Restoring {
                            log::info!("Moving to Seed view for wallet restoration");
                            // For restoring, go to the seed view where the user can enter their seed phrase
                            state.current_view = View::Seed;
                        }
                    }
                }
            }

            // Back button
            let back_response = ui.button("Go Back");
            if back_response.clicked() {
                back_button_clicked = true;
            }

            // Handle back button click outside the state read lock
            if back_button_clicked {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::Disclaimer;
                }
            }

            // Draw the back button icon
            crate::icons::draw_caret_left(ui, back_response.rect, Color32::WHITE);
        });
    }

    fn render_seed(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            // Check if we're in creating or restoring mode
            let (is_creating, has_seed, _current_seed) = if let Ok(state) = self.state.read() {
                (
                    state.wallet_state == WalletState::Creating,
                    state.seed_phrase.is_some(),
                    state.seed_phrase.clone(),
                )
            } else {
                (false, false, None)
            };

            // Handle seed phrase generation for new wallet
            if is_creating && !has_seed {
                // Generate seed phrase outside of any locks to avoid deadlocks
                match wallet::new_12_word_seed() {
                    Ok(new_seed) => {
                        log::info!("Successfully generated new seed phrase");
                        // Store the seed phrase in the state
                        if let Ok(mut state) = self.state.write() {
                            state.seed_phrase = Some(new_seed.clone());
                            // Split the seed phrase into words for the verification step
                            state.seed_words =
                                new_seed.split_whitespace().map(String::from).collect();
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to generate seed phrase: {}", e);
                    }
                }
            }

            // Re-read state after potential updates
            let (is_creating, seed_phrase, has_feedback) = if let Ok(state) = self.state.read() {
                (
                    state.wallet_state == WalletState::Creating,
                    state.seed_phrase.clone(),
                    state.copied_feedback.is_some(),
                )
            } else {
                (false, None, false)
            };

            if is_creating {
                ui.heading("Your Recovery Phrase");
                ui.add_space(10.0);
                ui.label("Write down these 12 words in order and keep them safe:");

                if let Some(seed_phrase) = &seed_phrase {
                    ui.add_space(20.0);

                    let words: Vec<&str> = seed_phrase.split_whitespace().collect();

                    if !words.is_empty() {
                        egui::Grid::new("seed_grid")
                            .num_columns(4)
                            .spacing([20.0, 10.0])
                            .show(ui, |ui| {
                                for (i, word) in words.iter().enumerate() {
                                    ui.label(format!("{}. {}", i + 1, word));

                                    if (i + 1) % 4 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });

                        ui.add_space(20.0);

                        // Add a copy to clipboard button
                        let copy_clicked = ui.button("📋 Copy to Clipboard").clicked();

                        // Show feedback if active
                        if has_feedback {
                            ui.label(RichText::new("✓ Copied to clipboard!").color(Color32::GREEN));
                        }

                        // Handle copy button click
                        if copy_clicked {
                            ui.output_mut(|o| o.copied_text = seed_phrase.clone());
                            if let Ok(mut state) = self.state.write() {
                                state.copied_feedback = Some(2.0); // Show feedback for 2 seconds
                            }
                        }

                        ui.add_space(20.0);

                        if ui.button("Continue").clicked() {
                            if let Ok(mut state) = self.state.write() {
                                state.current_view = View::SeedVerify;
                            }
                        }
                    } else {
                        ui.label("Error: Invalid seed phrase format");
                    }
                } else {
                    ui.label("Generating seed phrase...");
                    ui.spinner();
                }
            } else {
                // Restoring flow
                ui.heading("Restore from Recovery Phrase");
                ui.add_space(10.0);
                ui.label("Enter your 12-word recovery phrase:");

                if let Ok(mut state) = self.state.write() {
                    ui.add_space(20.0);

                    // Text input for seed phrase
                    ui.add(
                        egui::TextEdit::multiline(&mut state.verification_input)
                            .hint_text("Enter your 12 words in order, separated by spaces")
                            .desired_width(400.0)
                            .desired_rows(3),
                    );

                    ui.add_space(20.0);

                    if ui.button("Restore Wallet").clicked()
                        && !state.verification_input.trim().is_empty()
                    {
                        // Set the seed phrase from the input
                        state.seed_phrase = Some(state.verification_input.clone());
                        state.current_view = View::Wallet;
                        state.wallet_state = WalletState::Unlocked;
                    }
                }
            }

            let back_button_response = ui.add(egui::Button::new("Go Back"));
            if back_button_response.clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::PinChoice;
                }
            }
            crate::icons::draw_caret_left(ui, back_button_response.rect, Color32::WHITE);
        });
    }

    fn render_seed_verify(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Verify Recovery Phrase");
            ui.add_space(10.0);
            ui.label(
                "Please enter your recovery phrase to verify you've written it down correctly:",
            );

            // Read state once to get the values we need
            let (original_seed, mut verification_input, mut verification_result) =
                if let Ok(state) = self.state.read() {
                    (
                        state.seed_phrase.clone().unwrap_or_default(),
                        state.verification_input.clone(),
                        None,
                    )
                } else {
                    (String::new(), String::new(), None)
                };

            ui.add_space(20.0);

            // Text input for verification
            let response = ui.add(
                egui::TextEdit::multiline(&mut verification_input)
                    .hint_text("Enter your 12 words in order, separated by spaces")
                    .desired_width(400.0)
                    .desired_rows(3),
            );

            // Update the verification input in state if changed
            if response.changed() {
                if let Ok(mut state) = self.state.write() {
                    state.verification_input = verification_input.clone();
                }
            }

            ui.add_space(20.0);

            // Check if the entered text matches the original seed phrase
            let is_correct =
                !verification_input.is_empty() && verification_input.trim() == original_seed.trim();

            if !verification_input.is_empty() {
                if is_correct {
                    ui.label(RichText::new("✓ Correct!").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("✗ Incorrect. Please try again.").color(Color32::RED));
                }
            }

            ui.add_space(10.0);

            let verify_clicked = ui.button("Verify").clicked();
            let back_button = ui.add(egui::Button::new("Go Back"));
            let back_clicked = back_button.clicked();

            // Store the back button rect for the icon
            if back_button.rect.width() > 0.0 {
                ui.memory_mut(|mem| {
                    mem.data
                        .insert_temp(Id::new("back_button"), back_button.rect)
                });
            }

            // Handle button clicks outside of any locks
            if verify_clicked {
                if is_correct {
                    // Verification successful - update state once
                    if let Ok(mut state) = self.state.write() {
                        log::info!("Seed verification successful, moving to wallet view");
                        state.current_view = View::Wallet;
                        state.wallet_state = WalletState::Unlocked;
                    }
                } else {
                    // Verification failed
                    log::warn!("Seed verification failed - phrases don't match");
                    verification_result = Some(false);
                }
            }

            if back_clicked {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::Seed;
                }
            }

            // Show verification result if needed
            if let Some(false) = verification_result {
                ui.add_space(5.0);
                ui.label(
                    RichText::new("Verification failed. Please check your recovery phrase.")
                        .color(Color32::RED),
                );
            }

            // Draw back button icon
            if let Some(rect) = ui.memory(|m| m.data.get_temp::<Rect>(Id::new("back_button"))) {
                crate::icons::draw_caret_left(ui, rect, Color32::WHITE);
            }
        });
    }

    fn render_wallet(&self, ui: &mut Ui) {
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

            if let Ok(mut state) = self.state.write() {
                // Check if we need to encrypt and save the wallet
                if state.encrypted_wallet_data.is_none()
                    && state.seed_phrase.is_some()
                    && state.user_pin.is_some()
                {
                    if let (Some(seed), Some(pin)) = (&state.seed_phrase, &state.user_pin) {
                        log::info!("Attempting to encrypt and save wallet");
                        match crate::crypto::encrypt_seed(seed, pin) {
                            Ok(encrypted_data) => {
                                // Save the encrypted wallet data to memory
                                state.encrypted_wallet_data = Some(encrypted_data.clone());
                                log::info!("Wallet encrypted successfully");

                                // Save the encrypted wallet data to disk
                                match self.save_wallet_to_disk(&encrypted_data) {
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
                if let Ok(mut state) = self.state.write() {
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

    fn render_lock_screen(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Unlock Your Wallet");
            ui.add_space(20.0);

            ui.label("Enter your PIN to unlock your wallet");
            ui.add_space(30.0);

            // Check if we need to load the wallet data from disk
            let wallet_loaded = if let Ok(mut state) = self.state.write() {
                if state.encrypted_wallet_data.is_none() {
                    log::info!("Attempting to load wallet data from disk");
                    match self.load_wallet_from_disk() {
                        Ok(encrypted_data) => {
                            state.encrypted_wallet_data = Some(encrypted_data);
                            log::info!("Wallet data loaded from disk successfully");
                            true
                        }
                        Err(e) => {
                            log::error!("Failed to load wallet data: {}", e);
                            state.lock_error = Some(
                                "Failed to load wallet data. Please create a new wallet."
                                    .to_string(),
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
            if let Ok(state) = self.state.read() {
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
                if let Ok(mut state) = self.state.write() {
                    state.pin_input = pin_input;
                }
            }

            // Display error message if there is one
            if let Ok(state) = self.state.read() {
                if let Some(error) = &state.lock_error {
                    ui.add_space(10.0);
                    ui.colored_label(Color32::RED, error);
                }
            }

            ui.add_space(20.0);

            let unlock_button = ui.add_enabled(wallet_loaded, egui::Button::new("Unlock"));
            if unlock_button.clicked() {
                if let Ok(mut state) = self.state.write() {
                    // Try to load and decrypt the wallet
                    if let Some(encrypted_data) = &state.encrypted_wallet_data {
                        log::info!("Attempting to decrypt wallet");
                        match crate::crypto::decrypt_seed(encrypted_data, &state.pin_input) {
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
                                state.lock_error =
                                    Some("Incorrect PIN. Please try again.".to_string());
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
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::Home;
                    state.wallet_state = WalletState::New;
                    state.pin_input.clear();
                    state.lock_error = None;
                    log::info!("Returning to home screen");
                }
            }
        });
    }

    // Helper function to get the wallet file path
    fn get_wallet_file_path() -> Option<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("bitvault");

            // Create directory if it doesn't exist
            if !app_config_dir.exists() && fs::create_dir_all(&app_config_dir).is_err() {
                return None;
            }

            return Some(app_config_dir.join("wallet.dat"));
        }
        None
    }

    // Save wallet data to disk
    fn save_wallet_to_disk(&self, encrypted_data: &str) -> Result<(), String> {
        if let Some(file_path) = Self::get_wallet_file_path() {
            fs::write(file_path, encrypted_data)
                .map_err(|e| format!("Failed to save wallet: {}", e))
        } else {
            Err("Could not determine wallet file path".to_string())
        }
    }

    // Load wallet data from disk
    fn load_wallet_from_disk(&self) -> Result<String, String> {
        if let Some(file_path) = Self::get_wallet_file_path() {
            if file_path.exists() {
                fs::read_to_string(file_path)
                    .map_err(|e| format!("Failed to read wallet file: {}", e))
            } else {
                Err("Wallet file does not exist".to_string())
            }
        } else {
            Err("Could not determine wallet file path".to_string())
        }
    }
}

impl eframe::App for BitVaultApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Update the copy feedback timer if it exists
        if let Ok(mut state) = self.state.write() {
            if let Some(timer) = state.copied_feedback {
                let new_timer = timer - ctx.input(|i| i.unstable_dt).min(0.1);
                if new_timer <= 0.0 {
                    state.copied_feedback = None;
                } else {
                    state.copied_feedback = Some(new_timer);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let view = if let Ok(state) = self.state.read() {
                state.current_view.clone()
            } else {
                View::Home
            };

            match view {
                View::Home => self.render_home(ui),
                View::Disclaimer => self.render_disclaimer(ui),
                View::PinChoice => self.render_pin_choice(ui),
                View::Seed => self.render_seed(ui),
                View::SeedVerify => self.render_seed_verify(ui),
                View::Wallet => self.render_wallet(ui),
                View::LockScreen => self.render_lock_screen(ui),
            }
        });
    }
}
