//! Vault creation step implementations

use crate::state::{AppState, Navigation};
use crate::ui::pin::render_pin_setup;
use crate::ui::vault_creation::{VaultCreationState, VaultCreationStep};
use bip39::{Language, Mnemonic};
use bitvault_common::utils::TimeDelay;
use eframe::egui;

/// Step 1: Generate or import mnemonic
pub fn render_mnemonic_generation(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.label("Step 1: Generate or Import Seed Phrase");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        if ui.button("Generate New").clicked() {
            // Generate new mnemonic (12 words = 128 bits entropy)
            // bip39 2.0 API: Mnemonic::new(entropy, language)
            use rand::RngCore;
            let mut entropy = [0u8; 16]; // 128 bits for 12 words
            rand::thread_rng().fill_bytes(&mut entropy);
            match Mnemonic::from_entropy_in(Language::English, &entropy) {
                Ok(mnemonic) => {
                    state.mnemonic = Some(mnemonic);
                }
                Err(e) => {
                    state.error = Some(format!("Failed to generate mnemonic: {}", e));
                    return;
                }
            }
            state.current_step = VaultCreationStep::DisplaySeedPhrase;
        }

        if ui.button("Import Existing").clicked() {
            state.current_step = VaultCreationStep::ImportVault;
            state.error = None;
        }
    });
}

/// Step 2: Display seed phrase with warning
pub fn render_display_seed_phrase(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.label("Step 2: Write Down Your Seed Phrase");
    ui.add_space(10.0);

    ui.colored_label(
        egui::Color32::RED,
        "⚠️ WARNING: Write this down in a safe place!",
    );
    ui.label("You will need this to recover your vault.");
    ui.add_space(10.0);

    if let Some(ref mnemonic) = state.mnemonic {
        // Display words in a grid
        let words: Vec<&str> = mnemonic.word_iter().collect();
        egui::Grid::new("seed_words")
            .num_columns(3)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                for (i, word) in words.iter().enumerate() {
                    ui.label(format!("{}. {}", i + 1, word));
                    if (i + 1) % 3 == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    ui.add_space(20.0);

    if ui.button("I've Written It Down").clicked() {
        state.current_step = VaultCreationStep::VerifySeedPhrase;
    }
}

/// Step 3: Verify seed phrase
pub fn render_verify_seed_phrase(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.label("Step 3: Verify Seed Phrase");
    ui.add_space(10.0);

    ui.label("Enter your seed phrase to verify:");
    ui.add_space(10.0);

    // Simple verification: ask user to enter words at specific positions
    // For now, just ask them to confirm they've written it down
    ui.label("Please confirm you have written down your seed phrase.");
    ui.label("You will need it to recover your vault.");
    ui.add_space(10.0);

    ui.checkbox(
        &mut state.verified_seed_phrase,
        "I have written down my seed phrase",
    );

    ui.add_space(20.0);

    if ui.button("Verify").clicked() {
        if state.verified_seed_phrase {
            state.current_step = VaultCreationStep::SetTimeDelay;
            state.error = None;
        } else {
            state.error = Some("Please confirm you have written down your seed phrase".to_string());
        }
    }

    if ui.button("Back").clicked() {
        state.current_step = VaultCreationStep::DisplaySeedPhrase;
    }
}

/// Step 4: Set time delay
pub fn render_set_time_delay(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.label("Step 4: Set Time Delay");
    ui.add_space(10.0);

    ui.label("Time delay before fast path becomes available:");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("Days:");
        ui.add(egui::Slider::new(&mut state.time_delay_days, 0..=365));
    });

    ui.horizontal(|ui| {
        ui.label("Hours:");
        ui.add(egui::Slider::new(&mut state.time_delay_hours, 0..=23));
    });

    ui.add_space(20.0);

    if ui.button("Next").clicked() {
        state.current_step = VaultCreationStep::SetPin;
    }

    if ui.button("Back").clicked() {
        state.current_step = VaultCreationStep::VerifySeedPhrase;
    }
}

