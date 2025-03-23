use crate::app::state::{View, WalletState};
use crate::app::BitVaultApp;
use crate::wallet;
use eframe::egui::{self, Color32, RichText, Ui};

// PIN choice screen
pub fn render_pin_choice(app: &BitVaultApp, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Create a PIN");
        ui.add_space(20.0);

        ui.label("Choose a secure PIN to protect your wallet");
        ui.add_space(10.0);

        // Read current state values once
        let (mut pin_input, mut pin_confirm) = if let Ok(state) = app.state.read() {
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
                if let Ok(mut state) = app.state.write() {
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
                if let Ok(mut state) = app.state.write() {
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
            if let Ok(mut state) = app.state.write() {
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
            if let Ok(mut state) = app.state.write() {
                state.current_view = View::Disclaimer;
            }
        }

        // Draw the back button icon
        crate::icons::draw_caret_left(ui, back_response.rect, Color32::WHITE);
    });
}

// Seed phrase screen
pub fn render_seed(app: &BitVaultApp, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        // Check if we're in creating or restoring mode
        let (is_creating, has_seed, _current_seed) = if let Ok(state) = app.state.read() {
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
                    if let Ok(mut state) = app.state.write() {
                        state.seed_phrase = Some(new_seed.clone());
                        // Split the seed phrase into words for the verification step
                        state.seed_words = new_seed.split_whitespace().map(String::from).collect();
                    }
                }
                Err(e) => {
                    log::error!("Failed to generate seed phrase: {}", e);
                }
            }
        }

        // Re-read state after potential updates
        let (is_creating, seed_phrase, has_feedback) = if let Ok(state) = app.state.read() {
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
                        if let Ok(mut state) = app.state.write() {
                            state.copied_feedback = Some(2.0); // Show feedback for 2 seconds
                        }
                    }

                    ui.add_space(20.0);

                    if ui.button("Continue").clicked() {
                        if let Ok(mut state) = app.state.write() {
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

            if let Ok(mut state) = app.state.write() {
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
            if let Ok(mut state) = app.state.write() {
                state.current_view = View::PinChoice;
            }
        }
        crate::icons::draw_caret_left(ui, back_button_response.rect, Color32::WHITE);
    });
}

// Seed verification screen
pub fn render_seed_verify(app: &BitVaultApp, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Verify Recovery Phrase");
        ui.add_space(10.0);
        ui.label("Please enter your recovery phrase to verify you've written it down correctly:");

        // Read state once to get the values we need
        let (original_seed, mut verification_input) = if let Ok(state) = app.state.read() {
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
            if let Ok(mut state) = app.state.write() {
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
            if let Ok(mut state) = app.state.write() {
                log::info!("Seed verification successful, moving to wallet view");
                state.current_view = View::Wallet;
                state.wallet_state = WalletState::Unlocked;
            }
        }

        // Back button with simpler structure
        let back_button = ui.button("Go Back");
        if back_button.clicked() {
            if let Ok(mut state) = app.state.write() {
                state.current_view = View::Seed;
            }
        }

        // Draw back button icon
        crate::icons::draw_caret_left(ui, back_button.rect, Color32::WHITE);
    });
}
