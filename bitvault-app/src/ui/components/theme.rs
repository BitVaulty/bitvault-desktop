//! Theme and Color System
//!
//! Modern color palette and styling constants

use eframe::egui;

/// Modern color palette
pub struct Colors;

impl Colors {
    // Primary colors
    pub const PRIMARY: egui::Color32 = egui::Color32::from_rgb(59, 130, 246); // Blue-500
    pub const PRIMARY_DARK: egui::Color32 = egui::Color32::from_rgb(37, 99, 235); // Blue-600
    pub const PRIMARY_LIGHT: egui::Color32 = egui::Color32::from_rgb(147, 197, 253); // Blue-300

    // Success colors
    pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(34, 197, 94); // Green-500
    pub const SUCCESS_LIGHT: egui::Color32 = egui::Color32::from_rgb(134, 239, 172); // Green-300
    pub const SUCCESS_DARK: egui::Color32 = egui::Color32::from_rgb(22, 163, 74); // Green-600

    // Warning colors
    pub const WARNING: egui::Color32 = egui::Color32::from_rgb(234, 179, 8); // Yellow-500
    pub const WARNING_LIGHT: egui::Color32 = egui::Color32::from_rgb(253, 224, 71); // Yellow-300
    pub const WARNING_DARK: egui::Color32 = egui::Color32::from_rgb(202, 138, 4); // Yellow-600

    // Error colors
    pub const ERROR: egui::Color32 = egui::Color32::from_rgb(239, 68, 68); // Red-500
    pub const ERROR_LIGHT: egui::Color32 = egui::Color32::from_rgb(252, 165, 165); // Red-300
    pub const ERROR_DARK: egui::Color32 = egui::Color32::from_rgb(220, 38, 38); // Red-600

    // Neutral colors
    pub const GRAY_50: egui::Color32 = egui::Color32::from_rgb(249, 250, 251);
    pub const GRAY_100: egui::Color32 = egui::Color32::from_rgb(243, 244, 246);
    pub const GRAY_200: egui::Color32 = egui::Color32::from_rgb(229, 231, 235);
    pub const GRAY_300: egui::Color32 = egui::Color32::from_rgb(209, 213, 219);
    pub const GRAY_400: egui::Color32 = egui::Color32::from_rgb(156, 163, 175);
    pub const GRAY_500: egui::Color32 = egui::Color32::from_rgb(107, 114, 128);
    pub const GRAY_600: egui::Color32 = egui::Color32::from_rgb(75, 85, 99);
    pub const GRAY_700: egui::Color32 = egui::Color32::from_rgb(55, 65, 81);
    pub const GRAY_800: egui::Color32 = egui::Color32::from_rgb(31, 41, 55);
    pub const GRAY_900: egui::Color32 = egui::Color32::from_rgb(17, 24, 39);

    // Text colors (adapts to light/dark mode)
    pub fn text_primary(ctx: &egui::Context) -> egui::Color32 {
        if ctx.style().visuals.dark_mode {
            Self::GRAY_100
        } else {
            Self::GRAY_900
        }
    }

    pub fn text_secondary(ctx: &egui::Context) -> egui::Color32 {
        if ctx.style().visuals.dark_mode {
            Self::GRAY_400
        } else {
            Self::GRAY_600
        }
    }

    pub fn text_muted(_ctx: &egui::Context) -> egui::Color32 {
        Self::GRAY_500
    }

    // Background colors
    pub fn bg_card(ctx: &egui::Context) -> egui::Color32 {
        if ctx.style().visuals.dark_mode {
            Self::GRAY_800
        } else {
            egui::Color32::WHITE
        }
    }

    pub fn bg_secondary(ctx: &egui::Context) -> egui::Color32 {
        if ctx.style().visuals.dark_mode {
            Self::GRAY_900
        } else {
            Self::GRAY_50
        }
    }
}

/// Spacing constants
pub struct Spacing;

impl Spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
    pub const XXL: f32 = 48.0;
}

/// Typography helpers
pub struct Typography;

impl Typography {
    pub fn heading_large(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).size(32.0).strong()
    }

    pub fn heading(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).size(24.0).strong()
    }

    pub fn heading_small(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).size(20.0).strong()
    }

    pub fn body_large(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).size(16.0)
    }

    pub fn body(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).size(14.0)
    }

    pub fn body_small(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).size(12.0)
    }

    pub fn label(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).size(14.0)
    }

    pub fn caption(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text).size(12.0)
    }
}
