//! Vault Creation UI
//!
//! Implements vault setup with four modes:
//!
//! ## 1. View-Only Mode
//! - Scan descriptor QR from mobile
//! - No seed phrase needed
//! - Cannot sign transactions (monitoring only)
//!
//! ## 2. Create New Vault (Main Device)
//! - Generate new seed phrase on this device
//! - Set time delay
//! - Exchange public keys with co-owner
//! - Full signing capability
//!
//! ## 3. Join as Co-owner
//! - Generate new seed phrase on this device
//! - Exchange public keys with main device
//! - Full signing capability
//!
//! ## 4. Restore from Backup (Disaster Recovery)
//! - Enter seed phrase from PAPER BACKUP
//! - Scan descriptor QR from mobile
//! - Full signing capability restored

mod steps;

use crate::state::{AppState, Navigation};
use bip39::Mnemonic;
use eframe::egui;
use std::collections::HashMap;

/// Hardware wallet types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HardwareWalletType {
    Jade,
    JadePlus,
    Keystone,
    Passport, // Foundation Passport
    SeedSigner,
    BitBox02,
    LedgerNanoX,
    ColdCardQ,
}

impl HardwareWalletType {
    pub fn title(&self) -> &'static str {
        match self {
            HardwareWalletType::Jade => "Jade",
            HardwareWalletType::JadePlus => "Jade Plus",
            HardwareWalletType::Keystone => "Keystone",
            HardwareWalletType::Passport => "Foundation Passport",
            HardwareWalletType::SeedSigner => "Seed Signer",
            HardwareWalletType::BitBox02 => "BitBox02",
            HardwareWalletType::LedgerNanoX => "Ledger Nano X",
            HardwareWalletType::ColdCardQ => "ColdCard Q",
        }
    }

    /// Returns true if this hardware wallet uses multi-part UR codes
    pub fn uses_multi_part_ur(&self) -> bool {
        matches!(
            self,
            HardwareWalletType::Jade
                | HardwareWalletType::JadePlus
                | HardwareWalletType::Passport
                | HardwareWalletType::SeedSigner
        )
    }

    /// Get guidance message for this hardware wallet type
    pub fn guidance_message(&self) -> &'static str {
        if self.uses_multi_part_ur() {
            "This hardware wallet uses multi-part UR codes. Scan each QR code in sequence."
        } else {
            "This hardware wallet uses single-part UR codes. Scan the complete QR code."
        }
    }

    /// Get list of all supported hardware wallet types
    pub fn all_types() -> Vec<HardwareWalletType> {
        vec![
            HardwareWalletType::Jade,
            HardwareWalletType::JadePlus,
            HardwareWalletType::Keystone,
            HardwareWalletType::Passport,
            HardwareWalletType::SeedSigner,
            HardwareWalletType::BitBox02,
            HardwareWalletType::LedgerNanoX,
            HardwareWalletType::ColdCardQ,
        ]
    }
}

impl Default for HardwareWalletType {
    fn default() -> Self {
        HardwareWalletType::Jade // Default to most common
    }
}

/// Device role / setup mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DeviceRole {
    #[default]
    Main,
    Coowner,
    ViewOnly,
    Restore,
    /// Single device vault: Seed phrase + Hardware Wallet
    SingleDeviceSeedHW,
    /// Single device vault: Hardware Wallet + Hardware Wallet
    SingleDeviceHWHW,
}

/// Vault creation steps
#[derive(Debug, Clone, PartialEq)]
pub enum VaultCreationStep {
    /// First step: Select how to set up
    RoleSelection,
    /// Name the vault
    NameVault,
    /// Set time delay (main device only)
    SetTimeDelay,
    /// Generate new seed phrase
    MnemonicGeneration,
    /// Display seed phrase (with warning)
    DisplaySeedPhrase,
    /// Verify seed phrase
    VerifySeedPhrase,
    /// Set PIN
    SetPin,
    /// Main device: Scan co-owner's keys
    ScanCoownerKeys,
    /// Co-owner device: Display own keys for main to scan
    DisplayOwnKeys,
    /// Co-owner device: Enter exchange data from main
    EnterExchangeData,
    /// Email 2FA authentication
    EmailAuth,
    /// Create/Join vault
    CreateVault,
    /// Main device: Display exchange data for co-owner
    DisplayExchangeData,
    /// Completed
    Completed,
    // --- View-Only Flow ---
    /// Scan descriptor QR (view-only)
    ScanDescriptorViewOnly,
    /// View-only setup complete
    ViewOnlyComplete,
    // --- Restore from Backup Flow ---
    /// Enter seed phrase from paper backup
    EnterSeedPhrase,
    /// Scan descriptor QR for restore
    ScanDescriptorRestore,
}

