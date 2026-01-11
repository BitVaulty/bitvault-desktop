//! Test helpers for e2e testing

use eframe::egui;

/// Helper to create a test egui context with default settings
pub fn create_test_context() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.set_visuals(egui::Visuals::dark());
    ctx
}

/// Helper to create a test egui context with light mode
pub fn create_test_context_light() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.set_visuals(egui::Visuals::light());
    ctx
}

/// Helper to simulate a click on a UI element
pub fn simulate_click(ui: &mut egui::Ui, id: egui::Id) -> bool {
    let response = ui.interact(ui.available_rect_before_wrap(), id, egui::Sense::click());
    response.clicked()
}

/// Helper to simulate text input
pub fn simulate_text_input(ui: &mut egui::Ui, text: &mut String, id: egui::Id) {
    let response = ui.text_edit_singleline(text);
    // In a real test, we'd simulate keyboard input
    // For now, we just verify the widget exists
}

/// Helper to capture UI output for verification
pub struct UICapture {
    pub labels: Vec<String>,
    pub buttons: Vec<String>,
    pub errors: Vec<String>,
}

impl UICapture {
    pub fn new() -> Self {
        Self {
            labels: Vec::new(),
            buttons: Vec::new(),
            errors: Vec::new(),
        }
    }
}
