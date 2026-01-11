//! Badge Component
//!
//! Status badges and labels with modern styling

use super::theme::{Colors, Spacing};
use eframe::egui;

/// Badge style variants
#[derive(Clone, Copy)]
pub enum BadgeStyle {
    Success,
    Warning,
    Error,
    Info,
    Neutral,
}

impl BadgeStyle {
    fn colors(&self) -> (egui::Color32, egui::Color32) {
        match self {
            BadgeStyle::Success => (Colors::SUCCESS_LIGHT, Colors::SUCCESS_DARK),
            BadgeStyle::Warning => (Colors::WARNING_LIGHT, Colors::WARNING_DARK),
            BadgeStyle::Error => (Colors::ERROR_LIGHT, Colors::ERROR_DARK),
            BadgeStyle::Info => (Colors::PRIMARY_LIGHT, Colors::PRIMARY_DARK),
            BadgeStyle::Neutral => (Colors::GRAY_200, Colors::GRAY_700),
        }
    }
}

/// Render a status badge
pub fn badge(ui: &mut egui::Ui, text: impl Into<String>, style: BadgeStyle) {
    let text = text.into();
    let (bg_color, text_color) = style.colors();

    let padding = egui::Vec2::new(Spacing::SM, Spacing::XS);
    let corner_radius = 6.0;

    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let galley = ui.fonts(|f| f.layout_no_wrap(text.clone(), font_id, text_color));

    let size = galley.size() + padding * 2.0;
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

    // Draw badge background
    ui.painter().rect_filled(rect, corner_radius, bg_color);

    // Draw text
    ui.painter().galley(rect.min + padding, galley, text_color);
}

/// Render a status badge with icon
pub fn badge_with_icon(ui: &mut egui::Ui, icon: &str, text: impl Into<String>, style: BadgeStyle) {
    let text = text.into();
    let (bg_color, text_color) = style.colors();

    let padding = egui::Vec2::new(Spacing::SM, Spacing::XS);
    let corner_radius = 6.0;

    let icon_text = format!("{} {}", icon, text);
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let galley = ui.fonts(|f| f.layout_no_wrap(icon_text.clone(), font_id, text_color));

    let size = galley.size() + padding * 2.0;
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

    // Draw badge background
    ui.painter().rect_filled(rect, corner_radius, bg_color);

    // Draw text
    ui.painter().galley(rect.min + padding, galley, text_color);
}