/// Seed phrase verification state
#[derive(Default)]
pub struct SeedPhraseVerificationState {
    /// Selected words: Map<word_index (0-11), selected_word>
    pub selected_words: HashMap<usize, String>,
    /// Word choices for each position: Map<word_index, Vec<word_options>>
    pub word_choices: HashMap<usize, Vec<String>>,
    /// Indices of words being verified (e.g., [0, 3, 6, 9, 1, 4, 7, 10, 2, 5, 8, 11])
    pub verification_indices: Vec<usize>,
    /// Whether state has been initialized
    pub initialized: bool,
    /// Current page (0 = first 6 words, 1 = next 6 words)
    pub current_page: usize,
}

/// Vault creation state
pub struct VaultCreationState {
    pub current_step: VaultCreationStep,
    /// Previous step (for detecting step changes and auto-focus)
    pub previous_step: Option<VaultCreationStep>,
    /// History of steps for back button navigation
    pub step_history: Vec<VaultCreationStep>,
    /// This device's role (main or co-owner)
    pub device_role: DeviceRole,
    pub mnemonic: Option<Mnemonic>,
    pub seed_verification_state: SeedPhraseVerificationState,
    /// Word count for mnemonic generation (12 or 24 words)
    pub mnemonic_word_count: u8, // 12 or 24
    pub time_delay_days: u32,
    pub time_delay_hours: u32,
    /// Co-owner's public keys (text input from co-owner device)
    pub coowner_pubkeys: String,
    pub coowner_keys: Option<bitvault_common::derivation::CoownerKeys>,
    /// First hardware wallet keys (for HW+HW single device scenario)
    pub first_hw_keys: Option<bitvault_common::derivation::CoownerKeys>,
    /// Track which hardware wallet we're scanning in HW+HW mode (false = first, true = second)
    pub scanning_second_hw: bool,
    /// This device's own keys as text (for sharing with other device)
    pub my_keys_text: Option<String>,
    /// Exchange data from main device (for co-owner flow)
    pub exchange_data_input: String,
    pub vault_name: String,
    pub vault_address: Option<String>,
    /// Exchange data to share with co-owner (after vault creation)
    pub exchange_data_output: Option<String>,
    pub email: String,
    pub auth_code: String,
    pub code_sent: bool,
    pub is_sending_code: bool,
    pub error: Option<String>,
    pub is_creating: bool,
    pub pin_setup_state: crate::ui::pin::PinSetupState,
    // Import-specific fields
    pub import_mnemonic_text: String,
    pub import_descriptors_qr: String,
    pub is_importing: bool,
    // Camera for QR scanning
    pub camera_capture: Option<crate::utils::camera::CameraCapture>,
    pub is_scanning_qr: bool,
    /// Selected hardware wallet type (when using hardware wallet as co-owner)
    pub selected_hw_type: Option<HardwareWalletType>,
    /// First hardware wallet type (for HW+HW single device scenario)
    pub first_hw_type: Option<HardwareWalletType>,
    // Hardware wallet batch QR scanner for multi-part UR codes
    pub hw_batch_qr_scanner_state: crate::ui::hardware_wallet::BatchQrScannerState,
    // Track saved file paths for secure deletion
    pub saved_key_file: Option<std::path::PathBuf>,
    pub saved_exchange_file: Option<std::path::PathBuf>,
    // Encryption keys for file exchange
    // Co-owner: stores signing key pair (private key for File 2 decryption)
    // Main: stores co-owner's public key from File 1 (for File 2 encryption)
    pub signing_secret_key: Option<secp256k1::SecretKey>,
    pub recipient_public_key: Option<secp256k1::PublicKey>,
}

impl Default for VaultCreationState {
    fn default() -> Self {
        Self {
            current_step: VaultCreationStep::RoleSelection,
            previous_step: None,
            step_history: Vec::new(),
            device_role: DeviceRole::default(),
            mnemonic: None,
            seed_verification_state: SeedPhraseVerificationState::default(),
            mnemonic_word_count: 12, // Default to 12 words (matching mobile)
            time_delay_days: 0,
            time_delay_hours: 24,
            coowner_pubkeys: String::new(),
            coowner_keys: None,
            first_hw_keys: None,
            scanning_second_hw: false,
            my_keys_text: None,
            exchange_data_input: String::new(),
            vault_name: String::new(),
            vault_address: None,
            exchange_data_output: None,
            email: String::new(),
            auth_code: String::new(),
            code_sent: false,
            is_sending_code: false,
            error: None,
            is_creating: false,
            pin_setup_state: crate::ui::pin::PinSetupState::new(),
            import_mnemonic_text: String::new(),
            import_descriptors_qr: String::new(),
            is_importing: false,
            camera_capture: None,
            is_scanning_qr: false,
            selected_hw_type: None,
            first_hw_type: None,
            hw_batch_qr_scanner_state: crate::ui::hardware_wallet::BatchQrScannerState::default(),
            saved_key_file: None,
            saved_exchange_file: None,
            signing_secret_key: None,
            recipient_public_key: None,
        }
    }
}

