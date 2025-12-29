//! Help and Support Question List
//!
//! Displays a searchable list of frequently asked questions

use eframe::egui;
use crate::state::{AppState, Navigation};
use crate::ui::help::QuestionAndAnswer;
use super::question_detail::{render_question_detail, QuestionDetailState};

/// Help and Support state
pub struct HelpAndSupportState {
    questions: Vec<QuestionAndAnswer>,
    filtered_questions: Vec<QuestionAndAnswer>,
    search_text: String,
    selected_question: Option<QuestionAndAnswer>,
}

impl Default for HelpAndSupportState {
    fn default() -> Self {
        let questions = QuestionAndAnswer::all_questions();
        Self {
            filtered_questions: questions.clone(),
            questions,
            search_text: String::new(),
            selected_question: None,
        }
    }
}

impl HelpAndSupportState {
    pub fn new() -> Self {
        Self::default()
    }

    fn apply_search(&mut self) {
        if self.search_text.is_empty() {
            self.filtered_questions = self.questions.clone();
        } else {
            let search_lower = self.search_text.to_lowercase();
            self.filtered_questions = self.questions
                .iter()
                .filter(|q| q.question.to_lowercase().contains(&search_lower) ||
                           q.answer.to_lowercase().contains(&search_lower))
                .cloned()
                .collect();
        }
    }
}

/// Render help and support screen
pub fn render_help_and_support(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut HelpAndSupportState,
) {
    // If a question is selected, show detail view
    if let Some(ref question) = state.selected_question {
        let mut detail_state = QuestionDetailState {
            question: question.clone(),
        };
        render_question_detail(ui, _app_state, navigation, &mut detail_state);
        return;
    }

    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading("Help & Support");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕ Close").clicked() {
                    navigation.go_back();
                }
            });
        });

        ui.add_space(20.0);

        // Greeting
        ui.label(egui::RichText::new("Hello there,").size(18.0));
        ui.label(egui::RichText::new("How can we help?").size(20.0).strong());

        ui.add_space(20.0);

        // Search bar
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut state.search_text);
        });

        // Apply search filter
        state.apply_search();

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Question list
        if state.filtered_questions.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("No questions found");
                ui.label("Try a different search term");
            });
        } else {
            egui::ScrollArea::vertical()
                .max_height(500.0)
                .show(ui, |ui| {
                    for question in &state.filtered_questions {
                        ui.horizontal(|ui| {
                            // Question text
                            ui.label(egui::RichText::new(&question.question).size(14.0));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("→").clicked() {
                                    state.selected_question = Some(question.clone());
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                    }
                });
        }
    });
}
