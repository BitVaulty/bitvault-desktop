//! Dashboard UI
//!
//! Main dashboard with 3 tabs:
//! - Vault Detail (balance, address, recent transactions)
//! - Transaction History (all transactions)
//! - Settings (vault settings, network, etc.)

mod settings_tab;
mod transaction_history;
mod vault_detail;

use crate::state::{AppState, Navigation};
use eframe::egui;

/// Render the dashboard
pub fn render_dashboard(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    current_tab: usize,
) {
    // Tab bar
    ui.horizontal(|ui| {
        if ui.selectable_label(current_tab == 0, "Vault").clicked() {
            navigation.set_dashboard_tab(0);
        }
        if ui.selectable_label(current_tab == 1, "History").clicked() {
            navigation.set_dashboard_tab(1);
        }
        if ui.selectable_label(current_tab == 2, "Settings").clicked() {
            navigation.set_dashboard_tab(2);
        }
    });

    ui.separator();

    // Tab content
    match current_tab {
        0 => vault_detail::render(ui, app_state, navigation),
        1 => transaction_history::render(ui, app_state, navigation),
        2 => settings_tab::render(ui, app_state, navigation),
        _ => {}
    }
}