impl VaultCreationState {
    /// Advance to the next step in the workflow
    /// This tracks the step history for back button navigation
    pub fn advance_to_step(&mut self, step: VaultCreationStep) {
        // Don't track history if we're going back (handled by go_to_previous_step)
        // Only track when advancing forward
        if step != self.current_step {
            // Update previous_step for auto-focus detection
            self.previous_step = Some(self.current_step.clone());
            self.step_history.push(self.current_step.clone());
            self.current_step = step;
        }
    }

    /// Go back to the previous step in the workflow
    /// Returns true if there was a previous step, false if at first step
    pub fn go_to_previous_step(&mut self) -> bool {
        if let Some(previous) = self.step_history.pop() {
            self.previous_step = Some(self.current_step.clone());
            self.current_step = previous;
            true
        } else {
            false // At first step
        }
    }

    /// Check if we can go back in the workflow
    pub fn can_go_back_in_workflow(&self) -> bool {
        !self.step_history.is_empty()
    }

    /// Check if the step just changed (for auto-focus)
    /// Returns true only on the first frame after a step change.
    /// Updates previous_step immediately to prevent it from returning true again.
    pub fn step_just_changed(&mut self, current: VaultCreationStep) -> bool {
        // Check if step changed (previous_step is different from current)
        let changed = self.previous_step.as_ref() != Some(&current);
        if changed {
            // Immediately update previous_step to prevent this from returning true again
            // This ensures request_focus() is only called once per step change
            self.previous_step = Some(current.clone());
        }
        changed
    }

    /// Reset workflow state (called when exiting workflow)
    pub fn reset(&mut self) {
        self.current_step = VaultCreationStep::RoleSelection;
        self.step_history.clear();
        self.device_role = DeviceRole::default();
        // Don't clear other state - user might want to resume
    }

    /// Full reset for starting a new vault creation flow
    /// Call this when user selects a role from role selection
    pub fn reset_for_new_flow(&mut self) {
        // Clear step history
        self.step_history.clear();

        // Clear all input state
        self.mnemonic = None;
        self.seed_verification_state = SeedPhraseVerificationState::default();
        self.time_delay_days = 0;
        self.time_delay_hours = 24;
        self.coowner_pubkeys.clear();
        self.coowner_keys = None;
        self.first_hw_keys = None;
        self.first_hw_type = None;
        self.scanning_second_hw = false;
        self.my_keys_text = None;
        self.exchange_data_input.clear();
        self.vault_name.clear();
        self.vault_address = None;
        self.exchange_data_output = None;
        self.email.clear();
        self.auth_code.clear();
        self.code_sent = false;
        self.is_sending_code = false;
        self.error = None;
        self.is_creating = false;
        self.pin_setup_state = crate::ui::pin::PinSetupState::new();
        self.import_mnemonic_text.clear();
        self.import_descriptors_qr.clear();
        self.is_importing = false;

        // Stop camera if running
        if let Some(ref mut camera) = self.camera_capture {
            camera.stop_capture();
        }
        self.camera_capture = None;
        self.is_scanning_qr = false;
        self.saved_key_file = None;
        self.saved_exchange_file = None;
        self.signing_secret_key = None;
        self.recipient_public_key = None;

        log::info!("Reset vault creation state for new flow");
    }

    /// Clear sensitive data from memory (seed phrases, mnemonics)
    /// Call this after vault creation/import succeeds
    pub fn clear_sensitive_data(&mut self) {
        // Clear the generated mnemonic
        self.mnemonic = None;

        // Clear any imported seed phrase text by overwriting with zeros first
        // This helps ensure the data is actually cleared from memory
        let len = self.import_mnemonic_text.len();
        self.import_mnemonic_text.clear();
        self.import_mnemonic_text.reserve(len);
        for _ in 0..len {
            self.import_mnemonic_text.push('\0');
        }
        self.import_mnemonic_text.clear();

        // Clear PIN setup state
        self.pin_setup_state = crate::ui::pin::PinSetupState::new();

        log::info!("Cleared sensitive vault creation data from memory");
    }

