//! Subscription Status Display
//!
//! Shows current subscription status and information

use crate::state::{AppState, Navigation};
use crate::ui::components::{
    badge, button, button_large, card, BadgeStyle, ButtonStyle, Colors, Spacing, Typography,
};
use eframe::egui;

/// Get the subscription renewal URL for annual payments
///
/// Returns the payment link for subscription renewal. Currently uses a static URL
/// that matches the mobile app's `SubscriptionLinks.annual` configuration.
///
/// # Future Enhancement
/// This URL could be fetched from a remote configuration endpoint to allow
/// updates without app releases. For now, the static URL is sufficient as
/// payment links are stable.
///
/// # Returns
/// The Zaprite payment URL for annual subscription renewal
fn get_subscription_renewal_url() -> String {
    // Annual payment link - matches mobile app's SubscriptionLinks.annual
    "https://pay.zaprite.com/pl_81YcHRehEj".to_string()
}

/// State for subscription status display
#[derive(Default)]
struct SubscriptionStatusState {
    subscription_data: Option<bitvault_common::types::SubscriptionData>,
    is_loading: bool,
    error: Option<String>,
    last_refresh: Option<std::time::Instant>,
}

// Thread-local state for subscription status
thread_local! {
    static SUBSCRIPTION_STATE: std::cell::RefCell<SubscriptionStatusState> =
        std::cell::RefCell::new(SubscriptionStatusState::default());
}

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, _navigation: &mut Navigation) {
    let ctx = ui.ctx().clone();
    let error_clone = SUBSCRIPTION_STATE.with(|state| state.borrow().error.clone());
    SUBSCRIPTION_STATE.with(|state| {
        let mut state = state.borrow_mut();

        // Auto-refresh if needed (every 60 seconds)
        if let Some(last_refresh) = state.last_refresh {
            if last_refresh.elapsed().as_secs() > 60 {
                state.subscription_data = None;
                state.last_refresh = None;
            }
        }

        // Load subscription data if not loaded
        if state.subscription_data.is_none() && !state.is_loading {
            load_subscription_data(ui, app_state, &mut state);
        }

        // Show loading state
        if state.is_loading {
            ui.vertical_centered(|ui| {
                ui.add_space(Spacing::XXL);
                ui.spinner();
                ui.add_space(Spacing::MD);
                ui.label(
                    Typography::body("Loading subscription status...")
                        .color(Colors::text_secondary(&ctx)),
                );
            });
            return;
        }

        // Show error if any
        if let Some(ref error) = error_clone {
            card(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(Spacing::LG);
                    ui.label(Typography::heading_small("Error").color(Colors::ERROR));
                    ui.add_space(Spacing::SM);
                    ui.label(Typography::body(error).color(Colors::text_secondary(&ctx)));
                    ui.add_space(Spacing::LG);
                    if button(ui, "Retry", ButtonStyle::Primary).clicked() {
                        state.error = None;
                        state.subscription_data = None;
                    }
                    ui.add_space(Spacing::LG);
                });
            });
            return;
        }

        // Show subscription status
        if let Some(ref subscription) = state.subscription_data {
            render_subscription_info(ui, subscription, &ctx);
        } else {
            card(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(Spacing::LG);
                    ui.label(
                        Typography::body("No subscription data available")
                            .color(Colors::text_secondary(&ctx)),
                    );
                    ui.add_space(Spacing::LG);
                });
            });
        }

        ui.add_space(Spacing::MD);

        // Refresh button - centered
        ui.vertical_centered(|ui| {
            if button(ui, "Refresh Status", ButtonStyle::Secondary).clicked() {
                state.subscription_data = None;
                state.last_refresh = None;
            }
        });
    });
}

