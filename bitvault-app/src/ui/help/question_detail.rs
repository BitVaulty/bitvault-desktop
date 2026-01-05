//! Question Detail View
//!
//! Displays the full answer to a selected question

use crate::state::{AppState, Navigation};
use crate::ui::help::QuestionAndAnswer;
use eframe::egui;

/// Question detail state
pub struct QuestionDetailState {
    pub question: QuestionAndAnswer,
}

/// Render question detail view
pub fn render_question_detail(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut QuestionDetailState,
) {
    ui.vertical(|ui| {
        // Header with back button
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                // Go back to question list
                // This will be handled by the parent component
                navigation.go_back();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕ Close").clicked() {
                    navigation.go_back();
                }
            });
        });

        ui.add_space(20.0);

        // Question
        ui.heading(&state.question.question);

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        // Answer
        ui.label(egui::RichText::new("Answer:").size(16.0).strong());
        ui.add_space(10.0);

        // Display answer with proper formatting (handle newlines)
        let answer_lines: Vec<&str> = state.question.answer.split('\n').collect();
        for line in answer_lines {
            if line.trim().is_empty() {
                ui.add_space(5.0);
            } else {
                ui.label(line);
            }
        }

        ui.add_space(30.0);
        ui.separator();
        ui.add_space(20.0);

        // Back button at bottom
        if ui.button("← Back to Questions").clicked() {
            navigation.go_back();
        }
    });
}
