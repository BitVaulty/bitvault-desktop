//! PIN Setup UI
//!
//! Screen for setting up a new PIN during vault creation

use bitvault_common::PinService;
use eframe::egui;

/// State for PIN setup
pub struct PinSetupState {
    pin: String,
    confirm_pin: String,
    step: PinSetupStep,
    error: Option<String>,
    /// Track if we've auto-focused the first button (to avoid doing it every frame)
    has_auto_focused: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PinSetupStep {
    EnterPin,
    ConfirmPin,
}

impl Default for PinSetupState {
    fn default() -> Self {
        Self {
            pin: String::new(),
            confirm_pin: String::new(),
            step: PinSetupStep::EnterPin,
            error: None,
            has_auto_focused: false,
        }
    }
}

impl PinSetupState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.pin.clear();
        self.confirm_pin.clear();
        self.step = PinSetupStep::EnterPin;
        self.error = None;
        self.has_auto_focused = false;
    }
}

/// Render PIN setup screen
/// Returns true if PIN was successfully set
pub fn render_pin_setup(
    ui: &mut egui::Ui,
    state: &mut PinSetupState,
    _on_pin_set: &mut Option<Box<dyn FnMut()>>,
) -> bool {
    let mut pin_set = false;

    // Use ScrollArea to prevent content from flowing off screen
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                match state.step {
                    PinSetupStep::EnterPin => {
                        // Fixed height container to ensure consistent numpad positioning
                        let content_height = 79.0; // Total height: 5 + 20 (label) + 15 + 24 (PIN) + 15
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(ui.available_width(), content_height),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.add_space(5.0);
                                ui.label("Enter a 6-digit PIN to secure your wallet");
                                ui.add_space(15.0);

                                // PIN input display
                                let pin_display = "•".repeat(state.pin.len());
                                ui.label(egui::RichText::new(pin_display).size(24.0).monospace());
                            },
                        );

                        ui.add_space(15.0);

                        // Number pad - centered, all buttons in same UI context for proper tab order
                        let row_width = 190.0;
                        let available_width = ui.available_width();
                        let left_margin = ((available_width - row_width) / 2.0).max(0.0);

                        // Row 1: 1, 2, 3
                        ui.horizontal(|ui| {
                            ui.add_space(left_margin);
                            let button1_response = render_number_button(ui, "1", &mut state.pin);

                            // Auto-focus first button when entering step (only once)
                            if !state.has_auto_focused && state.pin.is_empty() {
                                button1_response.request_focus();
                                state.has_auto_focused = true;
                            }

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
                        // Last row: 0, DEL
                        let last_row_width = 125.0;
                        let last_left_margin = ((available_width - last_row_width) / 2.0).max(0.0);
                        ui.horizontal(|ui| {
                            ui.add_space(last_left_margin);
                            render_number_button(ui, "0", &mut state.pin);
                            ui.add_space(5.0);
                            render_del_button(ui, &mut state.pin);
                        });

                        // Validate PIN format when 6 digits entered
                        if state.pin.len() == 6 {
                            if !is_valid_pin(&state.pin) {
                                state.error = Some(
                                    "PIN must contain at least 4 different digits".to_string(),
                                );
                                state.pin.clear();
                            } else {
                                // Move to confirmation step
                                state.step = PinSetupStep::ConfirmPin;
                                state.error = None;
                                state.has_auto_focused = false; // Reset for confirm step
                            }
                        }
                    }
                    PinSetupStep::ConfirmPin => {
                        // Fixed height container to ensure consistent numpad positioning
                        // Same height as EnterPin step (79px)
                        let content_height = 79.0;
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(ui.available_width(), content_height),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.add_space(5.0);
                                ui.label("Re-enter your PIN to confirm");

                                // Show error if any - use smaller spacing to fit in fixed height
                                if let Some(ref error) = state.error {
                                    ui.add_space(5.0);
                                    ui.colored_label(egui::Color32::RED, error);
                                } else {
                                    ui.add_space(15.0);
                                }

                                // PIN input display
                                let pin_display = "•".repeat(state.confirm_pin.len());
                                ui.label(egui::RichText::new(pin_display).size(24.0).monospace());
                            },
                        );

                        ui.add_space(15.0);

                        // Number pad - centered, all buttons in same UI context for proper tab order
                        let row_width = 190.0;
                        let available_width = ui.available_width();
                        let left_margin = ((available_width - row_width) / 2.0).max(0.0);

                        // Row 1: 1, 2, 3
                        ui.horizontal(|ui| {
                            ui.add_space(left_margin);
                            let button1_response =
                                render_number_button(ui, "1", &mut state.confirm_pin);

                            // Auto-focus first button when entering confirm step (only once)
                            if !state.has_auto_focused && state.confirm_pin.is_empty() {
                                button1_response.request_focus();
                                state.has_auto_focused = true;
                            }

                            ui.add_space(5.0);
                            render_number_button(ui, "2", &mut state.confirm_pin);
                            ui.add_space(5.0);
                            render_number_button(ui, "3", &mut state.confirm_pin);
                        });

                        ui.add_space(5.0);
                        // Row 2: 4, 5, 6
                        ui.horizontal(|ui| {
                            ui.add_space(left_margin);
                            render_number_button(ui, "4", &mut state.confirm_pin);
                            ui.add_space(5.0);
                            render_number_button(ui, "5", &mut state.confirm_pin);
                            ui.add_space(5.0);
                            render_number_button(ui, "6", &mut state.confirm_pin);
                        });

                        ui.add_space(5.0);
                        // Row 3: 7, 8, 9
                        ui.horizontal(|ui| {
                            ui.add_space(left_margin);
                            render_number_button(ui, "7", &mut state.confirm_pin);
                            ui.add_space(5.0);
                            render_number_button(ui, "8", &mut state.confirm_pin);
                            ui.add_space(5.0);
                            render_number_button(ui, "9", &mut state.confirm_pin);
                        });

                        ui.add_space(5.0);
                        // Last row: 0, DEL
                        let last_row_width = 125.0;
                        let last_left_margin = ((available_width - last_row_width) / 2.0).max(0.0);
                        ui.horizontal(|ui| {
                            ui.add_space(last_left_margin);
                            render_number_button(ui, "0", &mut state.confirm_pin);
                            ui.add_space(5.0);
                            render_del_button(ui, &mut state.confirm_pin);
                        });

                        // Validate when confirm PIN is 6 digits
                        if state.confirm_pin.len() == 6 {
                            if state.confirm_pin == state.pin {
                                // PINs match - save it
                                let pin_service = PinService::new();
                                match pin_service.save_pin(&state.pin) {
                                    Ok(()) => {
                                        state.clear();
                                        pin_set = true;
                                    }
                                    Err(e) => {
                                        state.error = Some(format!("Failed to save PIN: {}", e));
                                        state.confirm_pin.clear();
                                    }
                                }
                            } else {
                                // PINs don't match
                                state.error =
                                    Some("PINs do not match. Please try again.".to_string());
                                state.pin.clear();
                                state.confirm_pin.clear();
                                state.step = PinSetupStep::EnterPin;
                            }
                        }
                    }
                }
            });
        });

    pin_set
}

fn render_number_button(ui: &mut egui::Ui, num: &str, pin: &mut String) -> egui::Response {
    let button = ui.add_sized(
        [60.0, 60.0],
        egui::Button::new(egui::RichText::new(num).size(24.0)),
    );
    if button.clicked() && pin.len() < 6 {
        pin.push_str(num);
    }
    button
}

fn render_del_button(ui: &mut egui::Ui, pin: &mut String) {
    // DEL button - using emoji/Unicode that should work with default fonts
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

/// Check if PIN is valid (security requirements)
/// PIN must have at least 4 different digits (to prevent weak PINs like 111111)
fn is_valid_pin(pin: &str) -> bool {
    if pin.len() != 6 {
        return false;
    }

    let unique_digits: std::collections::HashSet<char> = pin.chars().collect();
    unique_digits.len() >= 4
}
