//! Card Component
//!
//! Modern card container with rounded corners and shadow

use eframe::egui;
use super::theme::{Colors, Spacing};

/// Render a modern card container
pub fn card<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let padding = Spacing::MD;
    let corner_radius = 12.0;
    
    let bg_color = Colors::bg_card(ui.ctx());
    let border_color = if ui.ctx().style().visuals.dark_mode {
        Colors::GRAY_700
    } else {
        Colors::GRAY_200
    };

    // Allocate space for card
    let available_width = ui.available_width();
    let min_height = 100.0; // Minimum card height
    let (rect, response) = ui.allocate_at_least(
        egui::Vec2::new(available_width, min_height),
        egui::Sense::click()
    );

    // Draw card background with rounded corners
    ui.painter().rect_filled(
        rect,
        corner_radius,
        bg_color,
    );

    // Draw subtle border
    ui.painter().rect_stroke(
        rect,
        corner_radius,
        egui::Stroke::new(1.0, border_color),
    );

    // Add content with padding
    let content_rect = rect.shrink(padding);
    let mut content_ui = ui.child_ui(content_rect, *ui.layout());
    let inner_response = add_contents(&mut content_ui);

    egui::InnerResponse {
        inner: inner_response,
        response,
    }
}

// Note: card_with_header removed for now due to type complexity
// Can be added back when needed with proper type handling
