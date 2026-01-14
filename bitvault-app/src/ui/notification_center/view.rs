//! Notification Center View

use crate::state::{AppState, Navigation};
use crate::ui::notification_center::state::{NotificationCenterState, Notification, NotificationType};
use eframe::egui;

/// Render notification center view
pub fn render(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut NotificationCenterState,
) {
    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading("Notifications");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    navigation.go_back();
                }
            });
        });
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Refresh button
        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh").clicked() {
                if let Some(ref vault_service) = app_state.vault_service {
                    if let Some(runtime) = app_state.get_runtime() {
                        runtime.block_on(state.fetch_notifications(vault_service));
                    }
                }
            }
            if let Some(ref last_fetch) = state.last_fetch {
                ui.label(egui::RichText::new(
                    format!("Last updated: {}", last_fetch.format("%H:%M:%S"))
                ).small().weak());
            }
        });
        ui.add_space(10.0);

        // Content
        if state.is_loading {
            ui.centered_and_justified(|ui| {
                ui.spinner();
                ui.label("Loading notifications...");
            });
        } else if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            if ui.button("Retry").clicked() {
                state.error = None;
                if let Some(ref vault_service) = app_state.vault_service {
                    if let Some(runtime) = app_state.get_runtime() {
                        runtime.block_on(state.fetch_notifications(vault_service));
                    }
                }
            }
        } else if state.notifications.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No notifications yet");
                ui.label("We'll update you when there's something new.");
            });
        } else {
            // Notification list
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 50.0)
                .show(ui, |ui| {
                    for notification in &state.notifications {
                        render_notification_row(ui, notification, navigation);
                    }
                });
        }
    });
}

fn render_notification_row(
    ui: &mut egui::Ui,
    notification: &Notification,
    navigation: &mut Navigation,
) {
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // Icon based on type
                let icon = match notification.notification_type {
                    NotificationType::Transaction => "💰",
                    NotificationType::Alert => "⚠️",
                    NotificationType::Info => "ℹ️",
                };
                ui.label(egui::RichText::new(icon).size(20.0));
                
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&notification.title).strong());
                    ui.label(egui::RichText::new(&notification.body).small());
                    ui.label(egui::RichText::new(
                        format!("{}", notification.created_at.format("%Y-%m-%d %H:%M"))
                    ).small().weak());
                });

                // Click to view transaction details if it's a transaction
                if notification.notification_type == NotificationType::Transaction {
                    if ui.button("View").clicked() {
                        navigation.navigate_to(crate::state::View::TransactionDetail {
                            txid: notification.id.clone(),
                        });
                    }
                }
            });
        });
    });
    ui.add_space(5.0);
}
