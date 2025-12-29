//! UTXO Selection View
//!
//! Standalone view for selecting and managing UTXOs

use eframe::egui;
use crate::state::{AppState, Navigation};
use bitvault_common::types::OldUtxo;

/// State for UTXO selection view
#[derive(Default)]
pub struct UtxoSelectionViewState {
    pub utxos: Vec<OldUtxo>,
    pub selected: std::collections::HashSet<String>, // OutPoint as string
    pub is_loading: bool,
    pub error: Option<String>,
    pub show_recovery: bool, // true for recovery (>1 year), false for refresh (>6 months)
}


impl UtxoSelectionViewState {
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
    
    pub fn refresh(&mut self, app_state: &mut AppState, is_recovery: bool) {
        self.is_loading = true;
        self.error = None;
        self.selected.clear();
        
        if let (Some(vault_service), Some(runtime)) = 
            (app_state.vault_service.as_ref(), app_state.runtime.as_ref()) {
            
            let result = runtime.block_on(async {
                let mut vs = vault_service.write().await;
                // get_old_utxos takes a boolean: true for refresh (>6 months), false for recovery (>1 year)
                vs.get_old_utxos(!is_recovery).await
            });
            
            match result {
                Ok(utxos) => {
                    self.utxos = utxos;
                    self.is_loading = false;
                }
                Err(e) => {
                    self.error = Some(format!("Failed to load UTXOs: {}", e));
                    self.is_loading = false;
                }
            }
        } else {
            self.error = Some("Vault not loaded or runtime not available".to_string());
            self.is_loading = false;
        }
    }
}

/// Render UTXO selection view
pub fn render_utxo_selection_view(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut UtxoSelectionViewState,
) {
    if !app_state.is_vault_loaded() {
        ui.label("No vault loaded");
        return;
    }
    
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("UTXO Selection").heading());
        ui.add_space(10.0);
        
        // Mode selection
        ui.horizontal(|ui| {
            ui.label("Mode:");
            if ui.selectable_label(!state.show_recovery, "Refresh (>6 months)").clicked() {
                state.show_recovery = false;
                state.refresh(app_state, false);
            }
            if ui.selectable_label(state.show_recovery, "Recovery (>1 year)").clicked() {
                state.show_recovery = true;
                state.refresh(app_state, true);
            }
        });
        
        ui.add_space(10.0);
        
        // Refresh button
        if ui.button("🔄 Refresh UTXO List").clicked() {
            state.refresh(app_state, state.show_recovery);
        }
        
        ui.add_space(10.0);
        ui.separator();
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
                ui.label(if state.show_recovery {
                    "All UTXOs are less than 1 year old"
                } else {
                    "All UTXOs are less than 6 months old"
                });
            });
            return;
        }
        
        // Total selected amount
        let total = state.total_selected_amount();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Total Selected:").strong());
            ui.label(format!("{:.8} BTC", total));
            ui.label(format!("({} UTXO(s))", state.selected.len()));
        });
        ui.add_space(10.0);
        
        // Select all / Deselect all
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
        
        ui.add_space(10.0);
        
        // UTXO list with checkboxes
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for utxo in &state.utxos {
                    let is_selected = state.selected.contains(&utxo.outpoint);
                    
                    ui.horizontal(|ui| {
                        let mut checkbox_value = is_selected;
                        if ui.checkbox(&mut checkbox_value, "").changed() {
                            if checkbox_value {
                                state.selected.insert(utxo.outpoint.clone());
                            } else {
                                state.selected.remove(&utxo.outpoint);
                            }
                        }
                        
                        ui.vertical(|ui| {
                            ui.label(format!("OutPoint: {}", utxo.outpoint));
                            ui.label(format!("Amount: {:.8} BTC", utxo.amount));
                        });
                    });
                    
                    ui.separator();
                }
            });
        
        ui.add_space(10.0);
        
        // Action buttons
        if state.has_selection() {
            ui.horizontal(|ui| {
                if ui.button("Create Recovery Transaction").clicked() {
                    navigation.navigate_to(crate::state::View::Recovery);
                }
                if ui.button("Create Refresh Transaction").clicked() {
                    navigation.navigate_to(crate::state::View::UtxoRefresh);
                }
            });
        }
    });
}
