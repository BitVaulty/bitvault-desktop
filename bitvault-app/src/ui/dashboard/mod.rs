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
use crate::ui::components::tab_bar;
use eframe::egui;

/// Render the dashboard
pub fn render_dashboard(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    current_tab: usize,
) {
    // Modern tab bar with underline indicators
    let tabs = [
        ("Vault", current_tab == 0),
        ("History", current_tab == 1),
        ("Settings", current_tab == 2),
    ];
    
    // Collect tab clicks first, then apply navigation
    let mut clicked_tab: Option<usize> = None;
    tab_bar(ui, &tabs, |idx| {
        clicked_tab = Some(idx);
    });
    
    if let Some(idx) = clicked_tab {
        navigation.set_dashboard_tab(idx);
    }

    ui.add_space(8.0);

    // Tab content
    match current_tab {
        0 => vault_detail::render(ui, app_state, navigation),
        1 => transaction_history::render(ui, app_state, navigation),
        2 => settings_tab::render(ui, app_state, navigation),
        _ => {}
    }
}
