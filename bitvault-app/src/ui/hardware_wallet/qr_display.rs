//! QR Code Display Component
//!
//! Displays multi-part UR QR codes for hardware wallets to scan

use crate::state::{AppState, Navigation};
use crate::utils::qr::generate_qr_image;
use eframe::egui;

/// State for QR code display
#[derive(Default)]
pub struct QrDisplayState {
    pub ur_parts: Vec<String>,
    pub current_part: usize,
    pub is_animating: bool,
    pub error: Option<String>,
}

/// Render QR code display for hardware wallet scanning
pub fn render_qr_display(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut QrDisplayState,
    title: &str,
    description: &str,
) {
    ui.vertical_centered(|ui| {
        ui.heading(title);
        ui.add_space(10.0);
        ui.label(description);
        ui.add_space(20.0);

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }

        // Display QR code
        if state.ur_parts.is_empty() {
            ui.label("No QR codes to display");
        } else {
            // Show current part indicator
            ui.label(format!(
                "Part {} of {}",
                state.current_part + 1,
                state.ur_parts.len()
            ));
            ui.add_space(10.0);

            // Generate and display QR code image
            let current_ur = &state.ur_parts[state.current_part];

            // Generate QR code texture
            if let Some(qr_texture) = generate_qr_image(ui.ctx(), current_ur) {
                // Display QR code image
                let size = egui::Vec2::new(300.0, 300.0);
                ui.image((qr_texture.id(), size));
                ui.add_space(10.0);
            } else {
                // Fallback if QR generation fails
                ui.colored_label(egui::Color32::YELLOW, "Failed to generate QR code");
                ui.label(format!(
                    "UR String: {}...",
                    &current_ur[..current_ur.len().min(50)]
                ));
            }

            ui.add_space(10.0);

            // Show full UR string in a scrollable area (for manual entry if needed)
            ui.collapsing("Show UR String", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.label(current_ur);
                    });
            });

            ui.add_space(10.0);

            // Navigation buttons for multi-part QR codes
            if state.ur_parts.len() > 1 {
                ui.horizontal(|ui| {
                    if ui.button("← Previous").clicked() && state.current_part > 0 {
                        state.current_part -= 1;
                    }
                    if ui.button("Next →").clicked()
                        && state.current_part < state.ur_parts.len() - 1
                    {
                        state.current_part += 1;
                    }
                });

                ui.add_space(10.0);

                // Auto-advance checkbox
                ui.checkbox(&mut state.is_animating, "Auto-advance through parts");
            }

            ui.add_space(20.0);

            // Copy button
            if ui.button("Copy UR String").clicked() {
                ui.output_mut(|o| {
                    o.copied_text = current_ur.clone();
                });
            }
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Done button
        if ui.button("Done").clicked() {
            navigation.go_back();
        }
    });
}
