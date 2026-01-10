//! PIN verification for sensitive operations
//!
//! Provides a modal/overlay for PIN verification before performing
//! sensitive operations like sending transactions or deleting vaults

use crate::ui::pin::PinEntryState;
use bitvault_common::PinService;
use eframe::egui;

/// State for PIN verification modal
pub struct PinVerificationState {
    pin_entry: PinEntryState,
    is_visible: bool,
    verified: bool,
    error: Option<String>,
}

impl Default for PinVerificationState {
    fn default() -> Self {
        Self {
            pin_entry: PinEntryState::new(),
            is_visible: false,
            verified: false,
            error: None,
        }
    }
}

impl PinVerificationState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the PIN verification modal
    pub fn show(&mut self) {
        self.is_visible = true;
        self.verified = false;
        self.pin_entry.clear();
        self.error = None;
    }

    /// Hide the PIN verification modal
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.verified = false;
        self.pin_entry.clear();
        self.error = None;
    }

    /// Check if PIN is verified
    pub fn is_verified(&self) -> bool {
        self.verified
    }

    /// Check if modal is visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Reset verification state (after operation completes)
    pub fn reset(&mut self) {
        self.verified = false;
        self.pin_entry.clear();
        self.error = None;
    }
}

/// Render PIN verification modal
/// Returns true if PIN is verified, false otherwise
pub fn render_pin_verification(ctx: &egui::Context, state: &mut PinVerificationState) -> bool {
    if !state.is_visible {
        return false;
    }

    // Create modal window
    egui::Window::new("Verify PIN")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Verify PIN to Continue");
                ui.add_space(10.0);
                ui.label("Please enter your PIN to confirm this action");
                ui.add_space(20.0);

                // Show error if any
                if let Some(ref error) = state.error {
                    ui.colored_label(egui::Color32::RED, error);
                    ui.add_space(10.0);
                }

                // PIN input display
                let pin_display = "•".repeat(state.pin_entry.pin.len());
                ui.label(egui::RichText::new(pin_display).size(24.0).monospace());

                ui.add_space(20.0);

                // Number pad - centered (same container as text above)
                let row_width = 190.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(row_width, 60.0),
                    egui::Sense::click()
                );
                let mut row_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                if render_number_button(&mut row_ui, "1", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                row_ui.add_space(5.0);
                if render_number_button(&mut row_ui, "2", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                row_ui.add_space(5.0);
                if render_number_button(&mut row_ui, "3", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                
                ui.add_space(5.0);
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(row_width, 60.0),
                    egui::Sense::click()
                );
                let mut row_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                if render_number_button(&mut row_ui, "4", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                row_ui.add_space(5.0);
                if render_number_button(&mut row_ui, "5", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                row_ui.add_space(5.0);
                if render_number_button(&mut row_ui, "6", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                
                ui.add_space(5.0);
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(row_width, 60.0),
                    egui::Sense::click()
                );
                let mut row_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                if render_number_button(&mut row_ui, "7", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                row_ui.add_space(5.0);
                if render_number_button(&mut row_ui, "8", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                row_ui.add_space(5.0);
                if render_number_button(&mut row_ui, "9", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                
                ui.add_space(5.0);
                let last_row_width = 125.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(last_row_width, 60.0),
                    egui::Sense::click()
                );
                let mut row_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
                if render_number_button(&mut row_ui, "0", &mut state.pin_entry.pin) {
                    state.error = None;
                }
                row_ui.add_space(5.0);
                if render_del_button(&mut row_ui, &mut state.pin_entry.pin) {
                    state.error = None;
                }

                ui.add_space(20.0);

                // Validate when PIN is 6 digits
                if state.pin_entry.pin.len() == 6 && !state.pin_entry.is_validating {
                    state.pin_entry.is_validating = true;
                    let pin_clone = state.pin_entry.pin.clone();

                    // Validate PIN
                    let pin_service = PinService::new();
                    match pin_service.validate_pin(&pin_clone) {
                        Ok(true) => {
                            // PIN is valid
                            state.verified = true;
                            state.pin_entry.clear();
                        }
                        Ok(false) => {
                            // PIN is invalid
                            state.error = Some("Invalid PIN. Please try again.".to_string());
                            state.pin_entry.pin.clear();
                            state.pin_entry.is_validating = false;
                        }
                        Err(e) => {
                            // Handle rate limiting error specifically
                            let error_msg = match &e {
                                bitvault_common::PinServiceError::RateLimited(seconds) => {
                                    let minutes = seconds / 60;
                                    format!("Too many failed attempts. Please try again in {} minute(s).", minutes)
                                }
                                _ => format!("Error validating PIN: {}", e),
                            };
                            state.error = Some(error_msg);
                            state.pin_entry.pin.clear();
                            state.pin_entry.is_validating = false;
                        }
                    }
                }

                if state.pin_entry.is_validating {
                    ui.label("Validating...");
                }

                ui.add_space(20.0);

                // Cancel button
                if ui.button("Cancel").clicked() {
                    state.hide();
                }
            });
        });

    // Return verification status
    if state.verified {
        state.hide();
        true
    } else {
        false
    }
}

fn render_number_button(ui: &mut egui::Ui, num: &str, pin: &mut String) -> bool {
    let button = ui.add_sized([60.0, 60.0], egui::Button::new(
        egui::RichText::new(num).size(24.0)
    ));
    if button.clicked() && pin.len() < 6 {
        pin.push_str(num);
        true
    } else {
        false
    }
}

fn render_del_button(ui: &mut egui::Ui, pin: &mut String) -> bool {
    // DEL button - using emoji/Unicode that should work with default fonts
    let button = ui.add_sized([60.0, 60.0], egui::Button::new(
        egui::RichText::new("⌫").size(24.0) // Unicode BACKSPACE symbol
    ));
    if button.clicked() {
        pin.pop();
        true
    } else {
        false
    }
}
