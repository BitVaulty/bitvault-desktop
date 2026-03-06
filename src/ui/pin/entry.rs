//! PIN Entry UI
//!
//! Screen for entering PIN to authenticate
//! Supports biometric authentication as an alternative to PIN entry

use crate::services::biometric_service::{BiometricResult, BiometricService};
use bitvault_common::PinService;
use eframe::egui;

/// State for PIN entry
pub struct PinEntryState {
    pub pin: String,
    pub error: Option<String>,
    pub is_validating: bool,
    pub biometric_service: BiometricService,
    pub biometric_available: bool,
    pub biometric_type: crate::services::biometric_service::BiometricType,
    pub biometric_attempted: bool,
}

impl Default for PinEntryState {
    fn default() -> Self {
        let biometric_service = BiometricService::new();
        Self {
            pin: String::new(),
            error: None,
            is_validating: false,
            biometric_service,
            biometric_available: false,
            biometric_type: crate::services::biometric_service::BiometricType::None,
            biometric_attempted: false,
        }
    }
}

impl PinEntryState {
    pub fn new() -> Self {
        let mut state = Self::default();
        // Check biometric availability on initialization
        // Note: This creates a temporary runtime since app_state runtime isn't available yet
        // This is acceptable for initialization, but operations during rendering should use app_state runtime
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            state.biometric_available = rt.block_on(state.biometric_service.is_available());
            state.biometric_type = rt.block_on(state.biometric_service.get_biometric_type());
        } else {
            // If runtime creation fails, biometrics are unavailable
            state.biometric_available = false;
            state.biometric_type = crate::services::biometric_service::BiometricType::None;
        }
        state
    }

    pub fn clear(&mut self) {
        self.pin.clear();
        self.error = None;
        self.is_validating = false;
        self.biometric_attempted = false;
    }

    /// Attempt biometric authentication
    pub async fn try_biometric(&mut self) -> bool {
        if !self.biometric_available {
            return false;
        }

        if !self.biometric_service.is_enabled().await {
            return false;
        }

        self.biometric_attempted = true;
        let reason = format!("Authenticate using {}", self.biometric_type.display_name());

        match self.biometric_service.authenticate(&reason).await {
            BiometricResult::Success => {
                self.error = None;
                true
            }
            BiometricResult::Cancelled => {
                self.error = None; // User cancelled, not an error
                false
            }
            BiometricResult::Failed(e) => {
                self.error = Some(format!("Biometric authentication failed: {}", e));
                false
            }
            BiometricResult::NotAvailable | BiometricResult::NotEnrolled => {
                self.error = Some(format!(
                    "{} is not available or not enrolled",
                    self.biometric_type.display_name()
                ));
                false
            }
        }
    }
}

