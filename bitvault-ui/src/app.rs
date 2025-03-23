#![allow(non_snake_case)]
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Result;
use eframe::{
    egui::{self, Context},
    CreationContext,
};
use serde::{Deserialize, Serialize};

use crate::config::Settings;

pub mod assets;
pub mod screens;
pub mod state;

use self::state::{AppState, SharedAppState, View, WalletState};

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

pub struct BitVaultApp {
    state: SharedAppState,
    settings: Settings,
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
    pub fn save_wallet_to_disk(&self, encrypted_data: &str) -> Result<(), String> {
        if let Some(file_path) = Self::get_wallet_file_path() {
            fs::write(file_path, encrypted_data)
                .map_err(|e| format!("Failed to save wallet: {}", e))
        } else {
            Err("Could not determine wallet file path".to_string())
        }
    }

    // Load wallet data from disk
    pub fn load_wallet_from_disk(&self) -> Result<String, String> {
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
                        state.current_view = View::OnboardingOne;
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
                View::Home => screens::home::render(self, ui),
                View::Disclaimer => screens::home::render_disclaimer(self, ui),
                View::PinChoice => screens::seed::render_pin_choice(self, ui),
                View::Seed => screens::seed::render_seed(self, ui),
                View::SeedVerify => screens::seed::render_seed_verify(self, ui),
                View::Wallet => screens::wallet::render(self, ui),
                View::LockScreen => screens::lock::render(self, ui),
                View::SplashScreen => screens::home::render_splash_screen(ui, &self.state),
                View::OnboardingOne => screens::onboarding::render_one(ui, &self.state),
                View::OnboardingTwo => screens::onboarding::render_two(ui, &self.state),
                View::OnboardingThree => screens::onboarding::render_three(ui, &self.state),
            }
        });
    }
}