/// Step 5: Set PIN
pub fn render_set_pin(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    _navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.label("Step 5: Set PIN");
    ui.add_space(10.0);
    ui.label("Set a 6-digit PIN to secure your wallet");
    ui.add_space(20.0);

    let mut callback = None;
    let pin_set = render_pin_setup(ui, &mut state.pin_setup_state, &mut callback);

    if pin_set {
        // PIN set successfully - move to next step
        state.current_step = VaultCreationStep::GenerateCoownerQR;
    }

    ui.add_space(20.0);
    if ui.button("Back").clicked() {
        state.current_step = VaultCreationStep::SetTimeDelay;
    }
}

/// Step 6: Generate QR code for coowner
pub fn render_generate_coowner_qr(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.label("Step 5: Generate QR Code for Coowner");
    ui.add_space(10.0);

    ui.label("On the coowner device, generate and display the QR code.");
    ui.label("Then scan it here or enter the keys manually.");
    ui.add_space(10.0);

    ui.label("Coowner QR/Keys:");
    ui.text_edit_singleline(&mut state.coowner_pubkeys);

    ui.add_space(20.0);
    if ui.button("Next").clicked() {
        if !state.coowner_pubkeys.is_empty() {
            state.current_step = VaultCreationStep::EmailAuth;
            state.error = None;
        } else {
            state.error = Some("Please enter coowner keys or scan QR code".to_string());
        }
    }

    if ui.button("Back").clicked() {
        state.current_step = VaultCreationStep::SetTimeDelay;
    }
}

