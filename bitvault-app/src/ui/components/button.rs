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
/// Gray buttons (Secondary/Text) are smaller than primary buttons but not tiny
pub fn button(ui: &mut egui::Ui, text: impl Into<String>, style: ButtonStyle) -> egui::Response {
    let text = text.into();
    let (bg_color, hover_color, text_color) = style.colors(ui.ctx());
    
    // Gray buttons (Secondary/Text) should be smaller than primary but not tiny
    // Primary buttons are 200x44, so gray buttons should be around 140x38
    let min_size = match style {
        ButtonStyle::Primary => egui::vec2(200.0, 44.0),
        ButtonStyle::Secondary | ButtonStyle::Text => egui::vec2(140.0, 38.0),
        ButtonStyle::Danger | ButtonStyle::Success => egui::vec2(140.0, 38.0),
    };
    
    let button = egui::Button::new(
        egui::RichText::new(&text)
            .color(text_color)
            .size(14.0)
    )
    .fill(bg_color)
    .min_size(min_size);
    
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
    
    // Pre-allocate space for the button
    let desired_size = egui::Vec2::new(200.0, 44.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    
    // Check focus and keyboard input before using response
    let is_focused = response.has_focus();
    let keyboard_activated = is_focused 
        && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
    
    // Custom styling with focus indicator
    if ui.is_rect_visible(rect) {
        let bg_color = if is_focused {
            // Focused: slightly brighter
            Colors::PRIMARY_DARK
        } else if response.hovered() || keyboard_activated {
            Colors::PRIMARY_DARK
        } else {
            Colors::PRIMARY
        };
        
        let painter = ui.painter();
        
        // Draw focus outline if focused (blue outline)
        if is_focused {
            // Draw a 2px outline around the button
            let outline_rect = rect.expand(2.0);
            painter.rect_stroke(
                outline_rect, 
                8.0, 
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237))
            );
        }
        
        // Draw custom background
        painter.rect_filled(rect, 8.0, bg_color);
        
        // Draw text
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            egui::FontId::proportional(16.0),
            egui::Color32::WHITE,
        );
    }
    
    // Return response, and check for keyboard activation in the calling code
    // The caller should check: response.clicked() || (response.has_focus() && keyboard_activated)
    response
}
