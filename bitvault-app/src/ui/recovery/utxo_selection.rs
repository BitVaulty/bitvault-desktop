//! UTXO Selection UI Component
//!
//! Displays list of old UTXOs for selection in recovery/refresh flows

use eframe::egui;
use bitvault_common::types::OldUtxo;

/// State for UTXO selection
#[derive(Default)]
pub struct UtxoSelectionState {
    pub utxos: Vec<OldUtxo>,
    pub selected: std::collections::HashSet<String>, // OutPoint as string
    pub is_loading: bool,
    pub error: Option<String>,
}


impl UtxoSelectionState {
    pub fn total_selected_amount(&self) -> f64 {
        self.utxos
            .iter()
            .filter(|utxo| self.selected.contains(&utxo.outpoint))
            .map(|utxo| utxo.amount)
            .sum()
    }

    pub fn has_selection(&self) -> bool {
        !self.selected.is_empty()
    }
}

/// Recovery mode enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryMode {
    Recovery,  // For UTXOs older than 1 year
    Refresh,    // For UTXOs older than 6 months
}

/// Render UTXO selection UI
pub fn render_utxo_selection(
    ui: &mut egui::Ui,
    state: &mut UtxoSelectionState,
    mode: RecoveryMode,
) {
    ui.vertical(|ui| {
        let title = match mode {
            RecoveryMode::Recovery => "Select UTXOs for Recovery",
            RecoveryMode::Refresh => "Select UTXOs for Refresh",
        };
        ui.heading(title);

        let description = match mode {
            RecoveryMode::Recovery => {
                "Select UTXOs older than 1 year to recover. These will be moved to a new address."
            }
            RecoveryMode::Refresh => {
                "Select UTXOs older than 6 months to refresh. These will be moved to a new address."
            }
        };
        ui.label(description);
        ui.add_space(10.0);

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }

        // Show loading state
        if state.is_loading {
            ui.label("Loading UTXOs...");
            return;
        }

        // Show UTXO list
        if state.utxos.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("No old UTXOs found");
                ui.label("All UTXOs are recent or locked");
            });
            return;
        }

        // Total selected amount
        let total = state.total_selected_amount();
        ui.horizontal(|ui| {
            ui.label("Total Selected:");
            ui.label(format!("{:.8} BTC", total));
        });
        ui.add_space(10.0);

        // UTXO list with checkboxes
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for utxo in &state.utxos {
                    ui.horizontal(|ui| {
                        let is_selected = state.selected.contains(&utxo.outpoint);
                        let mut checkbox_state = is_selected;
                        if ui.checkbox(&mut checkbox_state, "").changed() {
                            if checkbox_state {
                                state.selected.insert(utxo.outpoint.clone());
                            } else {
                                state.selected.remove(&utxo.outpoint);
                            }
                        }

                        ui.label(format!("{:.8} BTC", utxo.amount));
                        ui.label(format!("({})", utxo.outpoint));
                    });
                }
            });

        ui.add_space(10.0);

        // Select all / Deselect all buttons
        ui.horizontal(|ui| {
            if ui.button("Select All").clicked() {
                for utxo in &state.utxos {
                    state.selected.insert(utxo.outpoint.clone());
                }
            }
            if ui.button("Deselect All").clicked() {
                state.selected.clear();
            }
        });
    });
}
