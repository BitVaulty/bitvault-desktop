#![allow(non_snake_case)]
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;
use eframe::{
    egui::{self, Color32, Context, RichText, Ui},
    CreationContext,
};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::config::Settings;
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
    settings: Settings,
}

// Add this helper module for asset management at the top level before BitVaultApp struct
mod assets {
    use eframe::egui;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    // Base paths to try for asset loading
    const BASE_PATHS: [&str; 3] = ["bitvault-ui", ".", ".."];

    // Find the correct base path once
    fn get_base_path() -> &'static PathBuf {
        static BASE_PATH: OnceLock<PathBuf> = OnceLock::new();

        BASE_PATH.get_or_init(|| {
            for base in BASE_PATHS {
                let path = PathBuf::from(base);
                if path.exists() {
                    return path;
                }
            }
            // Default to current directory if nothing found
            PathBuf::from(".")
        })
    }

    // Load a font file
    pub fn load_font(font_name: &str) -> Option<Vec<u8>> {
        let base = get_base_path();
        let font_path = base.join("assets").join(font_name);

        std::fs::read(&font_path).ok()
    }

    // Load an image file
    pub fn load_image(path: &str) -> Option<Vec<u8>> {
        let base = get_base_path();
        let img_path = base.join(path);

        std::fs::read(&img_path).ok()
    }

    // SVG loading function that works with the existing dependencies
    pub fn load_svg_as_texture(
        ctx: &egui::Context,
        name: &str,
        path: &str,
    ) -> Option<egui::TextureHandle> {
        let base = get_base_path();
        let svg_path = base.join(path);

        log::debug!("Loading SVG from: {:?}", svg_path);

        // First read the SVG file
        let svg_data = std::fs::read_to_string(&svg_path).ok()?;

        // Parse SVG with usvg
        let opt = usvg::Options {
            ..Default::default()
        };

        let tree = usvg::Tree::from_str(&svg_data, &opt).ok()?;

        // Get the size and create a pixmap
        let size = tree.size();

        // Apply a scale factor to increase resolution (2.0 = double resolution)
        let scale_factor = 2.0;
        let scaled_width = (size.width() * scale_factor) as u32;
        let scaled_height = (size.height() * scale_factor) as u32;

        let pixmap_size = tiny_skia::IntSize::from_wh(scaled_width, scaled_height)?;

        // Create a pixmap (tiny-skia's bitmap for rendering)
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;

        // Render the SVG tree to the pixmap with the scale transform
        resvg::render(
            &tree,
            tiny_skia::Transform::from_scale(scale_factor, scale_factor),
            &mut pixmap.as_mut(),
        );

        // Convert to egui texture
        let image_size = [pixmap_size.width() as _, pixmap_size.height() as _];
        let image_data = pixmap.data();

        // Create the color image and texture
        let color_image = egui::ColorImage::from_rgba_unmultiplied(image_size, image_data);

        Some(ctx.load_texture(name, color_image, Default::default()))
    }

    // Get a texture handle for an image
    pub fn get_image_texture(
        ctx: &egui::Context,
        name: &str,
        path: &str,
    ) -> Option<egui::TextureHandle> {
        load_image(path).and_then(|image_data| {
            image::load_from_memory(&image_data).ok().map(|image| {
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                ctx.load_texture(name, color_image, Default::default())
            })
        })
    }
}

