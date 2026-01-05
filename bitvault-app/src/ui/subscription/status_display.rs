//! Subscription Status Display
//!
//! Shows current subscription status and information

use crate::state::{AppState, Navigation};
use eframe::egui;

/// Get the subscription renewal URL
/// For now, uses the hardcoded annual payment link
/// TODO: Fetch from remote config in the future
fn get_subscription_renewal_url() -> String {
    // Default annual payment link from remote config
    // This matches the mobile app's SubscriptionLinks.annual
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
            ui.label("Loading subscription status...");
            return;
        }

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            ui.add_space(10.0);
            if ui.button("Retry").clicked() {
                state.error = None;
                state.subscription_data = None;
            }
            return;
        }

        // Show subscription status
        if let Some(ref subscription) = state.subscription_data {
            render_subscription_info(ui, subscription, &ctx);
        } else {
            ui.label("No subscription data available");
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Refresh button
        if ui.button("Refresh Status").clicked() {
            state.subscription_data = None;
            state.last_refresh = None;
        }
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
                state.error = Some(format!("Failed to load subscription: {}", e));
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
    ui.separator();
    ui.add_space(10.0);

    // Status
    let status_color = if subscription.is_valid() {
        egui::Color32::GREEN
    } else {
        egui::Color32::RED
    };
    ui.horizontal(|ui| {
        ui.label("Status:");
        ui.colored_label(status_color, subscription.status_string());
    });
    ui.add_space(10.0);

    // Lifetime indicator
    if subscription.lifetime {
        ui.colored_label(egui::Color32::GREEN, "✓ Lifetime Subscription");
        ui.add_space(10.0);
    }

    // Active indicator
    if subscription.is_active {
        ui.label("✓ Subscription is active");
        ui.add_space(10.0);
    }

    // Grace period indicator
    if subscription.is_in_grace_period() {
        ui.colored_label(
            egui::Color32::YELLOW,
            "⚠ Subscription in grace period (7 days)",
        );
        ui.add_space(10.0);
    }

    // Days remaining
    if let Some(days) = subscription.days_remaining {
        if days > 0 {
            ui.label(format!("Days remaining: {}", days));
        } else if days == 0 {
            ui.colored_label(egui::Color32::YELLOW, "Subscription expires today");
        }
        ui.add_space(10.0);
    }

    // Paid until date
    if let Some(paid_until) = subscription.paid_until {
        if let Some(dt) = chrono::DateTime::from_timestamp(paid_until as i64, 0) {
            ui.label(format!(
                "Paid until: {}",
                dt.format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }
    }

    ui.add_space(10.0);

    // Show renewal button for expired subscriptions or those in grace period
    if !subscription.is_valid() {
        ui.colored_label(
            egui::Color32::RED,
            "⚠ Subscription expired. Please renew to continue using the service.",
        );
        ui.add_space(10.0);
    } else if subscription.is_in_grace_period() {
        ui.colored_label(
            egui::Color32::YELLOW,
            "⚠ Subscription in grace period. Consider renewing to avoid service interruption.",
        );
        ui.add_space(10.0);
    } else if let Some(days) = subscription.days_remaining {
        if days <= 7 && days > 0 {
            ui.colored_label(
                egui::Color32::YELLOW,
                format!(
                    "⚠ Subscription expires in {} days. Consider renewing soon.",
                    days
                ),
            );
            ui.add_space(10.0);
        }
    }

    // Show renewal button if subscription is expired, in grace period, or expiring soon (within 7 days)
    let should_show_renewal = !subscription.is_valid()
        || subscription.is_in_grace_period()
        || subscription.days_remaining.is_some_and(|days| days <= 7);

    if should_show_renewal && ui.button("Renew Subscription").clicked() {
        let url = get_subscription_renewal_url();
        ctx.output_mut(|o| {
            o.open_url = Some(egui::OpenUrl {
                url: url.clone(),
                new_tab: true,
            });
        });
    }
}
