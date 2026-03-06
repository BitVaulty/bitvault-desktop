//! Batch QR Scanner for Multi-Part UR Codes
//!
//! Enhanced scanner for hardware wallets that use multi-part UR (Jade, Foundation Passport)
//! Tracks scanning progress and provides clear feedback

use crate::state::{AppState, Navigation};
use crate::ui::components::{button, button_large, ButtonStyle, Spacing};
use crate::utils::qr::decode_qr_from_file;
use bitvault_common::ur::MultiPartUrDecoder;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;

/// State for batch QR code scanner (multi-part UR)
pub struct BatchQrScannerState {
    /// Multi-part UR decoder
    decoder: MultiPartUrDecoder,
    /// Set of scanned UR parts (for deduplication)
    scanned_parts_set: HashSet<String>,
    /// Ordered list of scanned parts (for display)
    pub scanned_parts: Vec<String>,
    /// Whether scanning is active
    pub is_scanning: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Whether decoding is complete
    pub success: bool,
    /// Decoded PSBT (base64) when complete
    pub decoded_psbt: Option<String>,
    /// Selected file for scanning
    pub selected_file: Option<PathBuf>,
    /// Whether file selection dialog is pending
    pub pending_file_selection: bool,
    /// Estimated total parts (based on first scan, may not be exact for fountain codes)
    pub estimated_total_parts: Option<usize>,
}

impl Default for BatchQrScannerState {
    fn default() -> Self {
        Self {
            decoder: MultiPartUrDecoder::new(),
            scanned_parts_set: HashSet::new(),
            scanned_parts: Vec::new(),
            is_scanning: true,
            error: None,
            success: false,
            decoded_psbt: None,
            selected_file: None,
            pending_file_selection: false,
            estimated_total_parts: None,
        }
    }
}

impl BatchQrScannerState {
    /// Reset scanner state for new scan
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Process a scanned QR code part
    pub fn process_scanned_part(&mut self, ur_part: String) -> Result<bool, String> {
        // Deduplicate - don't process same part twice
        if self.scanned_parts_set.contains(&ur_part) {
            return Ok(self.success);
        }

        // Add to scanned set and list
        self.scanned_parts_set.insert(ur_part.clone());
        self.scanned_parts.push(ur_part.clone());

        // Try to receive part in decoder
        match self.decoder.receive_part(&ur_part) {
            Ok(is_complete) => {
                if is_complete {
                    // Decoding complete - extract message
                    match self.decoder.get_message() {
                        Ok(message_bytes) => {
                            // Convert to base64 PSBT
                            use base64::{engine::general_purpose, Engine as _};
                            let psbt_base64 = general_purpose::STANDARD.encode(&message_bytes);
                            self.decoded_psbt = Some(psbt_base64);
                            self.success = true;
                            self.is_scanning = false;
                            self.error = None;
                            Ok(true)
                        }
                        Err(e) => {
                            self.error = Some(format!("Failed to decode message: {}", e));
                            Err(self.error.clone().unwrap())
                        }
                    }
                } else {
                    // More parts needed - continue scanning
                    self.error = None;
                    Ok(false)
                }
            }
            Err(e) => {
                let err_msg = format!("Invalid UR part: {}", e);
                self.error = Some(err_msg.clone());
                Err(err_msg)
            }
        }
    }

    /// Get progress information for display
    pub fn progress_info(&self) -> String {
        if self.success {
            "Complete!".to_string()
        } else if let Some(total) = self.estimated_total_parts {
            format!("Scanned {} of ~{} parts", self.scanned_parts.len(), total)
        } else {
            format!("Scanned {} part(s)", self.scanned_parts.len())
        }
    }
}

