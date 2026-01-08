//! PIN Entry UI
//!
//! Screen for entering PIN to authenticate
//! Supports biometric authentication as an alternative to PIN entry

use crate::services::biometric_service::{BiometricResult, BiometricService};
use bitvault_common::PinService;
use eframe::egui;
use std::path::PathBuf;

/// Load the BitVault logo for display
fn load_bitvault_logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let mut possible_paths = vec![
        // Relative to workspace root
        PathBuf::from("bitvault-desktop/bitvault-app/resources/bitvault_logo.png"),
        PathBuf::from("bitvault-desktop/bitvault-app/resources/bitvault_logo.svg"),
        // Relative to current working directory
        PathBuf::from("resources/bitvault_logo.png"),
        PathBuf::from("resources/bitvault_logo.svg"),
        PathBuf::from("bitvault-app/resources/bitvault_logo.png"),
        PathBuf::from("bitvault-app/resources/bitvault_logo.svg"),
    ];
    
    // Add executable-relative paths
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // Try resources next to executable
            possible_paths.push(exe_dir.join("resources/bitvault_logo.png"));
            possible_paths.push(exe_dir.join("resources/bitvault_logo.svg"));
            
            // If we're in target/release, go up to find bitvault-app/resources
            let mut current = exe_dir;
            while let Some(parent) = current.parent() {
                // Check if we're in the bitvault-desktop directory structure
                let bitvault_app_resources = parent.join("bitvault-app/resources/bitvault_logo.png");
                if bitvault_app_resources.exists() {
                    possible_paths.push(bitvault_app_resources.clone());
                    possible_paths.push(parent.join("bitvault-app/resources/bitvault_logo.svg"));
                    break;
                }
                // Stop if we've gone too far up (reached root or workspace)
                if parent == current || !parent.exists() {
                    break;
                }
                current = parent;
            }
        }
    }
    
    for path in possible_paths.iter() {
        if path.exists() {
            if let Some(texture) = crate::utils::images::load_image_texture(ctx, path) {
                return Some(texture);
            }
        }
    }
    None
}

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
        // Display BitVault logo
        if let Some(logo_texture) = load_bitvault_logo(_ctx) {
            let logo_size = 200.0; // Size for the bigger logo
            let texture_size = logo_texture.size_vec2();
            let aspect_ratio = texture_size.y / texture_size.x;
            // Use Image widget - bg_fill(TRANSPARENT) makes the image background transparent
            // The panel behind will show through, matching the app theme
            ui.add(
                egui::Image::from_texture((logo_texture.id(), egui::Vec2::new(logo_size, logo_size * aspect_ratio)))
                    .bg_fill(egui::Color32::TRANSPARENT)
            );
            ui.add_space(20.0);
        }
        
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

        // Number pad - centered (same container as text above)
        // Calculate width: 3 buttons (60px each) + 2 spaces (5px each) = 190px
        let row_width = 190.0;
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(row_width, 60.0),
            egui::Sense::click()
        );
        let mut row_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        render_number_button(&mut row_ui, "1", &mut state.pin);
        row_ui.add_space(5.0);
        render_number_button(&mut row_ui, "2", &mut state.pin);
        row_ui.add_space(5.0);
        render_number_button(&mut row_ui, "3", &mut state.pin);
        
        ui.add_space(5.0);
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(row_width, 60.0),
            egui::Sense::click()
        );
        let mut row_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        render_number_button(&mut row_ui, "4", &mut state.pin);
        row_ui.add_space(5.0);
        render_number_button(&mut row_ui, "5", &mut state.pin);
        row_ui.add_space(5.0);
        render_number_button(&mut row_ui, "6", &mut state.pin);
        
        ui.add_space(5.0);
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(row_width, 60.0),
            egui::Sense::click()
        );
        let mut row_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        render_number_button(&mut row_ui, "7", &mut state.pin);
        row_ui.add_space(5.0);
        render_number_button(&mut row_ui, "8", &mut state.pin);
        row_ui.add_space(5.0);
        render_number_button(&mut row_ui, "9", &mut state.pin);
        
        ui.add_space(5.0);
        // Last row: 2 buttons (60px each) + 1 space (5px) = 125px
        let last_row_width = 125.0;
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(last_row_width, 60.0),
            egui::Sense::click()
        );
        let mut row_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        render_number_button(&mut row_ui, "0", &mut state.pin);
        row_ui.add_space(5.0);
        render_del_button(&mut row_ui, &mut state.pin);

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
                    // Handle rate limiting error specifically
                    let error_msg = match &e {
                        bitvault_common::PinServiceError::RateLimited(seconds) => {
                            let minutes = seconds / 60;
                            format!(
                                "Too many failed attempts. Please try again in {} minute(s).",
                                minutes
                            )
                        }
                        _ => format!("Error validating PIN: {}", e),
                    };
                    state.error = Some(error_msg);
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

fn render_number_button(ui: &mut egui::Ui, num: &str, pin: &mut String) {
    let button = ui.add_sized([60.0, 60.0], egui::Button::new(
        egui::RichText::new(num).size(24.0)
    ));
    if button.clicked() && pin.len() < 6 {
        pin.push_str(num);
    }
}

fn render_del_button(ui: &mut egui::Ui, pin: &mut String) {
    let button = ui.add_sized([60.0, 60.0], egui::Button::new(
        egui::RichText::new("Del").size(20.0)
    ));
    if button.clicked() {
        pin.pop();
    }
}
