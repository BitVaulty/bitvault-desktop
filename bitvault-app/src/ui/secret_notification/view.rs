//! Secret Notification Setup View

use crate::state::{AppState, Navigation};
use crate::ui::secret_notification::state::SecretNotificationState;
use eframe::egui;

/// Render secret notification setup view
pub fn render(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut SecretNotificationState,
) {
    ui.vertical_centered(|ui| {
        ui.heading("Turn ON Secret Notifications");
        ui.add_space(20.0);

        ui.label("To keep up with your vault activity when transactions are initiated or received, enable notifications.");
        ui.add_space(10.0);

        ui.label("This will open Telegram and start a secure chat with our notification bot");
        ui.add_space(20.0);

        if state.is_loading {
            ui.spinner();
            ui.label("Requesting registration link...");
        } else if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            if ui.button("Retry").clicked() {
                state.error = None;
                state.is_loading = true;
                // Trigger retry - will be handled by async command
                if let Some(ref mut handler) = app_state.async_handler {
                    handler.request_telegram_registration();
                }
            }
        } else if let Some(ref link) = app_state.telegram_registration_link {
            // Link received - show it and allow opening
            ui.label("Registration link received!");
            ui.add_space(10.0);

            if ui.button("Open Telegram Bot").clicked() {
                // Open link in browser (similar to subscription renewal)
                ui.output_mut(|o| {
                    o.open_url = Some(egui::OpenUrl {
                        url: link.clone(),
                        new_tab: true,
                    });
                });
            }

            ui.add_space(10.0);

            // Show link as copyable text
            ui.horizontal(|ui| {
                ui.label("Link:");
                let _ = ui.selectable_label(false, link);
                if ui.button("Copy").clicked() {
                    ui.output_mut(|o| {
                        o.copied_text = link.clone();
                    });
                }
            });

            // Clear loading state and store link in state for persistence
            state.is_loading = false;
            state.registration_link = Some(link.clone());
        } else if ui.button("Open Telegram Bot").clicked() {
            state.is_loading = true;
            state.error = None;
            // Request Telegram registration link via async command
            if let Some(ref mut handler) = app_state.async_handler {
                handler.request_telegram_registration();
            }
        }

        ui.add_space(20.0);

        if ui.button("Back").clicked() {
            navigation.go_back();
        }
    });
}
