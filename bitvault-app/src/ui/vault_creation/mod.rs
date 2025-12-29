//! Vault Creation UI
//!
//! Implements the vault creation flow matching the mobile app:
//! 1. Generate mnemonic (or import)
//! 2. Display seed phrase
//! 3. Verify seed phrase
//! 4. Set time delay
//! 5. Generate QR code for coowner
//! 6. Link coowner (scan QR or manual entry)
//! 7. Create vault on convenience service

mod steps;

use eframe::egui;
use crate::state::{AppState, Navigation};
use bip39::Mnemonic;

/// Vault creation state
#[derive(Debug, Clone, PartialEq)]
pub enum VaultCreationStep {
    /// Step 1: Generate or import mnemonic
    MnemonicGeneration,
    /// Step 2: Display seed phrase (with warning)
    DisplaySeedPhrase,
    /// Step 3: Verify seed phrase
    VerifySeedPhrase,
    /// Step 4: Set time delay
    SetTimeDelay,
    /// Step 5: Set PIN
    SetPin,
    /// Step 6: Generate QR for coowner
    GenerateCoownerQR,
    /// Step 7: Email 2FA authentication
    EmailAuth,
    /// Step 8: Link coowner (confirm details)
    LinkCoowner,
    /// Step 9: Create vault
    CreateVault,
    /// Completed
    Completed,
}

/// Vault creation state
pub struct VaultCreationState {
    pub current_step: VaultCreationStep,
    pub mnemonic: Option<Mnemonic>,
    pub verified_seed_phrase: bool,
    pub time_delay_days: u32,
    pub time_delay_hours: u32,
    pub coowner_pubkeys: String, // QR string from coowner device
    pub coowner_keys: Option<bitvault_common::derivation::CoownerKeys>,
    pub vault_name: String,
    pub vault_address: Option<String>,
    pub final_qr: Option<String>, // QR for second device
    pub email: String,
    pub auth_code: String,
    pub code_sent: bool,
    pub is_sending_code: bool,
    pub error: Option<String>,
    pub is_creating: bool,
    pub pin_setup_state: crate::ui::pin::PinSetupState,
}

impl Default for VaultCreationState {
    fn default() -> Self {
        Self {
            current_step: VaultCreationStep::MnemonicGeneration,
            mnemonic: None,
            verified_seed_phrase: false,
            time_delay_days: 0,
            time_delay_hours: 24,
            coowner_pubkeys: String::new(),
            coowner_keys: None,
            vault_name: String::new(),
            vault_address: None,
            final_qr: None,
            email: String::new(),
            auth_code: String::new(),
            code_sent: false,
            is_sending_code: false,
            error: None,
            is_creating: false,
            pin_setup_state: crate::ui::pin::PinSetupState::new(),
        }
    }
}

/// Render vault creation flow
pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation, state: &mut VaultCreationState) {
    ui.vertical_centered(|ui| {
        ui.heading("Create New Vault");
        ui.add_space(20.0);

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }

        // Render current step
        match state.current_step {
            VaultCreationStep::MnemonicGeneration => {
                steps::render_mnemonic_generation(ui, state);
            }
            VaultCreationStep::DisplaySeedPhrase => {
                steps::render_display_seed_phrase(ui, state);
            }
            VaultCreationStep::VerifySeedPhrase => {
                steps::render_verify_seed_phrase(ui, state);
            }
            VaultCreationStep::SetTimeDelay => {
                steps::render_set_time_delay(ui, state);
            }
            VaultCreationStep::SetPin => {
                steps::render_set_pin(ui, app_state, navigation, state);
            }
            VaultCreationStep::GenerateCoownerQR => {
                steps::render_generate_coowner_qr(ui, state);
            }
            VaultCreationStep::EmailAuth => {
                steps::render_email_auth(ui, app_state, state);
            }
            VaultCreationStep::LinkCoowner => {
                steps::render_link_coowner(ui, state);
            }
            VaultCreationStep::CreateVault => {
                steps::render_create_vault(ui, app_state, navigation, state);
            }
            VaultCreationStep::Completed => {
                steps::render_completed(ui, navigation, state);
            }
        }

        ui.add_space(20.0);

        // Navigation buttons
        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                navigation.go_back();
            }
        });
    });
}