/// Render batch QR scanner for multi-part UR codes
pub fn render_batch_qr_scanner(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut BatchQrScannerState,
    title: &str,
    description: &str,
) {
    ui.vertical_centered(|ui| {
        ui.heading(title);
        ui.add_space(Spacing::MD);
        ui.label(description);
        ui.add_space(Spacing::LG);

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(Spacing::SM);
        }

        // Show success message
        if state.success {
            ui.colored_label(egui::Color32::GREEN, "✓ All QR codes scanned successfully!");
            if let Some(ref psbt) = state.decoded_psbt {
                ui.label(format!("PSBT decoded: {}...", &psbt[..psbt.len().min(50)]));
            }
            ui.add_space(Spacing::MD);

            if button_large(ui, "Continue").clicked() {
                navigation.go_back();
            }
            return;
        }

        // Progress indicator
        if !state.scanned_parts.is_empty() {
            ui.add_space(Spacing::SM);
            let progress_text = state.progress_info();
            ui.label(egui::RichText::new(&progress_text).strong());
            ui.add_space(Spacing::SM);
        }

        // Scanner UI
        if state.is_scanning {
            ui.label("Scan QR code from image file:");
            ui.add_space(Spacing::SM);

            // File selection button - centered
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if button(ui, "Select Image File", ButtonStyle::Secondary).clicked() {
                    state.pending_file_selection = true;
                }
            });

            // Show selected file
            if let Some(ref file_path) = state.selected_file {
                ui.add_space(Spacing::SM);
                ui.label(format!("Selected: {}", file_path.display()));
                ui.add_space(Spacing::XS);

                // Scan button - centered
                let file_path_clone = file_path.clone();
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    if button(ui, "Scan QR Code from File", ButtonStyle::Primary).clicked() {
                        match decode_qr_from_file(&file_path_clone) {
                            Ok(decoded) => {
                                match state.process_scanned_part(decoded) {
                                    Ok(is_complete) => {
                                        if is_complete {
                                            // Success - PSBT decoded
                                            state.error = None;
                                        } else {
                                            // More parts needed - clear file selection for next scan
                                            state.selected_file = None;
                                            state.error = None;
                                        }
                                    }
                                    Err(e) => {
                                        state.error = Some(e);
                                    }
                                }
                            }
                            Err(e) => {
                                state.error = Some(format!("Failed to decode QR code: {}", e));
                            }
                        }
                    }
                });
            }

            ui.add_space(Spacing::MD);
            ui.separator();
            ui.add_space(Spacing::SM);

            // Manual input option (for testing/debugging)
            ui.label("Or enter UR string manually:");
            ui.add_space(Spacing::XS);

            let mut manual_input = String::new();
            ui.text_edit_singleline(&mut manual_input);

            if ui.button("Submit UR String").clicked() && !manual_input.is_empty() {
                match state.process_scanned_part(manual_input.clone()) {
                    Ok(is_complete) => {
                        if is_complete {
                            state.error = None;
                        } else {
                            state.error = None;
                            manual_input.clear();
                        }
                    }
                    Err(e) => {
                        state.error = Some(e);
                    }
                }
            }

            // Instructions for multi-part scanning
            if state.scanned_parts.is_empty() {
                ui.add_space(Spacing::MD);
                ui.label("ℹ️ For hardware wallets with multi-part QR codes:");
                ui.label("Scan each QR code in sequence until all parts are received.");
            } else {
                ui.add_space(Spacing::MD);
                ui.label("✓ Continue scanning additional QR codes...");
            }
        }

        // Handle file selection (non-blocking)
        if state.pending_file_selection {
            state.pending_file_selection = false;
            // Use rfd for file dialog
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image files", &["png", "jpg", "jpeg", "gif", "bmp"])
                .pick_file()
            {
                state.selected_file = Some(path);
            }
        }

        ui.add_space(Spacing::XL);
        ui.separator();
        ui.add_space(Spacing::SM);

        // Reset button (if scanning)
        if state.is_scanning && !state.scanned_parts.is_empty() {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if button(ui, "Reset Scan", ButtonStyle::Text).clicked() {
                    state.reset();
                }
            });
            ui.add_space(Spacing::SM);
        }

        // Cancel button - centered
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            if button(ui, "Cancel", ButtonStyle::Text).clicked() {
                navigation.go_back();
            }
        });
    });
}
