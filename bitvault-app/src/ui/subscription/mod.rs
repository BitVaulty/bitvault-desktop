//! Subscription Management UI
//!
//! Handles:
//! - Subscription status display
//! - Subscription validation
//! - Subscription renewal/upgrade prompts

mod status_display;
mod validation;


use eframe::egui;
use crate::state::{AppState, Navigation};

/// Render subscription management UI
pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical_centered(|ui| {
        ui.heading("Subscription");
        ui.add_space(20.0);

        if !app_state.is_vault_loaded() {
            ui.label("No vault loaded");
            if ui.button("Back").clicked() {
                navigation.go_back();
            }
            return;
        }

        status_display::render(ui, app_state, navigation);
    });
}
