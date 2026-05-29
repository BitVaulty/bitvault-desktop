//! Secret Notification Setup View

use crate::state::{AppState, Navigation};
use crate::ui::components::{button, ButtonStyle, Spacing};
use crate::ui::secret_notification::state::{SecretNotificationPhase, SecretNotificationState};
use eframe::egui;

/// Render secret notification setup view
pub fn render(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut SecretNotificationState,
) {
    if state.phase == SecretNotificationPhase::WaitingForConfirmation && state.should_poll() {
        if let Some(ref mut handler) = app_state.async_handler {
            handler.check_telegram_registration();
            state.mark_poll_sent();
        }
    }

    if let Some(registered) = app_state.telegram_registration_status.take() {
        if registered {
            state.mark_registered();
        } else if state.poll_attempts >= state.max_poll_attempts {
            state.mark_connection_failed("Registration timeout. Please try again.".to_string());
        }
    }

    if let Some(error) = app_state.telegram_async_error.take() {
        if state.phase == SecretNotificationPhase::WaitingForConfirmation
            && state.poll_attempts >= state.max_poll_attempts
        {
            state.mark_connection_failed(error);
        } else if state.phase == SecretNotificationPhase::Intro
            || state.phase == SecretNotificationPhase::LinkReady
        {
            state.error = Some(error);
            state.is_loading = false;
        }
    }

    if let Some(ref link) = app_state.telegram_registration_link {
        if state.registration_link.as_deref() != Some(link.as_str()) {
            state.registration_link = Some(link.clone());
            if state.phase == SecretNotificationPhase::Intro {
                state.phase = SecretNotificationPhase::LinkReady;
                state.is_loading = false;
            }
        }
    }

    ui.vertical_centered(|ui| {
        match state.phase {
            SecretNotificationPhase::Intro | SecretNotificationPhase::LinkReady => {
                render_intro(ui, app_state, state);
            }
            SecretNotificationPhase::WaitingForConfirmation => render_waiting(ui, state),
            SecretNotificationPhase::ConnectionFailed => render_failed(ui, app_state, state),
            SecretNotificationPhase::Success => render_success(ui, navigation),
        }

        ui.add_space(Spacing::LG);

        if state.phase != SecretNotificationPhase::Success
            && button(ui, "Back", ButtonStyle::Secondary).clicked()
        {
            navigation.go_back();
        }
    });
}

fn render_intro(ui: &mut egui::Ui, app_state: &mut AppState, state: &mut SecretNotificationState) {
    ui.heading("Turn ON Secret Notifications");
    ui.add_space(Spacing::MD);

    ui.label(
        "To keep up with your vault activity when transactions are initiated or received, enable notifications.",
    );
    ui.add_space(Spacing::SM);
    ui.label("This will open Telegram and start a secure chat with our notification bot.");
    ui.add_space(Spacing::LG);

    if state.is_loading {
        ui.spinner();
        ui.label("Requesting registration link...");
    } else if let Some(ref error) = state.error {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        if ui.button("Retry").clicked() {
            state.error = None;
            request_registration_link(app_state, state);
        }
    } else if let Some(link) = state.registration_link.clone() {
        ui.label("Registration link received!");
        ui.add_space(Spacing::SM);

        if button(ui, "Open Telegram Bot", ButtonStyle::Primary).clicked() {
            let open_link = link.clone();
            ui.output_mut(|o| {
                o.open_url = Some(egui::OpenUrl {
                    url: open_link,
                    new_tab: true,
                });
            });
            state.begin_waiting();
        }

        ui.add_space(Spacing::SM);
        ui.horizontal(|ui| {
            ui.label("Link:");
            let _ = ui.selectable_label(false, &link);
            if ui.button("Copy").clicked() {
                ui.output_mut(|o| {
                    o.copied_text = link.clone();
                });
            }
        });
    } else if button(ui, "Open Telegram Bot", ButtonStyle::Primary).clicked() {
        request_registration_link(app_state, state);
    }
}

fn render_waiting(ui: &mut egui::Ui, state: &SecretNotificationState) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(
            egui::RichText::new("Waiting for confirmation")
                .size(18.0)
                .strong(),
        );
    });
    ui.add_space(Spacing::SM);
    ui.label("Complete the Telegram bot setup, then return here.");
    if state.poll_attempts > 0 {
        ui.label(format!(
            "Checking registration status ({}/{})…",
            state.poll_attempts, state.max_poll_attempts
        ));
    }
}

fn render_failed(ui: &mut egui::Ui, app_state: &mut AppState, state: &mut SecretNotificationState) {
    ui.label(egui::RichText::new("⚠").size(28.0));
    ui.add_space(Spacing::SM);
    ui.heading("Failed to connect");
    ui.add_space(Spacing::SM);

    if let Some(ref error) = state.error {
        ui.colored_label(egui::Color32::RED, error);
    } else {
        ui.label("Registration timeout. Please try again.");
    }

    ui.add_space(Spacing::MD);

    let cooldown = state.retry_cooldown_remaining_secs();
    let label = if cooldown > 0 {
        format!("Retry ({})", cooldown)
    } else {
        "Retry".to_string()
    };

    ui.add_enabled_ui(cooldown == 0, |ui| {
        if button(ui, &label, ButtonStyle::Primary).clicked() {
            if let Some(ref link) = state.registration_link.clone() {
                ui.output_mut(|o| {
                    o.open_url = Some(egui::OpenUrl {
                        url: link.clone(),
                        new_tab: true,
                    });
                });
            } else {
                request_registration_link(app_state, state);
            }
            state.begin_retry();
        }
    });
}

fn render_success(ui: &mut egui::Ui, navigation: &mut Navigation) {
    ui.colored_label(egui::Color32::GREEN, "✓ Secret notifications enabled");
    ui.add_space(Spacing::MD);
    ui.label("You will receive Telegram alerts for vault activity.");
    ui.add_space(Spacing::LG);

    if button(ui, "Done", ButtonStyle::Primary).clicked() {
        navigation.go_back();
    }
}

fn request_registration_link(app_state: &mut AppState, state: &mut SecretNotificationState) {
    state.is_loading = true;
    state.error = None;
    if let Some(ref mut handler) = app_state.async_handler {
        handler.request_telegram_registration();
    }
}