/// Step 6: Email 2FA
pub fn render_email_auth(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut VaultCreationState,
) {
    ui.label("Step 6: Email Authentication");
    ui.add_space(10.0);

    ui.label("Enter your email address to receive an authentication code:");
    ui.add_space(5.0);

    ui.text_edit_singleline(&mut state.email);
    ui.add_space(10.0);

    // Send code button
    if ui.button("Send Authentication Code").clicked() && !state.email.trim().is_empty() {
        if !state.email.contains('@') {
            state.error = Some("Please enter a valid email address".to_string());
        } else {
            state.is_sending_code = true;
            state.error = None;

            // Create a temporary service just for sending the code
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
        ui.label("Sending code...");
    }

    if state.code_sent {
        ui.add_space(10.0);
        ui.colored_label(egui::Color32::GREEN, "✓ Code sent! Check your email.");
        ui.add_space(10.0);

        ui.label("Enter the authentication code:");
        ui.text_edit_singleline(&mut state.auth_code);
        ui.add_space(10.0);

        if ui.button("Verify and Continue").clicked() {
            if !state.auth_code.trim().is_empty() {
                state.current_step = VaultCreationStep::LinkCoowner;
                state.error = None;
            } else {
                state.error = Some("Please enter the authentication code".to_string());
            }
        }
    }

    ui.add_space(20.0);
    if ui.button("Back").clicked() {
        state.current_step = VaultCreationStep::GenerateCoownerQR;
        state.code_sent = false;
        state.auth_code.clear();
    }
}

/// Step 7: Link coowner (now just shows summary before creation)
pub fn render_link_coowner(ui: &mut egui::Ui, state: &mut VaultCreationState) {
    ui.label("Step 7: Confirm Vault Details");
    ui.add_space(10.0);

    ui.label("Coowner keys entered:");
    ui.label(&state.coowner_pubkeys);
    ui.add_space(10.0);

    ui.label("Vault Name:");
    ui.text_edit_singleline(&mut state.vault_name);

    ui.add_space(20.0);

    if ui.button("Next").clicked() {
        if !state.vault_name.is_empty() {
            state.current_step = VaultCreationStep::CreateVault;
            state.error = None;
        } else {
            state.error = Some("Please enter a vault name".to_string());
        }
    }

    if ui.button("Back").clicked() {
        state.current_step = VaultCreationStep::EmailAuth;
    }
}

/// Step 8: Create vault
pub fn render_create_vault(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.label("Step 8: Create Vault");
    ui.add_space(10.0);

    if state.is_creating {
        ui.label("Creating vault...");
        ui.add_space(10.0);
        ui.spinner();
        return;
    }

    ui.label("Ready to create vault:");
    ui.label(format!("Name: {}", state.vault_name));
    ui.label(format!(
        "Time Delay: {} days, {} hours",
        state.time_delay_days, state.time_delay_hours
    ));
    ui.label(format!("Email: {}", state.email));
    ui.add_space(10.0);

    ui.add_space(20.0);

    if ui.button("Create Vault").clicked() {
        state.is_creating = true;
        state.error = None;

        // Validate inputs
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

        if state.coowner_pubkeys.trim().is_empty() {
            state.error = Some("Please enter coowner keys or scan QR code".to_string());
            state.is_creating = false;
            return;
        }

        // Create vault synchronously using block_on (acceptable for one-time operation)
        if let (Some(mnemonic), Some(runtime)) =
            (state.mnemonic.as_ref(), app_state.runtime.as_ref())
        {
            let time_delay = TimeDelay {
                days: state.time_delay_days,
                hours: state.time_delay_hours,
            };
            let coowner_pubkeys = state.coowner_pubkeys.clone();
            let vault_name = state.vault_name.clone();
            let network = app_state.network;
            let email_clone = state.email.clone();
            let auth_code_clone = state.auth_code.clone();
            let runtime_handle = runtime.handle().clone();

            let result = runtime.block_on(async {
                let mut vault_service = bitvault_common::wallet::VaultService::new(network);

                let qr_result = vault_service
                    .setup_vault(
                        mnemonic,
                        &coowner_pubkeys,
                        time_delay,
                        &vault_name,
                        &email_clone,
                        &auth_code_clone,
                    )
                    .await;

                // If setup succeeded, vault_service now has the wallet initialized
                // Return both the QR result and the vault service
                match qr_result {
                    Ok(qr) => Ok((qr, vault_service)),
                    Err(e) => Err(e),
                }
            });

            match result {
                Ok((final_qr, vault_service)) => {
                    state.final_qr = Some(final_qr);

                    // Initialize vault in app_state (use runtime handle to avoid borrow conflict)
                    if let Err(e) = runtime_handle.block_on(async {
                        app_state.initialize_vault_from_service(vault_service).await
                    }) {
                        state.error = Some(format!("Failed to initialize vault in app: {}", e));
                    } else {
                        // Fetch initial data
                        if let Some(ref mut handler) = app_state.async_handler {
                            handler.fetch_balance();
                            handler.fetch_address();
                        }

                        // Navigate to dashboard
                        navigation.navigate_to(crate::state::View::Dashboard { tab: 0 });
                    }

                    state.current_step = VaultCreationStep::Completed;
                    state.is_creating = false;
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

    if ui.button("Back").clicked() {
        state.current_step = VaultCreationStep::LinkCoowner;
    }
}

/// Step 8: Completed
pub fn render_completed(
    ui: &mut egui::Ui,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.heading("Vault Created Successfully!");
    ui.add_space(10.0);

    ui.label(format!("Vault Name: {}", state.vault_name));

    if let Some(ref address) = state.vault_address {
        ui.label(format!("Vault Address: {}", address));
    }

    if let Some(ref final_qr) = state.final_qr {
        ui.add_space(10.0);
        ui.label("QR Code for Second Device:");
        // Generate and display QR code
        use crate::utils::qr::generate_qr_image;

        if let Some(qr_texture) = generate_qr_image(ui.ctx(), final_qr) {
            ui.image((qr_texture.id(), egui::Vec2::new(300.0, 300.0)));
            ui.add_space(10.0);
        } else {
            ui.colored_label(egui::Color32::YELLOW, "Failed to generate QR code");
            ui.label(format!(
                "QR Data: {}...",
                &final_qr[..final_qr.len().min(50)]
            ));
        }
    }

    ui.add_space(20.0);

    if ui.button("Go to Dashboard").clicked() {
        navigation.navigate_to(crate::state::View::Dashboard { tab: 0 });
    }
}

/// Import vault flow
/// Step 1: Enter mnemonic and scan QR descriptors
pub fn render_import_vault(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    ui.label("Import Existing Vault");
    ui.add_space(10.0);

    ui.label("Enter your 12 or 24-word mnemonic phrase:");
    ui.add_space(5.0);
    ui.text_edit_multiline(&mut state.import_mnemonic_text);
    ui.add_space(10.0);

    ui.label("Scan or paste the descriptor QR code (compressed base64):");
    ui.add_space(5.0);
    ui.text_edit_multiline(&mut state.import_descriptors_qr);
    ui.add_space(10.0);

    ui.label("Vault Name:");
    ui.text_edit_singleline(&mut state.vault_name);
    ui.add_space(10.0);

    ui.checkbox(&mut state.is_coowner, "This is a coowner device");
    ui.add_space(10.0);

    if state.is_importing {
        ui.label("Importing vault...");
        ui.add_space(10.0);
        ui.spinner();
        return;
    }

    ui.horizontal(|ui| {
        if ui.button("Import").clicked() {
            // Validate inputs
            if state.import_mnemonic_text.trim().is_empty() {
                state.error = Some("Please enter your mnemonic phrase".to_string());
                return;
            }

            if state.import_descriptors_qr.trim().is_empty() {
                state.error = Some("Please enter or scan the descriptor QR code".to_string());
                return;
            }

            if state.vault_name.trim().is_empty() {
                state.error = Some("Please enter a vault name".to_string());
                return;
            }

            // Parse mnemonic
            let mnemonic_text = state.import_mnemonic_text.trim();
            let mnemonic = match Mnemonic::parse_in(Language::English, mnemonic_text) {
                Ok(m) => m,
                Err(e) => {
                    state.error = Some(format!("Invalid mnemonic: {}", e));
                    return;
                }
            };

            state.is_importing = true;
            state.error = None;

            // Import vault using runtime
            if let Some(ref runtime) = app_state.runtime {
                let descriptors_qr = state.import_descriptors_qr.clone();
                let vault_name = state.vault_name.clone();
                let is_coowner = state.is_coowner;
                let network = app_state.network;
                let runtime_handle = runtime.handle().clone();

                let result: Result<(bitvault_common::wallet::VaultService, String), String> =
                    runtime.block_on(async {
                        let mut vault_service = bitvault_common::wallet::VaultService::new(network);

                        vault_service
                            .import_vault(&mnemonic, &descriptors_qr, &vault_name, is_coowner)
                            .await
                            .map_err(|e| format!("Import failed: {}", e))?;

                        // After import, the vault service has the wallet initialized
                        // Get the vault address from the service
                        let vault_address = vault_service
                            .get_address()
                            .map_err(|e| format!("Failed to get address: {}", e))?;
                        Ok((vault_service, vault_address))
                    });

                match result {
                    Ok((vault_service, vault_address)) => {
                        // Initialize vault in app_state
                        if let Err(e) = runtime_handle.block_on(async {
                            app_state.initialize_vault_from_service(vault_service).await
                        }) {
                            state.error = Some(format!("Failed to initialize vault in app: {}", e));
                            state.is_importing = false;
                            return;
                        }

                        // Fetch initial data
                        if let Some(ref mut handler) = app_state.async_handler {
                            handler.fetch_balance();
                            handler.fetch_address();
                        }

                        // Navigate to dashboard
                        navigation.navigate_to(crate::state::View::Dashboard { tab: 0 });
                        state.vault_address = Some(vault_address);
                        state.current_step = VaultCreationStep::Completed;
                        state.is_importing = false;
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to import vault: {}", e));
                        state.is_importing = false;
                    }
                }
            } else {
                state.error = Some("Runtime not available".to_string());
                state.is_importing = false;
            }
        }

        if ui.button("Cancel").clicked() {
            state.current_step = VaultCreationStep::MnemonicGeneration;
            state.import_mnemonic_text.clear();
            state.import_descriptors_qr.clear();
            state.vault_name.clear();
            state.error = None;
        }
    });

    // Show error if any
    if let Some(ref error) = state.error {
        ui.add_space(10.0);
        ui.colored_label(egui::Color32::RED, error);
    }
}
