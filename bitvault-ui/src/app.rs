#![allow(non_snake_case)]
use std::sync::Arc;

use eframe::{egui, CreationContext};
use egui::{Color32, Context, RichText, Ui};
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
    // Unlocked,
    // Locked,
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
    pub selected_words: Vec<String>,
    pub verification_input: String,
    pub copied_feedback: Option<f32>, // Timer for showing copy feedback (in seconds)
}

// Create a type alias for a thread-safe, shared reference to the state
pub type SharedAppState = Arc<RwLock<AppState>>;

pub struct BitVaultApp {
    state: SharedAppState,
}

impl BitVaultApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState::default())),
        }
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

            let mut back_button_response = None;

            if let Ok(mut state) = self.state.write() {
                ui.horizontal(|ui| {
                    ui.label("PIN: ");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.pin_input)
                            .password(true)
                            .hint_text("Enter PIN")
                            .desired_width(200.0),
                    );
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Confirm PIN: ");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.pin_confirm)
                            .password(true)
                            .hint_text("Confirm PIN")
                            .desired_width(200.0),
                    );
                });

                ui.add_space(20.0);

                let pin_valid = !state.pin_input.is_empty() && state.pin_input == state.pin_confirm;

                if ui
                    .add_enabled(pin_valid, egui::Button::new("Continue"))
                    .clicked()
                {
                    state.user_pin = Some(state.pin_input.clone());

                    if state.wallet_state == WalletState::Creating {
                        // Generate a new seed phrase using wallet module
                        match wallet::new_12_word_seed() {
                            Ok(seed_phrase) => {
                                state.seed_phrase = Some(seed_phrase);
                                state.current_view = View::Seed;
                            }
                            Err(e) => {
                                // In a real app, handle this error properly
                                log::error!("Failed to generate seed phrase: {}", e);
                            }
                        }
                    } else {
                        // For restoration flow
                        state.current_view = View::Seed;
                    }
                }

                back_button_response = Some(ui.add(egui::Button::new("Go Back")));
                if back_button_response.as_ref().unwrap().clicked() {
                    state.current_view = View::Disclaimer;
                }
            }

            if let Some(response) = back_button_response {
                crate::icons::draw_caret_left(ui, response.rect, Color32::WHITE);
            }
        });
    }

    fn render_seed(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            // First, read the state to determine what to display
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

                        // Set the feedback timer
                        if let Ok(mut state) = self.state.write() {
                            state.copied_feedback = Some(2.0); // Show feedback for 2 seconds
                        }
                    }

                    ui.add_space(10.0);

                    if ui.button("I've Written Down My Recovery Phrase").clicked() {
                        if let Ok(mut state) = self.state.write() {
                            // Prepare for verification
                            if let Some(seed) = &state.seed_phrase {
                                let words: Vec<String> =
                                    seed.split_whitespace().map(String::from).collect();
                                state.seed_words = words;
                                state.selected_words = Vec::new();
                                state.verification_input = String::new();
                                state.current_view = View::SeedVerify;
                            }
                        }
                    }
                }
            } else {
                // Restoration flow
                ui.heading("Restore from Recovery Phrase");
                ui.add_space(10.0);
                ui.label("Enter your 12-word recovery phrase:");

                if let Ok(mut state) = self.state.write() {
                    let mut seed_input = state.seed_phrase.clone().unwrap_or_default();

                    ui.add(
                        egui::TextEdit::multiline(&mut seed_input)
                            .hint_text("Enter your 12 words separated by spaces")
                            .desired_width(400.0)
                            .desired_rows(3),
                    );

                    state.seed_phrase = Some(seed_input);

                    ui.add_space(20.0);

                    let is_valid = state.seed_phrase.as_ref().is_some_and(|phrase| {
                        let words = phrase.split_whitespace().count();
                        words == 12 || words == 24
                    });

                    if ui
                        .add_enabled(is_valid, egui::Button::new("Restore Wallet"))
                        .clicked()
                    {
                        // In a real app, validate the seed phrase here
                        state.current_view = View::Wallet;
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

            if let Ok(mut state) = self.state.write() {
                let original_seed = state.seed_phrase.clone().unwrap_or_default();

                ui.add_space(20.0);

                // Text input for verification
                ui.add(
                    egui::TextEdit::multiline(&mut state.verification_input)
                        .hint_text("Enter your 12 words in order, separated by spaces")
                        .desired_width(400.0)
                        .desired_rows(3),
                );

                ui.add_space(20.0);

                // Check if the entered text matches the original seed phrase
                let is_correct = state.verification_input.trim() == original_seed.trim();

                if !state.verification_input.is_empty() {
                    if is_correct {
                        ui.label(RichText::new("✓ Correct!").color(Color32::GREEN));
                    } else {
                        ui.label(
                            RichText::new("✗ Incorrect. Please try again.").color(Color32::RED),
                        );
                    }
                }

                ui.add_space(10.0);

                if ui
                    .add_enabled(is_correct, egui::Button::new("Confirm"))
                    .clicked()
                {
                    state.current_view = View::Wallet;
                }

                let back_button_response = ui.add(egui::Button::new("Go Back"));
                if back_button_response.clicked() {
                    state.current_view = View::Seed;
                }

                crate::icons::draw_caret_left(ui, back_button_response.rect, Color32::WHITE);
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

            if ui.button("Lock Wallet").clicked() {
                if let Ok(mut state) = self.state.write() {
                    state.current_view = View::Home;
                    state.wallet_state = WalletState::New;
                    state.user_pin = None;
                    state.pin_input.clear();
                    state.pin_confirm.clear();
                    state.seed_phrase = None;
                    state.seed_words.clear();
                    state.selected_words.clear();
                }
            }
        });
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
            }
        });
    }
}