    /// Get the next step based on role and current step
    pub fn next_step_for_role(&self) -> Option<VaultCreationStep> {
        match self.device_role {
            DeviceRole::Main => self.next_step_main(),
            DeviceRole::Coowner => self.next_step_coowner(),
            DeviceRole::ViewOnly => self.next_step_view_only(),
            DeviceRole::Restore => self.next_step_restore(),
            DeviceRole::SingleDeviceSeedHW => self.next_step_single_device_seed_hw(),
            DeviceRole::SingleDeviceHWHW => self.next_step_single_device_hw_hw(),
        }
    }

    fn next_step_main(&self) -> Option<VaultCreationStep> {
        match self.current_step {
            VaultCreationStep::RoleSelection => Some(VaultCreationStep::NameVault),
            VaultCreationStep::NameVault => Some(VaultCreationStep::SetTimeDelay),
            VaultCreationStep::SetTimeDelay => Some(VaultCreationStep::MnemonicGeneration),
            VaultCreationStep::MnemonicGeneration => Some(VaultCreationStep::DisplaySeedPhrase),
            VaultCreationStep::DisplaySeedPhrase => Some(VaultCreationStep::VerifySeedPhrase),
            VaultCreationStep::VerifySeedPhrase => Some(VaultCreationStep::SetPin),
            VaultCreationStep::SetPin => Some(VaultCreationStep::ScanCoownerKeys),
            VaultCreationStep::ScanCoownerKeys => Some(VaultCreationStep::EmailAuth),
            VaultCreationStep::EmailAuth => Some(VaultCreationStep::CreateVault),
            VaultCreationStep::CreateVault => Some(VaultCreationStep::DisplayExchangeData),
            VaultCreationStep::DisplayExchangeData => Some(VaultCreationStep::Completed),
            _ => None,
        }
    }

    fn next_step_coowner(&self) -> Option<VaultCreationStep> {
        match self.current_step {
            VaultCreationStep::RoleSelection => Some(VaultCreationStep::NameVault),
            VaultCreationStep::NameVault => Some(VaultCreationStep::MnemonicGeneration),
            VaultCreationStep::MnemonicGeneration => Some(VaultCreationStep::DisplaySeedPhrase),
            VaultCreationStep::DisplaySeedPhrase => Some(VaultCreationStep::VerifySeedPhrase),
            VaultCreationStep::VerifySeedPhrase => Some(VaultCreationStep::SetPin),
            VaultCreationStep::SetPin => Some(VaultCreationStep::DisplayOwnKeys),
            VaultCreationStep::DisplayOwnKeys => Some(VaultCreationStep::EnterExchangeData),
            VaultCreationStep::EnterExchangeData => Some(VaultCreationStep::EmailAuth),
            VaultCreationStep::EmailAuth => Some(VaultCreationStep::CreateVault),
            VaultCreationStep::CreateVault => Some(VaultCreationStep::Completed),
            _ => None,
        }
    }

    fn next_step_view_only(&self) -> Option<VaultCreationStep> {
        match self.current_step {
            VaultCreationStep::RoleSelection => Some(VaultCreationStep::NameVault),
            VaultCreationStep::NameVault => Some(VaultCreationStep::ScanDescriptorViewOnly),
            VaultCreationStep::ScanDescriptorViewOnly => Some(VaultCreationStep::ViewOnlyComplete),
            VaultCreationStep::ViewOnlyComplete => Some(VaultCreationStep::Completed),
            _ => None,
        }
    }

    fn next_step_restore(&self) -> Option<VaultCreationStep> {
        match self.current_step {
            VaultCreationStep::RoleSelection => Some(VaultCreationStep::NameVault),
            VaultCreationStep::NameVault => Some(VaultCreationStep::EnterSeedPhrase),
            VaultCreationStep::EnterSeedPhrase => Some(VaultCreationStep::ScanDescriptorRestore),
            VaultCreationStep::ScanDescriptorRestore => Some(VaultCreationStep::SetPin),
            VaultCreationStep::SetPin => Some(VaultCreationStep::Completed),
            _ => None,
        }
    }

