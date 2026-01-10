//! Modern Button Components
//!
//! Styled button variants with consistent design

use eframe::egui;
use super::theme::Colors;

/// Standard margin for text inputs
pub const TEXT_INPUT_MARGIN: egui::Vec2 = egui::Vec2::new(8.0, 6.0);

/// Button style variants
#[derive(Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
    Success,
    Text,
}

impl ButtonStyle {
    fn colors(&self, ctx: &egui::Context) -> (egui::Color32, egui::Color32, egui::Color32) {
        let is_dark = ctx.style().visuals.dark_mode;
        match self {
            ButtonStyle::Primary => (Colors::PRIMARY, Colors::PRIMARY_DARK, egui::Color32::WHITE),
            ButtonStyle::Secondary => {
                if is_dark {
                    (Colors::GRAY_700, Colors::GRAY_600, Colors::text_primary(ctx))
                } else {
                    (Colors::GRAY_200, Colors::GRAY_300, Colors::GRAY_900)
                }
            }
            ButtonStyle::Danger => (Colors::ERROR, Colors::ERROR_DARK, egui::Color32::WHITE),
            ButtonStyle::Success => (Colors::SUCCESS, Colors::SUCCESS_DARK, egui::Color32::WHITE),
            ButtonStyle::Text => {
                if is_dark {
                    // Dark mode: transparent bg, darker hover (GRAY_800), light text (GRAY_100)
                    // GRAY_800 (31,41,55) vs GRAY_100 (243,244,246) = good contrast
                    (egui::Color32::TRANSPARENT, Colors::GRAY_800, Colors::text_primary(ctx))
                } else {
                    // Light mode: transparent bg, light hover (GRAY_200), dark text (GRAY_900)
                    // GRAY_200 (229,231,235) vs GRAY_900 (17,24,39) = good contrast
                    (egui::Color32::TRANSPARENT, Colors::GRAY_200, Colors::GRAY_900)
                }
            }
        }
    }
}

/// Render a styled button with proper padding
pub fn button(ui: &mut egui::Ui, text: impl Into<String>, style: ButtonStyle) -> egui::Response {
    let text = text.into();
    let (bg_color, hover_color, text_color) = style.colors(ui.ctx());
    
    let button = egui::Button::new(
        egui::RichText::new(&text)
            .color(text_color)
            .size(14.0)
    )
    .fill(bg_color)
    .min_size(egui::vec2(80.0, 32.0)); // Minimum size with padding
    
    let response = ui.add(button);
    
    // For Text style buttons, show hover background
    if matches!(style, ButtonStyle::Text) && response.hovered() {
        let painter = ui.painter();
        painter.rect_filled(response.rect, 4.0, hover_color);
        painter.text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            egui::FontId::proportional(14.0),
            text_color,
        );
    }
    
    response
}

/// Render a large primary button
pub fn button_large(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let text = text.into();
    
    // Pre-allocate the rect to check hover state
    let desired_size = egui::Vec2::new(200.0, 44.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    
    if ui.is_rect_visible(rect) {
        let bg_color = if response.hovered() {
            Colors::PRIMARY_DARK
        } else {
            Colors::PRIMARY
        };
        
        let painter = ui.painter();
        painter.rect_filled(rect, 8.0, bg_color);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            egui::FontId::proportional(16.0),
            egui::Color32::WHITE,
        );
    }
    
    response
}
