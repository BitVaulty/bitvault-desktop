//! Modern Tab Component
//!
//! Tab bar with underline indicators and modern styling

use super::theme::{Colors, Spacing};
use eframe::egui;

/// Render a modern tab bar with underline indicators
pub fn tab_bar(ui: &mut egui::Ui, tabs: &[(&str, bool)], mut on_tab_clicked: impl FnMut(usize)) {
    let is_dark = ui.ctx().style().visuals.dark_mode;
    let underline_height = 3.0;
    let padding = Spacing::MD;

    let mut responses = Vec::new();

    ui.horizontal(|ui| {
        for (idx, (label, is_active)) in tabs.iter().enumerate() {
            let response = ui.selectable_label(*is_active, *label);
            responses.push((idx, *is_active, response));
            ui.add_space(padding);
        }
    });

    // Draw effects after all labels are rendered
    for (idx, is_active, response) in responses.iter() {
        if response.clicked() && !is_active {
            on_tab_clicked(*idx);
        }

        // Draw underline for active tab
        if *is_active {
            let underline_rect = egui::Rect::from_min_max(
                egui::Pos2::new(response.rect.min.x, response.rect.max.y - underline_height),
                egui::Pos2::new(response.rect.max.x, response.rect.max.y),
            );
            ui.painter()
                .rect_filled(underline_rect, 0.0, Colors::PRIMARY);
        }

        // Hover effect for inactive tabs
        if response.hovered() && !is_active {
            let hover_rect = response.rect;
            ui.painter().rect_filled(
                hover_rect,
                0.0,
                if is_dark {
                    Colors::GRAY_800
                } else {
                    Colors::GRAY_100
                },
            );
        }
    }

    ui.add_space(Spacing::SM);
    ui.separator();
}
