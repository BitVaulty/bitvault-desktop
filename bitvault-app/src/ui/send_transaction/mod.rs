//! Send Transaction UI
//!
//! Implements the send transaction flow:
//! 1. Enter recipient address
//! 2. Enter amount (or send max)
//! 3. Set fee rate
//! 4. Add description (optional)
//! 5. Preview transaction
//! 6. Sign and broadcast

mod steps;

use crate::state::{AppState, Navigation};
use crate::ui::hardware_wallet::{render_qr_display, render_qr_scanner};
use crate::ui::pin::render_pin_verification;
use eframe::egui;

/// Send transaction state
pub struct SendTransactionState {
    pub recipient_address: String,
    pub amount_btc: String,
    pub fee_rate: u64, // sat/vB
    pub description: String,
    pub is_sending_max: bool,
    pub is_recovery: bool,
    pub preview: Option<bitvault_common::types::TransactionPreview>,
    pub error: Option<String>,
    pub success_message: Option<String>,
    pub is_building: bool,
    pub is_signing: bool,
    pub pin_verification: crate::ui::pin::PinVerificationState,
    // Hardware wallet signing state
    pub hw_signing_mode: HardwareWalletSigningMode,
    pub hw_qr_display_state: crate::ui::hardware_wallet::QrDisplayState,
    pub hw_qr_scanner_state: crate::ui::hardware_wallet::QrScannerState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareWalletSigningMode {
    None,
    DisplayingQR, // Showing QR for hardware wallet to scan
    ScanningQR,   // Scanning QR from hardware wallet
}

impl Default for SendTransactionState {
    fn default() -> Self {
        Self {
            recipient_address: String::new(),
            amount_btc: String::new(),
            fee_rate: 10, // Default 10 sat/vB
            description: String::new(),
            is_sending_max: false,
            is_recovery: false,
            preview: None,
            error: None,
            success_message: None,
            is_building: false,
            is_signing: false,
            pin_verification: crate::ui::pin::PinVerificationState::new(),
            hw_signing_mode: HardwareWalletSigningMode::None,
            hw_qr_display_state: crate::ui::hardware_wallet::QrDisplayState::default(),
            hw_qr_scanner_state: crate::ui::hardware_wallet::QrScannerState::default(),
        }
    }
}

/// Render send transaction flow
pub fn render(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    navigation: &mut Navigation,
    state: &mut SendTransactionState,
) {
    // Check PIN before showing sensitive content if PIN is set
    let pin_service = bitvault_common::PinService::new();
    let requires_pin = pin_service.has_pin();

    // Render PIN verification modal if needed
    if requires_pin && !state.pin_verification.is_verified() {
        if render_pin_verification(ui.ctx(), &mut state.pin_verification) {
            // PIN verified, continue with transaction
        } else if !state.pin_verification.is_visible() {
            // Show PIN verification modal
            state.pin_verification.show();
        }
        return; // Don't show transaction form until PIN is verified
    }
    ui.vertical_centered(|ui| {
        ui.heading("Send Transaction");
        ui.add_space(20.0);

        // Show error if any
        if let Some(ref error) = state.error {
            ui.colored_label(egui::Color32::RED, error);
            ui.add_space(10.0);
        }

        // Show success message if any
        if let Some(ref success) = state.success_message {
            ui.colored_label(egui::Color32::GREEN, success);
            ui.add_space(10.0);

            // Add button to go back to dashboard - centered
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button("Back to Dashboard").clicked() {
                    navigation.go_back();
                }
            });
            return; // Don't show the form if transaction was successful
        }

        // Recipient address
        ui.label("Recipient Address:");
        ui.text_edit_singleline(&mut state.recipient_address);
        ui.add_space(10.0);

        // Amount or send max
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.is_sending_max, "Send Max");
            if !state.is_sending_max {
                ui.label("Amount (BTC):");
                ui.text_edit_singleline(&mut state.amount_btc);
            }
        });
        ui.add_space(10.0);

        // Fee rate
        ui.label("Fee Rate (sat/vB):");
        ui.add(egui::Slider::new(&mut state.fee_rate, 1..=100));
        ui.label(format!("{} sat/vB", state.fee_rate));
        ui.add_space(10.0);

        // Description (optional)
        ui.label("Description (optional):");
        ui.text_edit_multiline(&mut state.description);
        ui.add_space(10.0);

        // Recovery mode
        ui.checkbox(&mut state.is_recovery, "Recovery Mode");

        ui.add_space(20.0);

        // Buttons - centered
        let button_width = 180.0;
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
            egui::Sense::click()
        );
        let mut button_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
        if button_ui.button("Preview Transaction").clicked() {
            steps::build_preview(ui, app_state, state);
        }
        button_ui.add_space(10.0);
        if button_ui.button("Cancel").clicked() {
            navigation.go_back();
        }

        // Show preview if available
        if let Some(ref preview) = state.preview {
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label("Transaction Preview:");
            ui.label(format!("Amount: {:.8} BTC", preview.amount));
            ui.label(format!("Fee: {} sats", preview.fee));
            ui.label(format!("Recipient: {}", preview.recipient));
            if let Some(ref desc) = preview.description {
                ui.label(format!("Description: {}", desc));
            }
            ui.label(format!("Network: {}", preview.network));
            ui.label(format!("Date: {}", preview.date));

            ui.add_space(10.0);

            // Buttons - centered
            let button_width = 200.0;
            let (rect, _) = ui.allocate_exact_size(
                egui::Vec2::new(button_width * 2.0 + 10.0, 30.0),
                egui::Sense::click()
            );
            let mut button_ui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::Center));
            if button_ui.button("Sign & Broadcast").clicked() {
                steps::sign_and_broadcast(ui, app_state, navigation, state);
            }
            button_ui.add_space(10.0);
            if button_ui.button("Sign with Hardware Wallet").clicked() {
                steps::start_hardware_wallet_signing(ui, app_state, state);
            }
        }

        // Handle hardware wallet signing flow
        match state.hw_signing_mode {
            HardwareWalletSigningMode::DisplayingQR => {
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                render_qr_display(
                    ui,
                    app_state,
                    navigation,
                    &mut state.hw_qr_display_state,
                    "Scan with Hardware Wallet",
                    "Scan this QR code with your hardware wallet to sign the transaction",
                );

                // Check if user clicked "Done" (hardware wallet has scanned)
                if !state.hw_qr_display_state.ur_parts.is_empty() {
                    // Move to scanning mode - centered
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        if ui.button("Hardware Wallet Signed - Scan QR").clicked() {
                            state.hw_signing_mode = HardwareWalletSigningMode::ScanningQR;
                            state.hw_qr_scanner_state =
                                crate::ui::hardware_wallet::QrScannerState::default();
                        }
                    });
                }
            }
            HardwareWalletSigningMode::ScanningQR => {
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                render_qr_scanner(
                    ui,
                    app_state,
                    navigation,
                    &mut state.hw_qr_scanner_state,
                    "Scan Signed PSBT",
                    "Scan the QR code from your hardware wallet containing the signed PSBT",
                );

                // If QR scanner succeeded, decode and send to convenience service
                if state.hw_qr_scanner_state.success {
                    let signed_psbt = state.hw_qr_scanner_state.decoded_psbt.clone();
                    if let Some(ref psbt) = signed_psbt {
                        steps::send_hardware_wallet_signed_psbt(
                            ui, app_state, navigation, state, psbt,
                        );
                    }
                }
            }
            HardwareWalletSigningMode::None => {
                // Normal flow
            }
        }
    });
}