    fn next_step_single_device_seed_hw(&self) -> Option<VaultCreationStep> {
        match self.current_step {
            VaultCreationStep::RoleSelection => Some(VaultCreationStep::NameVault),
            VaultCreationStep::NameVault => Some(VaultCreationStep::SetTimeDelay),
            VaultCreationStep::SetTimeDelay => Some(VaultCreationStep::MnemonicGeneration),
            VaultCreationStep::MnemonicGeneration => Some(VaultCreationStep::DisplaySeedPhrase),
            VaultCreationStep::DisplaySeedPhrase => Some(VaultCreationStep::VerifySeedPhrase),
            VaultCreationStep::VerifySeedPhrase => Some(VaultCreationStep::SetPin),
            VaultCreationStep::SetPin => Some(VaultCreationStep::ScanCoownerKeys), // Scan HW keys instead of co-owner keys
            VaultCreationStep::ScanCoownerKeys => Some(VaultCreationStep::EmailAuth),
            VaultCreationStep::EmailAuth => Some(VaultCreationStep::CreateVault),
            VaultCreationStep::CreateVault => Some(VaultCreationStep::Completed),
            _ => None,
        }
    }

    fn next_step_single_device_hw_hw(&self) -> Option<VaultCreationStep> {
        match self.current_step {
            VaultCreationStep::RoleSelection => Some(VaultCreationStep::NameVault),
            VaultCreationStep::NameVault => Some(VaultCreationStep::SetTimeDelay),
            VaultCreationStep::SetTimeDelay => Some(VaultCreationStep::ScanCoownerKeys), // Scan first HW
            VaultCreationStep::ScanCoownerKeys => {
                // After scanning first HW, check if we need to scan second HW
                if self.first_hw_keys.is_some() && !self.scanning_second_hw {
                    // First HW scanned, now need second HW - stay on ScanCoownerKeys
                    None // Will be handled by UI logic - it sets scanning_second_hw and stays on step
                } else if self.first_hw_keys.is_some() && self.coowner_keys.is_some() {
                    // Both HWs scanned, proceed to SetPin
                    Some(VaultCreationStep::SetPin)
                } else {
                    // Still scanning first HW or error
                    None
                }
            }
            VaultCreationStep::SetPin => Some(VaultCreationStep::EmailAuth),
            VaultCreationStep::EmailAuth => Some(VaultCreationStep::CreateVault),
            VaultCreationStep::CreateVault => Some(VaultCreationStep::Completed),
            _ => None,
        }
    }
}

/// Render vault creation flow
pub fn render(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut VaultCreationState,
) {
    let ctx = ui.ctx().clone();

    ui.vertical_centered(|ui| {
        // Render current step (each step handles its own error display)
        match state.current_step {
            VaultCreationStep::RoleSelection => {
                steps::render_role_selection(ui, state, navigation);
            }
            VaultCreationStep::NameVault => {
                steps::render_name_vault(ui, state);
            }
            VaultCreationStep::SetTimeDelay => {
                steps::render_set_time_delay(ui, state);
            }
            VaultCreationStep::MnemonicGeneration => {
                steps::render_mnemonic_generation(ui, state);
            }
            VaultCreationStep::DisplaySeedPhrase => {
                steps::render_display_seed_phrase(ui, state);
            }
            VaultCreationStep::VerifySeedPhrase => {
                steps::render_verify_seed_phrase(ui, state);
            }
            VaultCreationStep::SetPin => {
                steps::render_set_pin(ui, app_state, navigation, state);
            }
            VaultCreationStep::ScanCoownerKeys => {
                steps::render_scan_coowner_keys(ui, &ctx, state);
            }
            VaultCreationStep::DisplayOwnKeys => {
                steps::render_display_own_keys(ui, &ctx, state);
            }
            VaultCreationStep::EnterExchangeData => {
                steps::render_enter_exchange_data(ui, &ctx, state);
            }
            VaultCreationStep::EmailAuth => {
                steps::render_email_auth(ui, app_state, state);
            }
            VaultCreationStep::CreateVault => {
                steps::render_create_vault(ui, app_state, navigation, state);
            }
            VaultCreationStep::DisplayExchangeData => {
                steps::render_display_exchange_data(ui, &ctx, state);
            }
            VaultCreationStep::Completed => {
                steps::render_completed(ui, navigation, state);
            }
            // View-Only flow
            VaultCreationStep::ScanDescriptorViewOnly => {
                steps::render_scan_descriptor_view_only(ui, state);
            }
            VaultCreationStep::ViewOnlyComplete => {
                steps::render_view_only_complete(ui, app_state, navigation, state);
            }
            // Restore from Backup flow
            VaultCreationStep::EnterSeedPhrase => {
                steps::render_enter_seed_phrase(ui, state);
            }
            VaultCreationStep::ScanDescriptorRestore => {
                steps::render_scan_descriptor_restore(ui, app_state, navigation, state);
            }
        }
    });
}
