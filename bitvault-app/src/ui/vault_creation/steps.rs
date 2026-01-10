//! Vault creation step implementations

use crate::state::{AppState, Navigation, View};
use crate::ui::components::{button_large, Spacing};
use crate::ui::pin::render_pin_setup;
use crate::ui::vault_creation::{DeviceRole, VaultCreationState, VaultCreationStep};
use crate::utils::icons::{Icon, icon_image};
use bip39::{Language, Mnemonic};
use bitvault_common::utils::TimeDelay;
use eframe::egui;

/// Role selection step - first step in vault creation
pub fn render_role_selection(
    ui: &mut egui::Ui,
    state: &mut VaultCreationState,
    navigation: &mut Navigation,
) {
    ui.vertical_centered(|ui| {
        ui.heading("Set Up Vault");
        ui.add_space(Spacing::MD);

        ui.label("How would you like to set up your vault?");
        ui.add_space(Spacing::LG);
    });

    let icon_color = ui.style().visuals.text_color();
    let icon_size = 20.0;
    let card_width = 280.0;
    let card_height = 120.0;
    let row_width = card_width * 2.0 + Spacing::MD;

    // Center the grid
    let available_width = ui.available_width();
    let left_margin = ((available_width - row_width) / 2.0).max(0.0);

    // Row 1: View-Only and Create New
    ui.horizontal(|ui| {
        ui.add_space(left_margin);
        
        // Option 1: View-Only Mode
        ui.group(|ui| {
            ui.set_min_size(egui::vec2(card_width, card_height));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(img) = icon_image(ui.ctx(), Icon::Import, icon_size, icon_color) {
                        ui.add(img);
                    }
                    ui.strong("View-Only Mode");
                });
                ui.add_space(Spacing::SM);
                ui.label("Monitor without signing.");
                ui.add_space(Spacing::MD);
                if ui.button("Set Up View-Only").clicked() {
                    state.device_role = DeviceRole::ViewOnly;
                    state.advance_to_step(VaultCreationStep::NameVault);
                }
            });
        });

        ui.add_space(Spacing::MD);

        // Option 2: Create New Vault
        ui.group(|ui| {
            ui.set_min_size(egui::vec2(card_width, card_height));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(img) = icon_image(ui.ctx(), Icon::Plus, icon_size, icon_color) {
                        ui.add(img);
                    }
                    ui.strong("Create New Vault");
                });
                ui.add_space(Spacing::SM);
                ui.label("Start a new vault.");
                ui.add_space(Spacing::MD);
                if ui.button("Create New Vault").clicked() {
                    state.device_role = DeviceRole::Main;
                    state.advance_to_step(VaultCreationStep::NameVault);
                }
            });
        });
    });

    ui.add_space(Spacing::MD);

    // Row 2: Join as Co-owner and Restore
    ui.horizontal(|ui| {
        ui.add_space(left_margin);
        
        // Option 3: Join as Co-owner
        ui.group(|ui| {
            ui.set_min_size(egui::vec2(card_width, card_height));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(img) = icon_image(ui.ctx(), Icon::Users, icon_size, icon_color) {
                        ui.add(img);
                    }
                    ui.strong("Join as Co-owner");
                });
                ui.add_space(Spacing::SM);
                ui.label("Pair with main device.");
                ui.add_space(Spacing::MD);
                if ui.button("Join as Co-owner").clicked() {
                    state.device_role = DeviceRole::Coowner;
                    state.advance_to_step(VaultCreationStep::NameVault);
                }
            });
        });

        ui.add_space(Spacing::MD);

        // Option 4: Restore from Backup
        ui.group(|ui| {
            ui.set_min_size(egui::vec2(card_width, card_height));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(img) = icon_image(ui.ctx(), Icon::Back, icon_size, icon_color) {
                        ui.add(img);
                    }
                    ui.strong("Restore from Backup");
                });
                ui.add_space(Spacing::SM);
                ui.colored_label(egui::Color32::YELLOW, "Requires paper backup.");
                ui.add_space(Spacing::MD);
                if ui.button("Restore from Backup").clicked() {
                    state.device_role = DeviceRole::Restore;
                    state.advance_to_step(VaultCreationStep::NameVault);
                }
            });
        });
    });

    ui.add_space(Spacing::XL);

    ui.vertical_centered(|ui| {
        if ui.button("Cancel").clicked() {
            navigation.go_back();
        }
    });
}

