//! Subscription Management UI
//!
//! Handles:
//! - Subscription status display
//! - Subscription validation
//! - Subscription renewal/upgrade prompts

mod status_display;
pub mod validation;

use crate::state::{AppState, Navigation};
use eframe::egui;

/// Render subscription management UI
pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical_centered(|ui| {
        ui.heading("Subscription");
        ui.add_space(20.0);

        if !app_state.is_vault_loaded() {
            ui.label("No vault loaded");
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button("Back").clicked() {
                    navigation.go_back();
                }
            });
            return;
        }

        status_display::render(ui, app_state, navigation);
    });
}
