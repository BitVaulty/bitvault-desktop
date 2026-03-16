//! Fee Rate Setting UI
//!
//! Allows users to configure default fee rates and view fee recommendations

use crate::state::AppState;
use eframe::egui;

/// State for fee rate setting
#[derive(Default)]
pub struct FeeRateSettingState {
    pub custom_fee_rate: Option<u64>, // sat/vB
    pub recommended_fees: Option<RecommendedFees>,
    pub is_loading: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RecommendedFees {
    pub fastest: u64,   // sat/vB
    pub half_hour: u64, // sat/vB
    pub hour: u64,      // sat/vB
    pub economy: u64,   // sat/vB
    pub minimum: u64,   // sat/vB
}

impl FeeRateSettingState {
    pub fn load_recommended_fees(&mut self, app_state: &mut AppState) {
        self.is_loading = true;
        self.error = None;

        if let Some(ref runtime) = app_state.runtime {
            let handle = runtime.handle().clone();
            let result: Result<RecommendedFees, String> = handle.block_on(async {
                // Fetch recommended fees from mempool service
                // This is a placeholder - actual implementation would use MempoolService
                // For now, return default values
                Ok(RecommendedFees {
                    fastest: 50,
                    half_hour: 25,
                    hour: 15,
                    economy: 5,
                    minimum: 1,
                })
            });

            match result {
                Ok(fees) => {
                    self.recommended_fees = Some(fees);
                    self.is_loading = false;
                }
                Err(e) => {
                    self.error = Some(format!("Failed to load fees: {}", crate::utils::sanitize_error_for_ui(&e)));
                    self.is_loading = false;
                }
            }
        } else {
            self.error = Some("Runtime not available".to_string());
            self.is_loading = false;
        }
    }
}

/// Render fee rate setting view
pub fn render_fee_rate_setting(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut FeeRateSettingState,
) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("Fee Rate Settings").heading());
        ui.add_space(10.0);

        ui.label("Configure default fee rates for transactions.");
        ui.label("Fee rates are in satoshis per virtual byte (sat/vB).");
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Load recommended fees button
        if ui.button("🔄 Load Recommended Fees").clicked() {
            state.load_recommended_fees(app_state);
        }

        ui.add_space(10.0);

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }

        // Show loading state
        if state.is_loading {
            ui.label("Loading recommended fees...");
            return;
        }

        // Show recommended fees
        if let Some(ref fees) = state.recommended_fees {
            ui.label(egui::RichText::new("Recommended Fees:").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Fastest (next block):");
                ui.label(format!("{} sat/vB", fees.fastest));
                if ui.button("Use").clicked() {
                    state.custom_fee_rate = Some(fees.fastest);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Half hour:");
                ui.label(format!("{} sat/vB", fees.half_hour));
                if ui.button("Use").clicked() {
                    state.custom_fee_rate = Some(fees.half_hour);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Hour:");
                ui.label(format!("{} sat/vB", fees.hour));
                if ui.button("Use").clicked() {
                    state.custom_fee_rate = Some(fees.hour);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Economy:");
                ui.label(format!("{} sat/vB", fees.economy));
                if ui.button("Use").clicked() {
                    state.custom_fee_rate = Some(fees.economy);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Minimum:");
                ui.label(format!("{} sat/vB", fees.minimum));
                if ui.button("Use").clicked() {
                    state.custom_fee_rate = Some(fees.minimum);
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
        }

        // Custom fee rate input
        ui.label(egui::RichText::new("Custom Fee Rate:").strong());
        ui.add_space(5.0);

        let mut fee_rate_input = state
            .custom_fee_rate
            .map(|f| f.to_string())
            .unwrap_or_default();

        ui.horizontal(|ui| {
            ui.label("Fee rate (sat/vB):");
            ui.text_edit_singleline(&mut fee_rate_input);

            if ui.button("Set").clicked() {
                match fee_rate_input.parse::<u64>() {
                    Ok(rate) if rate > 0 => {
                        state.custom_fee_rate = Some(rate);
                    }
                    Ok(_) => {
                        state.error = Some("Fee rate must be greater than 0".to_string());
                    }
                    Err(_) => {
                        state.error = Some("Invalid fee rate format".to_string());
                    }
                }
            }
        });

        if let Some(rate) = state.custom_fee_rate {
            ui.add_space(5.0);
            ui.label(format!("Current custom fee rate: {} sat/vB", rate));
            if ui.button("Clear").clicked() {
                state.custom_fee_rate = None;
            }
        }

        ui.add_space(10.0);

        // Note about fee rates
        ui.separator();
        ui.add_space(10.0);
        ui.label(egui::RichText::new("Note:").strong());
        ui.label("Fee rates affect how quickly your transaction is confirmed.");
        ui.label("Higher fees = faster confirmation, lower fees = slower confirmation.");
        ui.label("Custom fee rates will be used as defaults for new transactions.");
    });
}