/// Name vault step
pub fn render_name_vault(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    let role_text = match state.device_role {
        DeviceRole::Main => "Main Device Setup",
        DeviceRole::Coowner => "Co-owner Setup",
        DeviceRole::ViewOnly => "View-Only Setup",
        DeviceRole::Restore => "Restore from Backup",
    };
    
    ui.vertical_centered(|ui| {
        ui.heading(role_text);
        ui.add_space(Spacing::LG);

        ui.label("Enter a name for this vault:");
        ui.add_space(Spacing::MD);

        ui.add(
            egui::TextEdit::singleline(&mut state.vault_name)
                .hint_text("My Bitcoin Vault")
                .desired_width(300.0)
                .margin(egui::vec2(8.0, 6.0))
        );

        if let Some(ref error) = state.error {
            ui.add_space(Spacing::SM);
            ui.colored_label(egui::Color32::RED, error);
        }

        ui.add_space(Spacing::XL);

        if button_large(ui, "Continue").clicked() {
            if state.vault_name.trim().is_empty() {
                state.error = Some("Please enter a vault name".to_string());
            } else {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
        }

        ui.add_space(Spacing::MD);
        if ui.button("← Back").clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Set time delay (main device only)
pub fn render_set_time_delay(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.vertical_centered(|ui| {
        ui.heading("Set Time Delay");
        ui.add_space(Spacing::MD);

        ui.label("Choose how long before the fast-path becomes available.");
        ui.label("This is your security buffer if your device is compromised.");
        ui.add_space(Spacing::LG);

        // Days slider in a centered group
        ui.label("Days:");
        ui.add_space(4.0);
        ui.group(|ui| {
            ui.set_width(400.0);
            ui.style_mut().spacing.slider_width = 380.0;
            ui.add(egui::Slider::new(&mut state.time_delay_days, 0..=365));
        });

        ui.add_space(Spacing::MD);

        // Hours slider in a centered group
        ui.label("Hours:");
        ui.add_space(4.0);
        ui.group(|ui| {
            ui.set_width(400.0);
            ui.style_mut().spacing.slider_width = 380.0;
            ui.add(egui::Slider::new(&mut state.time_delay_hours, 0..=23));
        });

        ui.add_space(Spacing::MD);

        let total_hours = (state.time_delay_days * 24 + state.time_delay_hours) as f32;
        if total_hours < 24.0 {
            ui.colored_label(egui::Color32::YELLOW, "⚠ A time delay of at least 24 hours is recommended.");
        }

        ui.add_space(Spacing::XL);

        if button_large(ui, "Continue").clicked() {
            if let Some(next) = state.next_step_for_role() {
                state.advance_to_step(next);
            }
        }

        ui.add_space(Spacing::MD);
        if ui.button("← Back").clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Generate mnemonic step
pub fn render_mnemonic_generation(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.vertical_centered(|ui| {
        ui.heading("Create Seed Phrase");
        ui.add_space(Spacing::LG);

        ui.label("Your seed phrase is the key to your vault.");
        ui.label("It will be generated securely on this device.");
        ui.add_space(Spacing::XL);

        if button_large(ui, "Generate Seed Phrase").clicked() {
            use rand::RngCore;
            let mut entropy = [0u8; 16]; // 128 bits for 12 words
            rand::thread_rng().fill_bytes(&mut entropy);
            
            match Mnemonic::from_entropy_in(Language::English, &entropy) {
                Ok(mnemonic) => {
                    state.mnemonic = Some(mnemonic);
                    state.error = None;
                    state.advance_to_step(VaultCreationStep::DisplaySeedPhrase);
                }
                Err(e) => {
                    state.error = Some(format!("Failed to generate mnemonic: {}", e));
                }
            }
        }

        if let Some(ref error) = state.error {
            ui.add_space(Spacing::SM);
            ui.colored_label(egui::Color32::RED, error);
        }

        ui.add_space(Spacing::MD);
        if ui.button("← Back").clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Display seed phrase
pub fn render_display_seed_phrase(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.vertical_centered(|ui| {
        ui.heading("Write Down Your Seed Phrase");
        ui.add_space(Spacing::MD);

        ui.colored_label(egui::Color32::RED, "⚠ IMPORTANT: Write this down on paper!");
        ui.label("Never share it. Never store it digitally. You need this to recover your vault.");
        ui.add_space(Spacing::LG);
    });

    // Center the seed phrase card
    if let Some(ref mnemonic) = state.mnemonic {
        let words: Vec<&str> = mnemonic.words().collect();
        let card_width = 380.0;
        let available_width = ui.available_width();
        let left_margin = ((available_width - card_width) / 2.0).max(0.0);

        ui.horizontal(|ui| {
            ui.add_space(left_margin);
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(20.0))
                .show(ui, |ui| {
                    ui.set_width(card_width - 40.0); // Account for margins
                    egui::Grid::new("seed_words")
                        .num_columns(3)
                        .spacing([28.0, 14.0])
                        .show(ui, |ui| {
                            for (i, word) in words.iter().enumerate() {
                                ui.monospace(format!("{:2}. {}", i + 1, word));
                                if (i + 1) % 3 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
    }

    ui.add_space(Spacing::XL);

    ui.vertical_centered(|ui| {
        if button_large(ui, "I've Written It Down").clicked() {
            state.advance_to_step(VaultCreationStep::VerifySeedPhrase);
        }

        ui.add_space(Spacing::MD);
        if ui.button("← Back").clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Verify seed phrase
pub fn render_verify_seed_phrase(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.vertical_centered(|ui| {
        ui.heading("Verify Seed Phrase");
        ui.add_space(Spacing::LG);

        ui.label("Please confirm you have written down your seed phrase.");
        ui.add_space(Spacing::MD);

        ui.checkbox(&mut state.verified_seed_phrase, "I have written down my seed phrase and stored it securely");

        if let Some(ref error) = state.error {
            ui.add_space(Spacing::SM);
            ui.colored_label(egui::Color32::RED, error);
        }

        ui.add_space(Spacing::XL);

        if button_large(ui, "Continue").clicked() {
            if state.verified_seed_phrase {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            } else {
                state.error = Some("Please confirm you have written down your seed phrase".to_string());
            }
        }

        ui.add_space(Spacing::MD);
        if ui.button("← Back").clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Set PIN step
pub fn render_set_pin(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    _navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.vertical_centered(|ui| {
        ui.heading("Set PIN");
        ui.add_space(Spacing::MD);
        ui.label("Set a 6-digit PIN to secure your wallet");
        ui.add_space(Spacing::LG);
    });

    let mut callback = None;
    let pin_set = render_pin_setup(ui, &mut state.pin_setup_state, &mut callback);

    if pin_set {
        if let Some(next) = state.next_step_for_role() {
            state.advance_to_step(next);
        }
    }

    ui.add_space(Spacing::MD);
    ui.vertical_centered(|ui| {
        if ui.button("← Back").clicked() {
            state.go_to_previous_step();
        }
    });
}

/// Main device: Scan/enter co-owner's keys
pub fn render_scan_coowner_keys(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.heading("Get Co-owner's Keys");
    ui.add_space(Spacing::MD);

    ui.label("Ask your co-owner to share their public keys with you.");
    ui.label("They can copy the text from their device and send it to you securely.");
    ui.add_space(Spacing::LG);

    ui.label("Paste co-owner's key data:");
    ui.add_space(Spacing::SM);
    
    ui.add(egui::TextEdit::multiline(&mut state.coowner_pubkeys)
        .hint_text("Paste the key data here...")
        .desired_width(400.0)
        .desired_rows(4));

    ui.add_space(Spacing::MD);

    // Or load from file
    if ui.button("Load from File").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt", "json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    state.coowner_pubkeys = contents.trim().to_string();
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to read file: {}", e));
                }
            }
        }
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Continue").clicked() {
        if state.coowner_pubkeys.trim().is_empty() {
            state.error = Some("Please paste or load the co-owner's key data".to_string());
        } else {
            // Validate the keys by decoding as CoownerKeys
            match bitvault_common::ur::decode_qr_data::<bitvault_common::derivation::CoownerKeys>(&state.coowner_pubkeys) {
                Ok(keys) => {
                    state.coowner_keys = Some(keys);
                    state.error = None;
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Invalid key data: {}", e));
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.go_to_previous_step();
    }
}

/// Co-owner device: Display own keys for main device
pub fn render_display_own_keys(ui: &mut egui::Ui, ctx: &egui::Context, state: &mut VaultCreationState) {
    ui.heading("Share Your Keys");
    ui.add_space(Spacing::MD);

    ui.label("Share this key data with the main device owner.");
    ui.label("They will enter it on their device to link you as co-owner.");
    ui.add_space(Spacing::LG);

    // Generate keys text if not already done
    if state.my_keys_text.is_none() {
        if let Some(ref mnemonic) = state.mnemonic {
            match bitvault_common::derivation::get_owner_keys(mnemonic) {
                Ok(owner_keys) => {
                    match bitvault_common::ur::encode_qr_data(&owner_keys) {
                        Ok(keys_text) => {
                            state.my_keys_text = Some(keys_text);
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to encode keys: {}", e));
                        }
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Failed to derive keys: {}", e));
                }
            }
        }
    }

    if let Some(ref keys_text) = state.my_keys_text {
        // Show QR code
        if let Some(qr_texture) = crate::utils::qr::generate_qr_image(ctx, keys_text) {
            ui.image((qr_texture.id(), egui::Vec2::new(200.0, 200.0)));
            ui.add_space(Spacing::MD);
        }

        // Copy button
        if ui.button("Copy Key Data").clicked() {
            ui.ctx().copy_text(keys_text.clone());
        }

        ui.add_space(Spacing::SM);

        // Save to file
        if ui.button("Save to File").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("coowner_keys.txt")
                .save_file()
            {
                if let Err(e) = std::fs::write(&path, keys_text) {
                    state.error = Some(format!("Failed to save file: {}", e));
                }
            }
        }

        ui.add_space(Spacing::MD);

        // Show truncated text
        ui.collapsing("Show Key Data", |ui| {
            ui.monospace(keys_text);
        });
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "I've Shared My Keys").clicked() {
        if let Some(next) = state.next_step_for_role() {
            state.advance_to_step(next);
        }
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.go_to_previous_step();
    }
}

/// Co-owner device: Enter exchange data from main device
pub fn render_enter_exchange_data(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.heading("Enter Vault Configuration");
    ui.add_space(Spacing::MD);

    ui.label("The main device owner will share the vault configuration with you");
    ui.label("after they create the vault. Paste it below.");
    ui.add_space(Spacing::LG);

    ui.label("Paste vault configuration:");
    ui.add_space(Spacing::SM);

    ui.add(egui::TextEdit::multiline(&mut state.exchange_data_input)
        .hint_text("Paste the configuration data here...")
        .desired_width(400.0)
        .desired_rows(4));

    ui.add_space(Spacing::MD);

    if ui.button("Load from File").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt", "json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    state.exchange_data_input = contents.trim().to_string();
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to read file: {}", e));
                }
            }
        }
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Continue").clicked() {
        if state.exchange_data_input.trim().is_empty() {
            state.error = Some("Please paste or load the vault configuration".to_string());
        } else {
            // Validate the exchange data
            match bitvault_common::ur::decode_qr_data::<bitvault_common::ur::QrExchangeData>(&state.exchange_data_input) {
                Ok(exchange_data) => {
                    // Store the main device's keys
                    state.coowner_keys = Some(exchange_data.coowner_public_keys);
                    // Extract time delay from exchange data
                    let time_delay = bitvault_common::utils::blocks_to_time_delay(exchange_data.timelock_in_blocks);
                    state.time_delay_days = time_delay.days;
                    state.time_delay_hours = time_delay.hours;
                    state.error = None;
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Invalid configuration data: {}", e));
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.go_to_previous_step();
    }
}

/// Email authentication step
pub fn render_email_auth(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut VaultCreationState,
) {
    ui.heading("Email Verification");
    ui.add_space(Spacing::MD);

    ui.label("Enter your email address to verify your identity:");
    ui.add_space(Spacing::MD);

    ui.add(egui::TextEdit::singleline(&mut state.email)
        .hint_text("you@example.com")
        .desired_width(300.0));

    ui.add_space(Spacing::MD);

    if !state.code_sent {
        if ui.button("Send Verification Code").clicked() {
            if state.email.trim().is_empty() || !state.email.contains('@') {
                state.error = Some("Please enter a valid email address".to_string());
            } else {
                state.is_sending_code = true;
                state.error = None;

                if let Some(ref runtime) = app_state.runtime {
                    let email = state.email.clone();
                    let network = app_state.network;
                    let result = runtime.block_on(async {
                        let temp_service = bitvault_common::wallet::VaultService::new(network);
                        temp_service.send_email_auth_code(&email).await
                    });

                    match result {
                        Ok(_) => {
                            state.code_sent = true;
                            state.is_sending_code = false;
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to send code: {}", e));
                            state.is_sending_code = false;
                        }
                    }
                }
            }
        }

        if state.is_sending_code {
            ui.spinner();
            ui.label("Sending...");
        }
    } else {
        ui.colored_label(egui::Color32::GREEN, "✓ Code sent! Check your email.");
        ui.add_space(Spacing::MD);

        ui.label("Enter the verification code:");
        ui.add_space(Spacing::SM);

        ui.add(egui::TextEdit::singleline(&mut state.auth_code)
            .hint_text("123456")
            .desired_width(150.0));

        ui.add_space(Spacing::XL);

        if button_large(ui, "Verify & Continue").clicked() {
            if state.auth_code.trim().is_empty() {
                state.error = Some("Please enter the verification code".to_string());
            } else {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
        }
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.code_sent = false;
        state.auth_code.clear();
        state.go_to_previous_step();
    }
}

/// Create vault step
pub fn render_create_vault(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    let action_text = match state.device_role {
        DeviceRole::Main => "Create Vault",
        DeviceRole::Coowner => "Join Vault",
        DeviceRole::ViewOnly => "Set Up View-Only",
        DeviceRole::Restore => "Restore Vault",
    };

    ui.heading(action_text);
    ui.add_space(Spacing::MD);

    if state.is_creating {
        ui.spinner();
        ui.label(format!("{}...", if state.device_role == DeviceRole::Main { "Creating vault" } else { "Joining vault" }));
        return;
    }

    // Summary
    ui.label(format!("Vault Name: {}", state.vault_name));
    ui.label(format!("Time Delay: {} days, {} hours", state.time_delay_days, state.time_delay_hours));
    ui.label(format!("Email: {}", state.email));
    ui.label(format!("Role: {}", if state.device_role == DeviceRole::Main { "Main Device" } else { "Co-owner" }));

    ui.add_space(Spacing::XL);

    if button_large(ui, action_text).clicked() {
        state.is_creating = true;
        state.error = None;

        // Validation
        if state.vault_name.trim().is_empty() {
            state.error = Some("Vault name cannot be empty".to_string());
            state.is_creating = false;
            return;
        }

        if state.email.trim().is_empty() || !state.email.contains('@') {
            state.error = Some("Please enter a valid email address".to_string());
            state.is_creating = false;
            return;
        }

        if state.auth_code.trim().is_empty() {
            state.error = Some("Please enter an authentication code".to_string());
            state.is_creating = false;
            return;
        }

        if state.coowner_pubkeys.trim().is_empty() && state.coowner_keys.is_none() {
            state.error = Some("Co-owner keys are required".to_string());
            state.is_creating = false;
            return;
        }

        // Create/join vault
        if let (Some(mnemonic), Some(runtime)) = (state.mnemonic.as_ref(), app_state.runtime.as_ref()) {
            let time_delay = TimeDelay {
                days: state.time_delay_days,
                hours: state.time_delay_hours,
            };
            let coowner_pubkeys = state.coowner_pubkeys.clone();
            let vault_name = state.vault_name.clone();
            let network = app_state.network;
            let email = state.email.clone();
            let auth_code = state.auth_code.clone();
            let runtime_handle = runtime.handle().clone();

            let result = runtime.block_on(async {
                let mut vault_service = bitvault_common::wallet::VaultService::new(network);

                let qr_result = vault_service
                    .setup_vault(
                        mnemonic,
                        &coowner_pubkeys,
                        time_delay,
                        &vault_name,
                        &email,
                        &auth_code,
                    )
                    .await;

                match qr_result {
                    Ok(qr) => Ok((qr, vault_service)),
                    Err(e) => Err(e),
                }
            });

            match result {
                Ok((exchange_data, vault_service)) => {
                    state.exchange_data_output = Some(exchange_data);

                    if let Err(e) = runtime_handle.block_on(async {
                        app_state.initialize_vault_from_service(vault_service).await
                    }) {
                        state.error = Some(format!("Failed to initialize vault: {}", e));
                        state.is_creating = false;
                        return;
                    }

                    if let Some(ref mut handler) = app_state.async_handler {
                        handler.fetch_balance();
                        handler.fetch_address();
                    }

                    state.is_creating = false;
                    
                    // Main device shows exchange data, co-owner goes to completed
                    if state.device_role == DeviceRole::Main {
                        state.advance_to_step(VaultCreationStep::DisplayExchangeData);
                    } else {
                        state.advance_to_step(VaultCreationStep::Completed);
                        navigation.navigate_to(View::Dashboard { tab: 0 });
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Failed to create vault: {}", e));
                    state.is_creating = false;
                }
            }
        } else {
            state.error = Some("Missing mnemonic or runtime".to_string());
            state.is_creating = false;
        }
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.go_to_previous_step();
    }
}

/// Main device: Display exchange data for co-owner
pub fn render_display_exchange_data(ui: &mut egui::Ui, ctx: &egui::Context, state: &mut VaultCreationState) {
    ui.heading("Share with Co-owner");
    ui.add_space(Spacing::MD);

    ui.colored_label(egui::Color32::GREEN, "✓ Vault created successfully!");
    ui.add_space(Spacing::MD);

    ui.label("Share this configuration with your co-owner.");
    ui.label("They will enter it on their device to join the vault.");
    ui.add_space(Spacing::LG);

    if let Some(ref exchange_data) = state.exchange_data_output {
        // Show QR code
        if let Some(qr_texture) = crate::utils::qr::generate_qr_image(ctx, exchange_data) {
            ui.image((qr_texture.id(), egui::Vec2::new(200.0, 200.0)));
            ui.add_space(Spacing::MD);
        }

        if ui.button("Copy Configuration").clicked() {
            ui.ctx().copy_text(exchange_data.clone());
        }

        ui.add_space(Spacing::SM);

        if ui.button("Save to File").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("vault_config.txt")
                .save_file()
            {
                if let Err(e) = std::fs::write(&path, exchange_data) {
                    state.error = Some(format!("Failed to save file: {}", e));
                }
            }
        }

        ui.add_space(Spacing::MD);

        ui.collapsing("Show Configuration Data", |ui| {
            ui.monospace(exchange_data);
        });
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Go to Dashboard").clicked() {
        state.advance_to_step(VaultCreationStep::Completed);
    }
}

/// Completed step
pub fn render_completed(ui: &mut egui::Ui, navigation: &mut Navigation, state: &mut VaultCreationState) {
    ui.heading("Vault Setup Complete!");
    ui.add_space(Spacing::LG);

    ui.colored_label(egui::Color32::GREEN, "✓ Your vault is ready to use.");
    ui.add_space(Spacing::MD);

    ui.label(format!("Vault Name: {}", state.vault_name));

    if let Some(ref address) = state.vault_address {
        ui.label(format!("Vault Address: {}", address));
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Go to Dashboard").clicked() {
        navigation.navigate_to(View::Dashboard { tab: 0 });
    }
}

// ============================================================================
// VIEW-ONLY FLOW
// ============================================================================

/// Scan descriptor QR for view-only mode
pub fn render_scan_descriptor_view_only(
    ui: &mut egui::Ui,
    state: &mut VaultCreationState,
) {
    ui.heading("View-Only Setup");
    ui.add_space(Spacing::MD);

    ui.label("Scan or paste the descriptor from your mobile device.");
    ui.add_space(Spacing::SM);
    
    ui.colored_label(
        egui::Color32::from_rgb(100, 149, 237),
        "This will let you monitor your vault without signing capability."
    );
    ui.add_space(Spacing::LG);

    ui.label("Paste the descriptor configuration:");
    ui.add_space(Spacing::SM);
    ui.add(egui::TextEdit::multiline(&mut state.import_descriptors_qr)
        .hint_text("Paste configuration from mobile app...")
        .desired_width(400.0)
        .desired_rows(3));

    ui.add_space(Spacing::MD);

    // File load option
    ui.horizontal(|ui| {
        if ui.button("Load from File").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Text/JSON", &["txt", "json"])
                .pick_file()
            {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    state.import_descriptors_qr = contents;
                } else {
                    state.error = Some("Failed to read file".to_string());
                }
            }
        }
    });

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Continue").clicked() {
        if state.import_descriptors_qr.trim().is_empty() {
            state.error = Some("Please enter the descriptor configuration".to_string());
            return;
        }
        state.error = None;
        if let Some(next) = state.next_step_for_role() {
            state.advance_to_step(next);
        }
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.go_to_previous_step();
    }
}

/// View-only setup complete
pub fn render_view_only_complete(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.heading("View-Only Setup");
    ui.add_space(Spacing::MD);

    if state.is_importing {
        ui.spinner();
        ui.label("Setting up view-only wallet...");
        return;
    }

    if state.vault_address.is_some() {
        // Success
        ui.colored_label(egui::Color32::GREEN, "✓ View-only wallet created!");
        ui.add_space(Spacing::MD);
        ui.label("You can now monitor your vault balance and transactions.");
        ui.label("Signing transactions will require your mobile device.");
        ui.add_space(Spacing::XL);
        
        if button_large(ui, "Open Wallet").clicked() {
            navigation.navigate_to(View::Dashboard { tab: 0 });
        }
    } else {
        // Setup button
        ui.label("Ready to set up view-only wallet?");
        ui.add_space(Spacing::LG);

        if button_large(ui, "Create View-Only Wallet").clicked() {
            state.is_importing = true;
            state.error = None;

            if let Some(ref runtime) = app_state.runtime {
                let descriptors_qr = state.import_descriptors_qr.clone();
                let vault_name = state.vault_name.clone();
                let network = app_state.network;
                let runtime_handle = runtime.handle().clone();

                // For view-only, we use a dummy mnemonic since we don't need signing
                let entropy: [u8; 16] = rand::random();
                let dummy_mnemonic = Mnemonic::from_entropy(&entropy).expect("Failed to generate dummy mnemonic");

                let result: Result<(bitvault_common::wallet::VaultService, String), String> =
                    runtime.block_on(async {
                        let mut vault_service = bitvault_common::wallet::VaultService::new(network);
                        vault_service
                            .import_vault(&dummy_mnemonic, &descriptors_qr, &vault_name, false)
                            .await
                            .map_err(|e| format!("Setup failed: {}", e))?;

                        let vault_address = vault_service
                            .get_address()
                            .map_err(|e| format!("Failed to get address: {}", e))?;
                        Ok((vault_service, vault_address))
                    });

                match result {
                    Ok((vault_service, vault_address)) => {
                        if let Err(e) = runtime_handle.block_on(async {
                            app_state.initialize_vault_from_service(vault_service).await
                        }) {
                            state.error = Some(format!("Failed to initialize: {}", e));
                            state.is_importing = false;
                            return;
                        }

                        if let Some(ref mut handler) = app_state.async_handler {
                            handler.fetch_balance();
                            handler.fetch_address();
                        }

                        state.vault_address = Some(vault_address);
                        state.is_importing = false;
                    }
                    Err(e) => {
                        state.error = Some(e);
                        state.is_importing = false;
                    }
                }
            } else {
                state.error = Some("Runtime not available".to_string());
                state.is_importing = false;
            }
        }
    }

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.go_to_previous_step();
    }
}

// ============================================================================
// RESTORE FROM BACKUP FLOW
// ============================================================================

/// Enter seed phrase from paper backup
pub fn render_enter_seed_phrase(
    ui: &mut egui::Ui,
    state: &mut VaultCreationState,
) {
    ui.heading("Restore from Backup");
    ui.add_space(Spacing::MD);

    // Warning banner
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(80, 60, 0))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::YELLOW, "⚠");
                ui.colored_label(
                    egui::Color32::WHITE,
                    "Only use your PAPER BACKUP - the seed phrase you wrote down during vault creation."
                );
            });
        });

    ui.add_space(Spacing::LG);

    ui.label("Enter your 12 or 24 word seed phrase:");
    ui.add_space(Spacing::SM);
    
    ui.add(egui::TextEdit::multiline(&mut state.import_mnemonic_text)
        .hint_text("word1 word2 word3 word4 ...")
        .desired_width(400.0)
        .desired_rows(4)
        .password(true)); // Hide for security

    ui.add_space(Spacing::SM);
    ui.label(egui::RichText::new("Your seed phrase is never transmitted and stays on this device.")
        .small()
        .color(egui::Color32::GRAY));

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    if button_large(ui, "Continue").clicked() {
        let words: Vec<&str> = state.import_mnemonic_text.trim().split_whitespace().collect();
        
        if words.is_empty() {
            state.error = Some("Please enter your seed phrase".to_string());
            return;
        }

        if words.len() != 12 && words.len() != 24 {
            state.error = Some(format!(
                "Seed phrase should be 12 or 24 words (you entered {})",
                words.len()
            ));
            return;
        }

        // Validate mnemonic
        match Mnemonic::parse_in(Language::English, state.import_mnemonic_text.trim()) {
            Ok(_) => {
                state.error = None;
                if let Some(next) = state.next_step_for_role() {
                    state.advance_to_step(next);
                }
            }
            Err(e) => {
                state.error = Some(format!("Invalid seed phrase: {}", e));
            }
        }
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.go_to_previous_step();
    }
}

/// Scan descriptor QR for restore flow
pub fn render_scan_descriptor_restore(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.heading("Restore from Backup");
    ui.add_space(Spacing::MD);

    ui.label("Now enter the descriptor configuration from your mobile device.");
    ui.add_space(Spacing::SM);
    ui.label("On your mobile, go to Settings → Export Vault Descriptor.");
    ui.add_space(Spacing::LG);

    ui.label("Paste the descriptor configuration:");
    ui.add_space(Spacing::SM);
    ui.add(egui::TextEdit::multiline(&mut state.import_descriptors_qr)
        .hint_text("Paste configuration from mobile app...")
        .desired_width(400.0)
        .desired_rows(3));

    ui.add_space(Spacing::MD);

    // File load option
    ui.horizontal(|ui| {
        if ui.button("Load from File").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Text/JSON", &["txt", "json"])
                .pick_file()
            {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    state.import_descriptors_qr = contents;
                } else {
                    state.error = Some("Failed to read file".to_string());
                }
            }
        }
    });

    if let Some(ref error) = state.error {
        ui.add_space(Spacing::MD);
        ui.colored_label(egui::Color32::RED, error);
    }

    ui.add_space(Spacing::XL);

    if state.is_importing {
        ui.spinner();
        ui.label("Restoring vault...");
        return;
    }

    if button_large(ui, "Restore Vault").clicked() {
        if state.import_descriptors_qr.trim().is_empty() {
            state.error = Some("Please enter the descriptor configuration".to_string());
            return;
        }

        // Parse mnemonic
        let mnemonic = match Mnemonic::parse_in(Language::English, state.import_mnemonic_text.trim()) {
            Ok(m) => m,
            Err(e) => {
                state.error = Some(format!("Invalid seed phrase: {}", e));
                return;
            }
        };

        state.is_importing = true;
        state.error = None;

        if let Some(ref runtime) = app_state.runtime {
            let descriptors_qr = state.import_descriptors_qr.clone();
            let vault_name = state.vault_name.clone();
            let network = app_state.network;
            let runtime_handle = runtime.handle().clone();

            let result: Result<(bitvault_common::wallet::VaultService, String), String> =
                runtime.block_on(async {
                    let mut vault_service = bitvault_common::wallet::VaultService::new(network);
                    vault_service
                        .import_vault(&mnemonic, &descriptors_qr, &vault_name, false)
                        .await
                        .map_err(|e| format!("Restore failed: {}", e))?;

                    let vault_address = vault_service
                        .get_address()
                        .map_err(|e| format!("Failed to get address: {}", e))?;
                    Ok((vault_service, vault_address))
                });

            match result {
                Ok((vault_service, vault_address)) => {
                    if let Err(e) = runtime_handle.block_on(async {
                        app_state.initialize_vault_from_service(vault_service).await
                    }) {
                        state.error = Some(format!("Failed to initialize: {}", e));
                        state.is_importing = false;
                        return;
                    }

                    if let Some(ref mut handler) = app_state.async_handler {
                        handler.fetch_balance();
                        handler.fetch_address();
                    }

                    state.vault_address = Some(vault_address);
                    state.is_importing = false;
                    
                    // Go to PIN setup
                    if let Some(next) = state.next_step_for_role() {
                        state.advance_to_step(next);
                    }
                }
                Err(e) => {
                    state.error = Some(e);
                    state.is_importing = false;
                }
            }
        } else {
            state.error = Some("Runtime not available".to_string());
            state.is_importing = false;
        }
    }

    ui.add_space(Spacing::MD);
    if ui.button("← Back").clicked() {
        state.go_to_previous_step();
    }
}