fn load_subscription_data(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut SubscriptionStatusState,
) {
    state.is_loading = true;
    state.error = None;

    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let result = runtime.block_on(async {
            let vs = vault_service.read().await;
            vs.get_subscription_info().await
        });

        match result {
            Ok(subscription_data) => {
                state.subscription_data = Some(subscription_data);
                state.last_refresh = Some(std::time::Instant::now());
                state.is_loading = false;
            }
            Err(e) => {
                state.error = Some(format!("Failed to load subscription: {}", crate::utils::sanitize_error_for_ui(&e)));
                state.is_loading = false;
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.is_loading = false;
    }
}

fn render_subscription_info(
    ui: &mut egui::Ui,
    subscription: &bitvault_common::types::SubscriptionData,
    ctx: &egui::Context,
) {
    // Main status card
    card(ui, |ui| {
        ui.vertical(|ui| {
            ui.add_space(Spacing::LG);

            // Header with status badge
            ui.horizontal(|ui| {
                ui.label(
                    Typography::heading("Subscription Status").color(Colors::text_primary(ctx)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let badge_style = if subscription.is_valid() {
                        if subscription.lifetime || subscription.is_active {
                            BadgeStyle::Success
                        } else {
                            BadgeStyle::Warning
                        }
                    } else {
                        BadgeStyle::Error
                    };
                    badge(ui, subscription.status_string(), badge_style);
                });
            });

            ui.add_space(Spacing::MD);
            ui.separator();
            ui.add_space(Spacing::MD);

            // Subscription details
            ui.vertical(|ui| {
                // Lifetime indicator
                if subscription.lifetime {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("✓").color(Colors::SUCCESS).size(18.0));
                        ui.add_space(Spacing::SM);
                        ui.label(
                            Typography::body("Lifetime Subscription")
                                .color(Colors::text_primary(ctx)),
                        );
                    });
                    ui.add_space(Spacing::SM);
                }

                // Active indicator
                if subscription.is_active && !subscription.lifetime {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("✓").color(Colors::SUCCESS).size(18.0));
                        ui.add_space(Spacing::SM);
                        ui.label(
                            Typography::body("Subscription is active")
                                .color(Colors::text_primary(ctx)),
                        );
                    });
                    ui.add_space(Spacing::SM);
                }

                // Grace period indicator
                if subscription.is_in_grace_period() {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("⚠").color(Colors::WARNING).size(18.0));
                        ui.add_space(Spacing::SM);
                        ui.label(
                            Typography::body("Subscription in grace period (7 days)")
                                .color(Colors::WARNING),
                        );
                    });
                    ui.add_space(Spacing::SM);
                }

                // Days remaining
                if let Some(days) = subscription.days_remaining {
                    if days > 0 {
                        ui.horizontal(|ui| {
                            ui.label(
                                Typography::body("Days remaining:")
                                    .color(Colors::text_secondary(ctx)),
                            );
                            ui.add_space(Spacing::SM);
                            ui.label(
                                Typography::body(format!("{}", days))
                                    .color(Colors::text_primary(ctx))
                                    .strong(),
                            );
                        });
                        ui.add_space(Spacing::SM);
                    } else if days == 0 {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("⚠").color(Colors::WARNING).size(18.0));
                            ui.add_space(Spacing::SM);
                            ui.label(
                                Typography::body("Subscription expires today")
                                    .color(Colors::WARNING),
                            );
                        });
                        ui.add_space(Spacing::SM);
                    }
                }

                // Paid until date
                if let Some(paid_until) = subscription.paid_until {
                    if let Some(dt) = chrono::DateTime::from_timestamp(paid_until as i64, 0) {
                        ui.add_space(Spacing::SM);
                        ui.separator();
                        ui.add_space(Spacing::SM);
                        ui.horizontal(|ui| {
                            ui.label(
                                Typography::body("Paid until:").color(Colors::text_secondary(ctx)),
                            );
                            ui.add_space(Spacing::SM);
                            ui.label(
                                Typography::body(dt.format("%B %d, %Y").to_string())
                                    .color(Colors::text_primary(ctx)),
                            );
                        });
                    }
                }
            });

            ui.add_space(Spacing::LG);
        });
    });

    ui.add_space(Spacing::MD);

    // Warning/action card
    let should_show_renewal = !subscription.is_valid()
        || subscription.is_in_grace_period()
        || subscription.days_remaining.is_some_and(|days| days <= 7);

    if should_show_renewal {
        let (warning_color, warning_text) = if !subscription.is_valid() {
            (
                Colors::ERROR,
                "Subscription expired. Please renew to continue using the service.".to_string(),
            )
        } else if subscription.is_in_grace_period() {
            (
                Colors::WARNING,
                "Subscription in grace period. Consider renewing to avoid service interruption."
                    .to_string(),
            )
        } else if let Some(days) = subscription.days_remaining {
            if days <= 7 && days > 0 {
                (
                    Colors::WARNING,
                    format!(
                        "Subscription expires in {} days. Consider renewing soon.",
                        days
                    ),
                )
            } else {
                return; // Don't show warning card
            }
        } else {
            return; // Don't show warning card
        };

        card(ui, |ui| {
            ui.vertical(|ui| {
                ui.add_space(Spacing::MD);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠").color(warning_color).size(20.0));
                    ui.add_space(Spacing::SM);
                    ui.label(Typography::body(warning_text).color(warning_color));
                });
                ui.add_space(Spacing::MD);
                ui.vertical_centered(|ui| {
                    if button_large(ui, "Renew Subscription").clicked() {
                        let url = get_subscription_renewal_url();
                        ui.output_mut(|o| {
                            o.open_url = Some(egui::OpenUrl {
                                url: url.clone(),
                                new_tab: true,
                            });
                        });
                    }
                });
                ui.add_space(Spacing::MD);
            });
        });
    }
}