/// Render PIN entry screen
/// Returns true if PIN was successfully validated or biometric authentication succeeded
pub fn render_pin_entry(
    ui: &mut egui::Ui,
    state: &mut PinEntryState,
    _on_pin_validated: &mut Option<Box<dyn FnMut()>>,
    _ctx: &egui::Context,
    runtime: Option<&tokio::runtime::Runtime>,
) -> bool {
    let mut pin_validated = false;

    // Try biometric authentication on first render if available and enabled
    // Note: This is commented out for now as it requires platform-specific implementation
    // Uncomment when biometrics crate is available
    // if state.biometric_available && !state.biometric_attempted {
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     if rt.block_on(state.try_biometric()) {
    //         return true; // Biometric authentication succeeded
    //     }
    // }

    ui.vertical_centered(|ui| {
        ui.heading("Enter PIN");
        ui.add_space(20.0);

        // Show biometric option if available
        if state.biometric_available {
            ui.label(format!(
                "Or use {} to authenticate",
                state.biometric_type.display_name()
            ));
            if ui
                .button(format!("Use {}", state.biometric_type.display_name()))
                .clicked()
            {
                if let Some(rt) = runtime {
                    if rt.block_on(state.try_biometric()) {
                        pin_validated = true;
                    }
                } else {
                    state.error = Some("Runtime not available".to_string());
                }
            }
            ui.add_space(10.0);
        }

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }

        // PIN input field (masked)
        ui.label("Enter your 6-digit PIN:");
        ui.add_space(10.0);

        // Display PIN as dots
        let pin_display = "•".repeat(state.pin.len());
        ui.label(egui::RichText::new(pin_display).size(24.0).monospace());

        ui.add_space(20.0);

        // Number pad - centered, all buttons in same UI context for proper tab order
        // Calculate width: 3 buttons (60px each) + 2 spaces (5px each) = 190px
        let row_width = 190.0;
        let available_width = ui.available_width();
        let left_margin = ((available_width - row_width) / 2.0).max(0.0);

        // Row 1: 1, 2, 3
        ui.horizontal(|ui| {
            ui.add_space(left_margin);
            render_number_button(ui, "1", &mut state.pin);
            ui.add_space(5.0);
            render_number_button(ui, "2", &mut state.pin);
            ui.add_space(5.0);
            render_number_button(ui, "3", &mut state.pin);
        });

        ui.add_space(5.0);
        // Row 2: 4, 5, 6
        ui.horizontal(|ui| {
            ui.add_space(left_margin);
            render_number_button(ui, "4", &mut state.pin);
            ui.add_space(5.0);
            render_number_button(ui, "5", &mut state.pin);
            ui.add_space(5.0);
            render_number_button(ui, "6", &mut state.pin);
        });

        ui.add_space(5.0);
        // Row 3: 7, 8, 9
        ui.horizontal(|ui| {
            ui.add_space(left_margin);
            render_number_button(ui, "7", &mut state.pin);
            ui.add_space(5.0);
            render_number_button(ui, "8", &mut state.pin);
            ui.add_space(5.0);
            render_number_button(ui, "9", &mut state.pin);
        });

        ui.add_space(5.0);
        // Last row: 0, DEL (centered differently)
        let last_row_width = 125.0;
        let last_left_margin = ((available_width - last_row_width) / 2.0).max(0.0);
        ui.horizontal(|ui| {
            ui.add_space(last_left_margin);
            render_number_button(ui, "0", &mut state.pin);
            ui.add_space(5.0);
            render_del_button(ui, &mut state.pin);
        });

        ui.add_space(20.0);

        // Validate when PIN is 6 digits
        if state.pin.len() == 6 && !state.is_validating {
            state.is_validating = true;
            let pin_clone = state.pin.clone();

            // Validate PIN asynchronously
            let pin_service = PinService::new();

            // Check if PIN exists first
            if !pin_service.has_pin() {
                eprintln!("[PIN] No PIN set, but user entered PIN. This shouldn't happen.");
                state.error = Some("No PIN is set. Please set up a PIN first.".to_string());
                state.pin.clear();
                state.is_validating = false;
            } else {
                eprintln!("[PIN] Attempting to validate PIN...");
                match pin_service.validate_pin(&pin_clone) {
                    Ok(true) => {
                        // PIN is valid
                        state.clear();
                        pin_validated = true;
                    }
                    Ok(false) => {
                        // PIN is invalid
                        state.error = Some("Invalid PIN. Please try again.".to_string());
                        state.pin.clear();
                        state.is_validating = false;
                    }
                    Err(e) => {
                        // Handle errors with better messages
                        let error_msg = match &e {
                            bitvault_common::PinServiceError::RateLimited(seconds) => {
                                let minutes = seconds / 60;
                                format!(
                                    "Too many failed attempts. Please try again in {} minute(s).",
                                    minutes
                                )
                            }
                            bitvault_common::PinServiceError::DecryptionFailed => {
                                eprintln!("[PIN_ERROR] Decryption failed - encryption key may be missing or wrong");
                                "Decryption failed. The encryption key may be missing or corrupted. Click 'Reset PIN' below to delete the corrupted PIN and start fresh.".to_string()
                            }
                            bitvault_common::PinServiceError::PinNotFound => {
                                eprintln!("[PIN_ERROR] PIN not found in storage");
                                "No PIN is set. Please set up a PIN first.".to_string()
                            }
                            _ => {
                                eprintln!("[PIN_ERROR] Validation error: {:?}", e);
                                format!("Error validating PIN: {}", e)
                            }
                        };
                        state.error = Some(error_msg);
                        state.pin.clear();
                        state.is_validating = false;
                    }
                }
            }
        }

        if state.is_validating {
            ui.label("Validating...");
        }

        // Show reset PIN option if there's a decryption error
        if let Some(ref error) = state.error {
            if error.contains("Decryption failed") {
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "If you cannot recover your encryption key, you can reset your PIN:"
                );
                ui.add_space(10.0);
                if ui.button("Reset PIN (Delete Corrupted PIN)").clicked() {
                    let pin_service = PinService::new();
                    match pin_service.delete_pin() {
                        Ok(_) => {
                            eprintln!("[PIN_RESET] PIN successfully deleted from PIN entry screen");
                            state.clear();
                            pin_validated = true; // Allow user to proceed
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to reset PIN: {}", e));
                            eprintln!("[PIN_RESET] Failed to delete PIN: {:?}", e);
                        }
                    }
                }
            }
        }
    });

    pin_validated
}

fn render_number_button(ui: &mut egui::Ui, num: &str, pin: &mut String) {
    let button = ui.add_sized(
        [60.0, 60.0],
        egui::Button::new(egui::RichText::new(num).size(24.0)),
    );
    if button.clicked() && pin.len() < 6 {
        pin.push_str(num);
    }
}

fn render_del_button(ui: &mut egui::Ui, pin: &mut String) {
    // DEL button - using emoji/Unicode that should work with default fonts
    // Try backspace emoji or arrow - fallback to text if not supported
    let button = ui.add_sized(
        [60.0, 60.0],
        egui::Button::new(
            egui::RichText::new("⌫").size(24.0), // Unicode BACKSPACE symbol
        ),
    );
    if button.clicked() {
        pin.pop();
    }
}
