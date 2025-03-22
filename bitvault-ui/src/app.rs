#![allow(non_snake_case)]
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;
use eframe::{
    egui::{self, Color32, Context, Id, Rect, RichText, Ui},
    CreationContext,
};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::wallet;
use bitvault_core::crypto;

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
    SplashScreen,
    OnboardingOne,
    OnboardingTwo,
    OnboardingThree,
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
    pub splash_timer: Option<f32>,    // Timer for splash screen (in seconds)
    pub testing_mode: bool,           // Flag for testing mode to bypass lock screen
    pub onboarding_completed: bool,   // Flag to track if onboarding has been completed
}

// Create a type alias for a thread-safe, shared reference to the state
pub type SharedAppState = Arc<RwLock<AppState>>;

pub struct BitVaultApp {
    state: SharedAppState,
}

impl BitVaultApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        // Check for testing mode environment variable
        let testing_mode = std::env::var("TESTING").unwrap_or_default() == "1";
        if testing_mode {
            log::info!("Running in TESTING mode - lock screen will be bypassed");
        }

        // Create the app with default state
        let app = Self {
            state: Arc::new(RwLock::new(AppState {
                current_view: View::SplashScreen,
                splash_timer: Some(1.0), // 1 second splash screen
                testing_mode,
                ..Default::default()
            })),
        };

        // Check if a wallet file exists and load it
        match app.load_wallet_from_disk() {
            Ok(encrypted_data) => {
                if let Ok(mut state) = app.state.write() {
                    state.encrypted_wallet_data = Some(encrypted_data);
                    state.wallet_state = WalletState::Locked;
                    // Don't set current_view here, as we want to show splash screen first
                    log::info!(
                        "Existing wallet found. Will start in locked mode after splash screen."
                    );
                }
            }
            Err(e) => {
                // No wallet file found or error loading it - this is normal for first run
                log::info!(
                    "No existing wallet found: {}. Will start in new wallet mode after splash screen.",
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

                // Check for Enter key press
                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if enter_pressed {
                    // Store the enter key state in memory for use outside this scope
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(Id::new("pin_enter_pressed"), true);
                    });
                }
            });

            ui.add_space(20.0);

            // Calculate pin_valid based on current values
            let pin_valid = !pin_input.is_empty() && pin_input == pin_confirm;

            // Get the enter key state from memory
            let enter_pressed = ui
                .memory(|mem| mem.data.get_temp::<bool>(Id::new("pin_enter_pressed")))
                .unwrap_or(false);

            // Set PIN button
            if ui
                .add_enabled(pin_valid, egui::Button::new("Set PIN"))
                .clicked()
                || (enter_pressed && pin_valid)
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
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut state.verification_input)
                            .hint_text("Enter your 12 words in order, separated by spaces")
                            .desired_width(400.0)
                            .desired_rows(3),
                    );

                    // Check for Enter key press
                    let enter_pressed =
                        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    ui.add_space(20.0);

                    if (ui.button("Restore Wallet").clicked() || enter_pressed)
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

            // Check for Enter key press
            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

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
            if verify_clicked || enter_pressed {
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
                        match crypto::encrypt_seed(seed, pin) {
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

            // Check for Enter key press
            let enter_pressed =
                pin_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            // Display error message if there is one
            if let Ok(state) = self.state.read() {
                if let Some(error) = &state.lock_error {
                    ui.add_space(10.0);
                    ui.colored_label(Color32::RED, error);
                }
            }

            ui.add_space(20.0);

            let unlock_button = ui.add_enabled(wallet_loaded, egui::Button::new("Unlock"));
            if unlock_button.clicked() || (enter_pressed && wallet_loaded) {
                if let Ok(mut state) = self.state.write() {
                    // Try to load and decrypt the wallet
                    if let Some(encrypted_data) = &state.encrypted_wallet_data {
                        log::info!("Attempting to decrypt wallet");
                        match crypto::decrypt_seed(encrypted_data, &state.pin_input) {
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

    fn render_splash_screen(&self, ui: &mut Ui) {
        // Set the background to black
        let screen_rect = ui.max_rect();
        ui.painter().rect_filled(screen_rect, 0.0, Color32::BLACK);

        // Track how many times this method is called
        static RENDER_COUNT: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let count = RENDER_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        log::trace!("Render splash screen called {} times", count);

        // Center the logo
        ui.vertical_centered(|ui| {
            // Center vertically - push down less to make it more centered
            ui.add_space(screen_rect.height() / 4.0);

            // Use a static texture handle to avoid reloading on every frame
            static TEXTURE_ID: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

            let texture_id = TEXTURE_ID.get_or_init(|| {
                log::debug!("Loading image - this should only happen once");

                // Try to load the image from the file system - try multiple paths
                let possible_paths = [
                    "public/splash_logo.png",
                    "./public/splash_logo.png",
                    "../public/splash_logo.png",
                    "bitvault-ui/public/splash_logo.png",
                ];

                for path in possible_paths {
                    log::debug!("Trying to load image from: {}", path);
                    if let Ok(image_data) = std::fs::read(path) {
                        // Use the image crate to decode the image
                        if let Ok(image) = image::load_from_memory(&image_data) {
                            let size = [image.width() as _, image.height() as _];
                            let image_buffer = image.to_rgba8();
                            let pixels = image_buffer.as_flat_samples();

                            let color_image =
                                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                            let texture = ui.ctx().load_texture(
                                "splash_logo",
                                color_image,
                                Default::default(),
                            );

                            log::debug!("Image loaded successfully from {}", path);
                            return Some(texture);
                        }
                    }
                }

                log::error!("Failed to read image file from any path");
                None
            });

            match texture_id {
                Some(texture) => {
                    // Make the image smaller (50% of original size)
                    let scale = 0.5;
                    ui.add(egui::Image::new(texture).fit_to_original_size(scale));
                    log::trace!("Image added to frame {}", count);
                }
                None => {
                    ui.colored_label(Color32::RED, "Failed to load splash image");
                    log::error!("No texture available for splash screen");
                }
            }
        });

        // Request a repaint to ensure the timer updates even without mouse movement
        ui.ctx().request_repaint();
    }

    // Common function to render a centered onboarding container
    fn render_onboarding_container(&self, ui: &mut Ui, render_content: impl FnOnce(&mut Ui)) {
        // Set the background to white
        let screen_rect = ui.max_rect();
        ui.painter().rect_filled(screen_rect, 0.0, Color32::WHITE);

        // Calculate the available space
        let available_width = screen_rect.width();
        let available_height = screen_rect.height();

        // Content width (fixed at 328px for mobile designs)
        let content_width: f32 = 328.0;
        let content_height: f32 = 650.0; // Approximate height of content

        // Calculate vertical padding to center content
        let min_padding: f32 = 10.0;
        let vertical_padding = (available_height - content_height).max(min_padding) / 2.0;

        // Add container that centers content both horizontally and vertically
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(vertical_padding);

                    // Create a container with fixed width but centered horizontally
                    let min_side_margin: f32 = 20.0;
                    let container_width = content_width.min(available_width - min_side_margin);
                    ui.allocate_ui_with_layout(
                        egui::vec2(container_width, content_height),
                        egui::Layout::top_down(egui::Align::Center),
                        render_content,
                    );

                    ui.add_space(vertical_padding);
                });
            });
    }

    fn render_onboarding_one(&self, ui: &mut Ui) {
        self.render_onboarding_container(ui, |ui| {
            // Upper panel with illustration
            ui.add_space(80.0); // Status bar + top spacing

            // Illustration frame
            let _ill_frame = ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
                // Center shape with circle outline
                let circle_center = ui.min_rect().center();
                let circle_radius = 97.0;

                // Draw outer circle outline
                ui.painter().circle_stroke(
                    circle_center,
                    circle_radius,
                    egui::Stroke::new(0.8, Color32::from_rgb(212, 212, 212)),
                );

                // Draw the 'V' icon in the center
                let rect = egui::Rect::from_center_size(
                    circle_center,
                    egui::vec2(43.0, 42.0),
                );
                ui.painter().rect_filled(
                    rect,
                    0.0,
                    Color32::BLACK,
                );

                // Draw bubbles
                // Blue bubble (Cloud)
                self.draw_feature_bubble(ui,
                    circle_center + egui::vec2(-69.0, -80.0),
                    Color32::from_rgb(204, 236, 253), // Light blue
                    Color32::from_rgb(51, 176, 246), // Blue icon
                    "☁");

                // Purple bubble (Devices)
                self.draw_feature_bubble(ui,
                    circle_center + egui::vec2(-85.0, 40.0),
                    Color32::from_rgb(227, 224, 252), // Light purple
                    Color32::from_rgb(114, 105, 218), // Purple icon
                    "⚙");

                // Red bubble (Mobile)
                self.draw_feature_bubble(ui,
                    circle_center + egui::vec2(80.0, 40.0),
                    Color32::from_rgb(255, 202, 202), // Light red
                    Color32::from_rgb(250, 82, 82), // Red icon
                    "📱");

                // Orange circle (Bitcoin logo)
                let bitcoin_center = circle_center + egui::vec2(85.0, -70.0);
                ui.painter().circle_filled(
                    bitcoin_center,
                    18.0,
                    Color32::from_rgb(247, 147, 26), // Bitcoin orange
                );
            });

            // Content
            ui.add_space(32.0);
            ui.heading(RichText::new("Multisig security").color(Color32::BLACK).size(24.0));
            ui.add_space(8.0);
            ui.label(
                RichText::new("Secure your funds with 2-of-3 multisig vaults. For extra security spread the 3 keys across 3 different geolocation and store an extra copy of one key in a physical vault or similar.")
                .color(Color32::from_rgb(82, 82, 82))
                .size(14.0)
            );

            // Indicators
            ui.add_space(24.0);
            self.draw_navigation_arrows(ui, 1);

            // Buttons at the bottom
            ui.add_space(32.0);

            if ui.add(egui::Button::new(
                    RichText::new("Create a new wallet")
                    .color(Color32::WHITE)
                    .size(16.0))
                .min_size(egui::vec2(328.0, 48.0))
                .fill(Color32::BLACK)
                .rounding(16.0)
            ).clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::OnboardingTwo;
                }
            }

            ui.add_space(8.0);

            if ui.add(egui::Button::new(
                    RichText::new("I already have a wallet")
                    .color(Color32::BLACK)
                    .size(16.0))
                .min_size(egui::vec2(328.0, 48.0))
                .stroke(egui::Stroke::new(1.0, Color32::from_gray(200)))
                .rounding(16.0)
            ).clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::Home;
                    state.onboarding_completed = true;
                }
            }

            ui.add_space(8.0);
            ui.label(
                RichText::new("By continuing, I agree to the Terms of Service")
                .color(Color32::from_rgb(82, 82, 82))
                .size(12.0)
            );

            // Navigation hint
            ui.add_space(4.0);
            ui.label(
                RichText::new("Tip: Use ← → arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0)
            );
        });
    }

    fn render_onboarding_two(&self, ui: &mut Ui) {
        self.render_onboarding_container(ui, |ui| {
            // Upper panel with illustration
            ui.add_space(80.0); // Status bar + top spacing

            // Illustration frame
            ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
                // Center shape with square outline
                let center = ui.min_rect().center();

                // Create a square outline (main frame)
                let square_size = 140.0;
                let square_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(square_size, square_size),
                );
                ui.painter().rect_stroke(
                    square_rect,
                    0.0,
                    egui::Stroke::new(1.5, Color32::from_rgb(38, 38, 38))
                );

                // Add the clock in the middle
                let clock_size = 64.0;
                let _clock_rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(clock_size, clock_size),
                );
                // Draw clock circle
                ui.painter().circle_filled(
                    center,
                    clock_size / 2.0,
                    Color32::from_rgb(153, 194, 77),  // Green color for clock
                );

                // Add cube elements on the left
                self.draw_cube(ui, center + egui::vec2(-93.5, -46.5));
                self.draw_cube(ui, center + egui::vec2(-93.5, 46.5));

                // Add cube elements on the right
                self.draw_cube(ui, center + egui::vec2(93.5, -46.5));
                self.draw_cube(ui, center + egui::vec2(93.5, 46.5));
            });

            // Content
            ui.add_space(32.0);
            ui.heading(RichText::new("Time-delay protection").color(Color32::BLACK).size(28.0));
            ui.add_space(8.0);
            ui.label(
                RichText::new("Set time-delays and prevent unauthorised withdrawals. The xPUB is of VITAL importance to recover your multisig vault. Keep AT LEAST a copy of the xPUB together with each key.")
                .color(Color32::from_rgb(82, 82, 82))
                .size(14.0)
            );

            // Indicators
            ui.add_space(24.0);
            self.draw_navigation_arrows(ui, 2);

            // Buttons at the bottom
            ui.add_space(32.0);

            if ui.add(egui::Button::new(
                    RichText::new("Continue")
                    .color(Color32::WHITE)
                    .size(16.0))
                .min_size(egui::vec2(328.0, 48.0))
                .fill(Color32::BLACK)
                .rounding(16.0)
            ).clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::OnboardingThree;
                }
            }

            ui.add_space(8.0);

            if ui.add(egui::Button::new(
                    RichText::new("Back")
                    .color(Color32::BLACK)
                    .size(16.0))
                .min_size(egui::vec2(328.0, 48.0))
                .stroke(egui::Stroke::new(1.0, Color32::from_gray(200)))
                .rounding(16.0)
            ).clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::OnboardingOne;
                }
            }

            ui.add_space(8.0);
            ui.label(
                RichText::new("By continuing, I agree to the Terms of Service")
                .color(Color32::from_rgb(82, 82, 82))
                .size(12.0)
            );

            // Navigation hint
            ui.add_space(4.0);
            ui.label(
                RichText::new("Tip: Use ← → arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0)
            );
        });
    }

    fn render_onboarding_three(&self, ui: &mut Ui) {
        self.render_onboarding_container(ui, |ui| {
            // Upper panel with illustration
            ui.add_space(80.0); // Status bar + top spacing

            // Illustration frame
            ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
                // Center position
                let center = ui.min_rect().center();

                // Draw a shield shape for the notification icon
                let shield_size = 100.0;
                let shield_radius = shield_size / 2.0;

                // Draw shield background (light gray)
                ui.painter().circle_filled(
                    center,
                    shield_radius,
                    Color32::from_rgb(245, 245, 245),
                );

                // Draw shield outline
                ui.painter().circle_stroke(
                    center,
                    shield_radius,
                    egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
                );

                // Draw lock icon inside the shield
                let lock_size = 40.0;
                let lock_top = center.y - lock_size * 0.2;
                let lock_bottom = center.y + lock_size * 0.5;
                let lock_left = center.x - lock_size * 0.3;
                let lock_right = center.x + lock_size * 0.3;

                // Lock body
                let lock_body = egui::Rect::from_min_max(
                    egui::pos2(lock_left, lock_top),
                    egui::pos2(lock_right, lock_bottom),
                );
                ui.painter().rect_filled(
                    lock_body,
                    5.0,
                    Color32::from_rgb(30, 30, 30),
                );

                // Lock shackle (arc)
                let shackle_radius = lock_size * 0.4;
                let shackle_center = egui::pos2(center.x, lock_top - shackle_radius * 0.3);
                let shackle_stroke = egui::Stroke::new(6.0, Color32::from_rgb(30, 30, 30));

                // Draw a semi-circle for the shackle
                ui.painter().circle_stroke(shackle_center, shackle_radius, shackle_stroke);

                // Draw notification markers (circles) around main icon
                let marker_positions = [
                    egui::vec2(-120.0, -70.0),
                    egui::vec2(120.0, -70.0),
                    egui::vec2(-120.0, 70.0),
                    egui::vec2(120.0, 70.0),
                ];

                for pos in marker_positions {
                    let marker_pos = center + pos;

                    // Draw marker circle
                    ui.painter().circle_filled(
                        marker_pos,
                        10.0,
                        Color32::from_rgb(240, 240, 240),
                    );

                    // Draw a small dot inside the circle
                    ui.painter().circle_filled(
                        marker_pos,
                        3.0,
                        Color32::from_rgb(150, 150, 150),
                    );
                }
            });

            // Content
            ui.add_space(32.0);
            ui.heading(RichText::new("Secret notifications").color(Color32::BLACK).size(28.0));
            ui.add_space(8.0);
            ui.label(
                RichText::new("Stay informed about important wallet events and security updates. Secret notifications are end-to-end encrypted to protect your privacy and security.")
                .color(Color32::from_rgb(82, 82, 82))
                .size(14.0)
            );

            // Indicators
            ui.add_space(24.0);
            self.draw_navigation_arrows(ui, 3);

            // Buttons at the bottom
            ui.add_space(32.0);

            if ui.add(egui::Button::new(
                    RichText::new("Let's go!")
                    .color(Color32::WHITE)
                    .size(16.0))
                .min_size(egui::vec2(328.0, 48.0))
                .fill(Color32::BLACK)
                .rounding(16.0)
            ).clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::Home;
                    state.onboarding_completed = true;
                }
            }

            ui.add_space(8.0);

            if ui.add(egui::Button::new(
                    RichText::new("Back")
                    .color(Color32::BLACK)
                    .size(16.0))
                .min_size(egui::vec2(328.0, 48.0))
                .stroke(egui::Stroke::new(1.0, Color32::from_gray(200)))
                .rounding(16.0)
            ).clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::OnboardingTwo;
                }
            }

            ui.add_space(8.0);
            ui.label(
                RichText::new("By continuing, I agree to the Terms of Service")
                .color(Color32::from_rgb(82, 82, 82))
                .size(12.0)
            );

            // Navigation hint
            ui.add_space(4.0);
            ui.label(
                RichText::new("Tip: Use ← → arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0)
            );
        });
    }

    // Helper function to draw feature bubbles on the onboarding screen
    fn draw_feature_bubble(
        &self,
        ui: &mut Ui,
        center: egui::Pos2,
        bg_color: Color32,
        icon_color: Color32,
        icon: &str,
    ) {
        // Draw circle background
        ui.painter().circle_filled(center, 18.0, bg_color);

        // Draw icon
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(14.0),
            icon_color,
        );
    }

    // Helper function to draw cubes for the time-delay screen
    fn draw_cube(&self, ui: &mut Ui, pos: egui::Pos2) {
        let cube_size = 48.0;
        let cube_rect = egui::Rect::from_center_size(pos, egui::vec2(cube_size, cube_size));

        // Draw cube inner shape
        let inner_margin = cube_size * 0.094; // 9.38% of size
        let inner_rect = cube_rect.shrink(inner_margin);
        ui.painter()
            .rect_filled(inner_rect, 0.0, Color32::from_rgb(38, 38, 38));
    }

    // Helper function to draw arrow navigation indicators
    fn draw_navigation_arrows(&self, ui: &mut Ui, screen_number: usize) {
        let total_width = ui.available_width();

        ui.horizontal(|ui| {
            // Left padding - depends on the current screen (no left arrow on first screen)
            let left_arrow_visible = screen_number > 1;
            let offset_mult = if left_arrow_visible { 6.0 } else { 7.5 };
            ui.add_space(total_width / offset_mult);

            // Left arrow - only for screens 2 and 3
            if left_arrow_visible {
                ui.label(
                    RichText::new("←")
                        .color(Color32::from_rgb(120, 120, 120))
                        .size(16.0),
                );
                ui.add_space(10.0);
            }

            // Indicator dots
            for i in 1..=3 {
                if i > 1 {
                    ui.add_space(4.0);
                }

                if i == screen_number {
                    // Active indicator
                    ui.add(
                        egui::widgets::Button::new("")
                            .min_size(egui::vec2(15.0, 5.0))
                            .fill(Color32::from_rgb(17, 165, 238))
                            .frame(false),
                    );
                } else {
                    // Inactive indicator (clickable)
                    let clicked = ui
                        .add(
                            egui::widgets::Button::new("")
                                .min_size(egui::vec2(5.0, 5.0))
                                .fill(Color32::from_rgb(217, 217, 217))
                                .frame(false),
                        )
                        .clicked();

                    if clicked {
                        // Store which view to navigate to
                        let target_view = match i {
                            1 => View::OnboardingOne,
                            2 => View::OnboardingTwo,
                            3 => View::OnboardingThree,
                            _ => View::OnboardingOne, // Fallback
                        };

                        // Update state in a separate step after UI is processed
                        ui.ctx().request_repaint();

                        if let Ok(mut state) = self.state.write() {
                            state.current_view = target_view;
                        }
                    }
                }
            }

            // Right arrow - only for screens 1 and 2
            if screen_number < 3 {
                ui.add_space(10.0);
                ui.label(
                    RichText::new("→")
                        .color(Color32::from_rgb(120, 120, 120))
                        .size(16.0),
                );
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
        // Always request a repaint when in splash screen mode to ensure timer updates
        if let Ok(state) = self.state.read() {
            if state.current_view == View::SplashScreen {
                // Request continuous repaints during splash screen
                ctx.request_repaint();
            }
        }

        // Check for arrow key navigation in onboarding screens
        if let Ok(state) = self.state.read() {
            if matches!(
                state.current_view,
                View::OnboardingOne | View::OnboardingTwo | View::OnboardingThree
            ) {
                // Check for left/right arrow keys
                let right_pressed = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight));
                let left_pressed = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft));

                // Store current view for use outside of the read lock
                let current_view = state.current_view.clone();

                // Release the read lock before attempting to acquire a write lock
                drop(state);

                if right_pressed || left_pressed {
                    // Now get a write lock to potentially change the view
                    if let Ok(mut state) = self.state.write() {
                        match (current_view, right_pressed, left_pressed) {
                            (View::OnboardingOne, true, _) => {
                                state.current_view = View::OnboardingTwo
                            }
                            (View::OnboardingTwo, true, _) => {
                                state.current_view = View::OnboardingThree
                            }
                            (View::OnboardingTwo, _, true) => {
                                state.current_view = View::OnboardingOne
                            }
                            (View::OnboardingThree, _, true) => {
                                state.current_view = View::OnboardingTwo
                            }
                            _ => {} // No change for other combinations
                        }
                    }
                }
            }
        }

        // Update the splash timer if active
        if let Ok(mut state) = self.state.write() {
            if let Some(timer) = state.splash_timer {
                let new_timer = timer - ctx.input(|i| i.unstable_dt).min(0.1);
                if new_timer <= 0.0 {
                    state.splash_timer = None;

                    // When in testing mode, bypass lock screen and go to onboarding
                    if state.testing_mode {
                        state.current_view = View::OnboardingOne;
                        log::info!(
                            "Testing mode active: Bypassing lock screen and showing onboarding"
                        );
                    }
                    // Normal flow - go to lock screen or home
                    else if state.wallet_state == WalletState::Locked {
                        state.current_view = View::LockScreen;
                    } else {
                        state.current_view = View::Home;
                    }

                    log::info!(
                        "Splash screen finished, transitioning to {:?}",
                        state.current_view
                    );
                } else {
                    state.splash_timer = Some(new_timer);
                }
            }

            // Update the copy feedback timer if it exists
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
                View::SplashScreen => self.render_splash_screen(ui),
                View::OnboardingOne => self.render_onboarding_one(ui),
                View::OnboardingTwo => self.render_onboarding_two(ui),
                View::OnboardingThree => self.render_onboarding_three(ui),
            }
        });
    }
}
