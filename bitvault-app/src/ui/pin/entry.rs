//! PIN Entry UI
//!
//! Screen for entering PIN to authenticate
//! Supports biometric authentication as an alternative to PIN entry

use eframe::egui;
use bitvault_common::PinService;
use crate::services::biometric_service::{BiometricService, BiometricResult};

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
        let rt = tokio::runtime::Runtime::new().unwrap();
        state.biometric_available = rt.block_on(state.biometric_service.is_available());
        state.biometric_type = rt.block_on(state.biometric_service.get_biometric_type());
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
                self.error = Some(format!("{} is not available or not enrolled", self.biometric_type.display_name()));
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
            ui.label(format!("Or use {} to authenticate", state.biometric_type.display_name()));
            if ui.button(format!("Use {}", state.biometric_type.display_name())).clicked() {
                let rt = tokio::runtime::Runtime::new().unwrap();
                if rt.block_on(state.try_biometric()) {
                    pin_validated = true;
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

        // Number pad
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                render_number_row(ui, &["1", "2", "3"], &mut state.pin);
                render_number_row(ui, &["4", "5", "6"], &mut state.pin);
                render_number_row(ui, &["7", "8", "9"], &mut state.pin);
                ui.horizontal(|ui| {
                    if ui.button("0").clicked() && state.pin.len() < 6 {
                        state.pin.push('0');
                        state.error = None;
                    }
                    if ui.button("⌫").clicked() {
                        state.pin.pop();
                        state.error = None;
                    }
                });
            });
        });

        ui.add_space(20.0);

        // Validate when PIN is 6 digits
        if state.pin.len() == 6 && !state.is_validating {
            state.is_validating = true;
            let pin_clone = state.pin.clone();
            
            // Validate PIN asynchronously
            let pin_service = PinService::new();
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
                    state.error = Some(format!("Error validating PIN: {}", e));
                    state.pin.clear();
                    state.is_validating = false;
                }
            }
        }

        if state.is_validating {
            ui.label("Validating...");
        }
    });
    
    pin_validated
}

fn render_number_row(ui: &mut egui::Ui, numbers: &[&str], pin: &mut String) {
    ui.horizontal(|ui| {
        for num in numbers {
            if ui.button(*num).clicked() && pin.len() < 6 {
                pin.push_str(num);
            }
        }
    });
}