impl BitVaultApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Attempt to configure a font with good Unicode support
        let mut fonts = egui::FontDefinitions::default();

        // Try to add Noto Sans which has good Unicode character support
        if let Some(font_data) = assets::load_font("NotoSans-Regular.ttf") {
            log::info!("Successfully loaded Noto Sans font");

            // Add font data
            fonts
                .font_data
                .insert("noto".to_owned(), egui::FontData::from_owned(font_data));

            // Set as primary font
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "noto".to_owned());

            // Apply the font configuration
            cc.egui_ctx.set_fonts(fonts);
            log::info!("Applied custom font configuration");
        } else {
            log::warn!("Could not load Noto Sans font - using default fonts");
        }

        // Check for testing mode environment variable
        let testing_mode = std::env::var("TESTING").unwrap_or_default() == "1";
        if testing_mode {
            log::info!("Running in TESTING mode - lock screen will be bypassed");
        }

        // Load settings or use defaults
        let settings = Settings::load();

        // Create the app with default state
        let app = Self {
            state: Arc::new(RwLock::new(AppState {
                current_view: View::SplashScreen,
                splash_timer: Some(1.0), // 1 second splash screen
                testing_mode,
                ..Default::default()
            })),
            settings,
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

            // Read current state values once
            let (mut pin_input, mut pin_confirm) = if let Ok(state) = self.state.read() {
                (state.pin_input.clone(), state.pin_confirm.clone())
            } else {
                (String::new(), String::new())
            };

            // PIN input fields
            ui.horizontal(|ui| {
                ui.label("PIN: ");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut pin_input)
                        .password(true)
                        .hint_text("Enter PIN")
                        .desired_width(200.0),
                );

                // Update state with new input if changed
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

                // Update state with new input if changed
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
                && pin_valid
            {
                if let Ok(mut state) = self.state.write() {
                    // Store the PIN
                    state.user_pin = Some(pin_input);
                    log::info!("PIN set successfully");

                    // Clear the input fields for security
                    state.pin_input.clear();
                    state.pin_confirm.clear();

                    // Move to the next step
                    if state.wallet_state == WalletState::Creating {
                        log::info!("Moving to Seed view for new wallet creation");
                        state.current_view = View::Seed;
                    } else if state.wallet_state == WalletState::Restoring {
                        log::info!("Moving to Seed view for wallet restoration");
                        state.current_view = View::Seed;
                    }
                }
            }

            // Back button with simpler structure
            let back_response = ui.button("Go Back");
            if back_response.clicked() {
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
            let (original_seed, mut verification_input) = if let Ok(state) = self.state.read() {
                (
                    state.seed_phrase.clone().unwrap_or_default(),
                    state.verification_input.clone(),
                )
            } else {
                (String::new(), String::new())
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

            // Verify button
            if ui.button("Verify").clicked() && is_correct {
                if let Ok(mut state) = self.state.write() {
                    log::info!("Seed verification successful, moving to wallet view");
                    state.current_view = View::Wallet;
                    state.wallet_state = WalletState::Unlocked;
                }
            }

            // Back button with simpler structure
            let back_button = ui.button("Go Back");
            if back_button.clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::Seed;
                }
            }

            // Draw back button icon
            crate::icons::draw_caret_left(ui, back_button.rect, Color32::WHITE);
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
            // Use a static texture handle to avoid reloading on every frame
            static TEXTURE_ID: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

            let texture_id = TEXTURE_ID.get_or_init(|| {
                log::debug!("Loading splash logo - this should only happen once");
                assets::get_image_texture(ui.ctx(), "splash_logo", "public/splash_logo.png")
            });

            match texture_id {
                Some(texture) => {
                    // Get texture size and available space
                    let available_size = ui.available_size();
                    let texture_size = texture.size_vec2();

                    // Calculate appropriate scale - use a smaller maximum to prevent oversizing
                    // Use a target width of 50-60% of screen width, but never larger than original
                    let target_width_ratio = 0.5;
                    let desired_width = available_size.x * target_width_ratio;
                    let scale = (desired_width / texture_size.x).min(1.0);

                    let display_size = texture_size * scale;

                    // Ensure vertical centering by adjusting spacing
                    let vertical_center_offset = (available_size.y - display_size.y) / 2.0;
                    ui.add_space(vertical_center_offset);

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

            // Illustration frame - use SVG
            ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
                ui.vertical_centered(|ui| {
                    // Load and display the SVG image
                    static TEXTURE_ID: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

                    let texture = TEXTURE_ID.get_or_init(|| {
                        log::debug!("Loading onboarding1.svg - this should only happen once");
                        assets::load_svg_as_texture(ui.ctx(), "onboarding1", "assets/onboarding1.svg")
                    });

                    if let Some(texture) = texture {
                        // Get texture size and available space
                        let available_size = ui.available_size();
                        let texture_size = texture.size_vec2();

                        // Scale to fit within the available space while preserving aspect ratio
                        let scale = (available_size.x / texture_size.x)
                            .min(available_size.y / texture_size.y)
                            .min(1.0); // Don't scale up if image is smaller

                        let display_size = texture_size * scale;

                        ui.add_space((available_size.y - display_size.y) / 2.0); // Center vertically
                        ui.add(egui::Image::new(texture).fit_to_original_size(scale));
                    } else {
                        ui.colored_label(Color32::RED, "Failed to load SVG image");

                        // Fallback to drawn elements if SVG fails to load
                        ui.add_space(20.0);

                        // Draw a shield with keys icon (for multisig)
                        let center = ui.available_rect_before_wrap().center();
                        let shield_size = 120.0;

                        // Shield background
                        ui.painter().circle_filled(
                            center,
                            shield_size/2.0,
                            Color32::from_rgb(240, 240, 240),
                        );

                        // Shield border
                        ui.painter().circle_stroke(
                            center,
                            shield_size/2.0 + 1.0,
                            egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
                        );

                        // Draw three key symbols
                        let key_color = Color32::from_rgb(50, 50, 50);
                        let key_spacing = shield_size * 0.3;

                        // Draw three symbolic keys
                        for i in -1..=1 {
                            let key_center = center + egui::vec2(i as f32 * key_spacing, 0.0);

                            // Key head (circle)
                            ui.painter().circle_filled(
                                key_center - egui::vec2(0.0, shield_size * 0.15),
                                shield_size * 0.08,
                                key_color,
                            );

                            // Key shaft
                            let shaft_rect = egui::Rect::from_min_size(
                                key_center + egui::vec2(-shield_size * 0.03, -shield_size * 0.05),
                                egui::vec2(shield_size * 0.06, shield_size * 0.25),
                            );

                            ui.painter().rect_filled(
                                shaft_rect,
                                2.0,
                                key_color,
                            );

                            // Key teeth
                            let teeth_top = key_center.y + shield_size * 0.08;
                            let teeth_width = shield_size * 0.04;
                            let teeth_height = shield_size * 0.06;

                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(key_center.x - teeth_width/2.0, teeth_top),
                                    egui::vec2(teeth_width, teeth_height)
                                ),
                                1.0,
                                key_color,
                            );
                        }
                    }
                });
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
                .frame(false)
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
                RichText::new("Tip: Use Left/Right arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0)
            );
        });
    }

    fn render_onboarding_two(&self, ui: &mut Ui) {
        self.render_onboarding_container(ui, |ui| {
            // Upper panel with illustration
            ui.add_space(80.0); // Status bar + top spacing

            // Illustration frame - use SVG
            ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
                ui.vertical_centered(|ui| {
                    // Load and display the SVG image
                    static TEXTURE_ID: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

                    let texture = TEXTURE_ID.get_or_init(|| {
                        log::debug!("Loading onboarding2.svg - this should only happen once");
                        assets::load_svg_as_texture(ui.ctx(), "onboarding2", "assets/onboarding2.svg")
                    });

                    if let Some(texture) = texture {
                        // Get texture size and available space
                        let available_size = ui.available_size();
                        let texture_size = texture.size_vec2();

                        // Scale to fit within the available space while preserving aspect ratio
                        let scale = (available_size.x / texture_size.x)
                            .min(available_size.y / texture_size.y)
                            .min(1.0); // Don't scale up if image is smaller

                        let display_size = texture_size * scale;

                        ui.add_space((available_size.y - display_size.y) / 2.0); // Center vertically
                        ui.add(egui::Image::new(texture).fit_to_original_size(scale));
                    } else {
                        ui.colored_label(Color32::RED, "Failed to load SVG image");

                        // Fallback to drawn elements if SVG fails to load
                        ui.add_space(20.0);

                        // Draw a clock (for time delay)
                        let center = ui.available_rect_before_wrap().center();
                        let clock_size = 120.0;

                        // Clock face
                        ui.painter().circle_filled(
                            center,
                            clock_size/2.0,
                            Color32::from_rgb(240, 240, 240),
                        );

                        // Clock border
                        ui.painter().circle_stroke(
                            center,
                            clock_size/2.0,
                            egui::Stroke::new(2.0, Color32::from_rgb(50, 50, 50)),
                        );

                        // Clock hands
                        let hour_hand = center + egui::vec2(0.0, -clock_size * 0.25);
                        let minute_hand = center + egui::vec2(clock_size * 0.3, 0.0);

                        ui.painter().line_segment(
                            [center, hour_hand],
                            egui::Stroke::new(3.0, Color32::BLACK),
                        );

                        ui.painter().line_segment(
                            [center, minute_hand],
                            egui::Stroke::new(3.0, Color32::BLACK),
                        );

                        // Clock center dot
                        ui.painter().circle_filled(
                            center,
                            4.0,
                            Color32::BLACK,
                        );
                    }
                });
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
                .frame(false)
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
                RichText::new("Tip: Use Left/Right arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0)
            );
        });
    }

    fn render_onboarding_three(&self, ui: &mut Ui) {
        self.render_onboarding_container(ui, |ui| {
            // Upper panel with illustration
            ui.add_space(80.0); // Status bar + top spacing

            // Illustration frame - use SVG
            ui.allocate_ui(egui::vec2(328.0, 249.0), |ui| {
                ui.vertical_centered(|ui| {
                    // Load and display the SVG image
                    static TEXTURE_ID: OnceLock<Option<egui::TextureHandle>> = OnceLock::new();

                    let texture = TEXTURE_ID.get_or_init(|| {
                        log::debug!("Loading onboarding3.svg - this should only happen once");
                        assets::load_svg_as_texture(ui.ctx(), "onboarding3", "assets/onboarding3.svg")
                    });

                    if let Some(texture) = texture {
                        // Get texture size and available space
                        let available_size = ui.available_size();
                        let texture_size = texture.size_vec2();

                        // Scale to fit within the available space while preserving aspect ratio
                        let scale = (available_size.x / texture_size.x)
                            .min(available_size.y / texture_size.y)
                            .min(1.0); // Don't scale up if image is smaller

                        let display_size = texture_size * scale;

                        ui.add_space((available_size.y - display_size.y) / 2.0); // Center vertically
                        ui.add(egui::Image::new(texture).fit_to_original_size(scale));
                    } else {
                        ui.colored_label(Color32::RED, "Failed to load SVG image");

                        // Fallback to the shield rendering if SVG fails
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
                    }
                });
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
                .frame(false)
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
                RichText::new("Tip: Use Left/Right arrow keys to navigate")
                .color(Color32::from_rgb(150, 150, 150))
                .size(10.0)
            );
        });
    }

    // Helper function to draw arrow navigation indicators
    fn draw_navigation_arrows(&self, ui: &mut Ui, screen_number: usize) {
        // Available width needed for centering calculation
        let available_width = ui.available_width();

        // Create fixed-width dots
        let active_width = 15.0;
        let inactive_width = 5.0;
        let dot_height = 4.0; // Slightly thicker for better visibility while still bead-like
        let dot_spacing = 3.0;
        let click_padding = 12.0; // Larger click area padding for better usability

        // Calculate total width of all dots
        let total_dot_width = match screen_number {
            1 => active_width + 2.0 * inactive_width + 2.0 * dot_spacing,
            2 => inactive_width + active_width + inactive_width + 2.0 * dot_spacing,
            3 => 2.0 * inactive_width + active_width + 2.0 * dot_spacing,
            _ => active_width + 2.0 * inactive_width + 2.0 * dot_spacing,
        };

        // Add space for centering
        let left_padding = (available_width - total_dot_width) / 2.0;

        ui.horizontal(|ui| {
            ui.add_space(left_padding);

            // Create a container for our dots with extra height for easier clicking
            let response = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(total_dot_width, dot_height + click_padding),
                ),
                egui::Sense::click(), // Make the entire area clickable
            );

            // Draw the dots directly using the painter
            let painter = ui.painter();
            let mut current_x = response.rect.min.x;
            let center_y = response.rect.center().y;

            // Store click positions for later processing
            let mut click_areas = Vec::new();

            // Draw all dots
            for i in 1..=3 {
                if i > 1 {
                    current_x += dot_spacing;
                }

                // Determine dot properties based on state
                let (width, color) = if i == screen_number {
                    (active_width, Color32::from_rgb(17, 165, 238))
                } else {
                    (inactive_width, Color32::from_rgb(217, 217, 217))
                };

                // Calculate the dot rectangle
                let dot_rect = egui::Rect::from_min_size(
                    egui::pos2(current_x, center_y - dot_height / 2.0),
                    egui::vec2(width, dot_height),
                );

                // Draw the dot
                painter.rect_filled(dot_rect, dot_height / 2.0, color);

                // Store click area if this is an inactive dot
                if i != screen_number {
                    // Create a larger clickable area
                    let click_rect = egui::Rect::from_min_max(
                        egui::pos2(current_x - 2.0, center_y - (click_padding / 2.0)),
                        egui::pos2(current_x + width + 2.0, center_y + (click_padding / 2.0)),
                    );

                    click_areas.push((click_rect, i));
                }

                // Move to the next dot position
                current_x += width;
            }

            // Handle clicks for navigation
            if response.clicked() {
                if let Some(mouse_pos) = ui.ctx().pointer_latest_pos() {
                    // Handle clicks directly on dots
                    for (rect, idx) in click_areas {
                        if rect.contains(mouse_pos) {
                            if let Ok(mut state) = self.state.write() {
                                state.current_view = match idx {
                                    1 => View::OnboardingOne,
                                    2 => View::OnboardingTwo,
                                    3 => View::OnboardingThree,
                                    _ => View::OnboardingOne,
                                };
                            }
                            break;
                        }
                    }
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
        // Check for window resize events and save the new size
        let screen_rect = ctx.input(|i| i.screen_rect);
        let size = screen_rect.size();
        if size.x != self.settings.window_width || size.y != self.settings.window_height {
            self.settings.update_window_size(size.x, size.y);
        }

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
