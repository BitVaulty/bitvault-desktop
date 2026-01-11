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
use crate::ui::components::{
    badge, button, button_large, card, BadgeStyle, ButtonStyle, Colors, Spacing, Typography,
};
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
    let ctx = ui.ctx().clone();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(Spacing::XL);

            ui.label(
                Typography::heading("Send Bitcoin")
                    .color(Colors::text_primary(&ctx))
            );
            ui.add_space(Spacing::LG);

            // Show error if any
            if let Some(ref error) = state.error {
                card(ui, |ui| {
                    ui.label(
                        Typography::body(error)
                            .color(Colors::ERROR)
                    );
                });
                ui.add_space(Spacing::MD);
            }

            // Show success message if any
            if let Some(ref success) = state.success_message {
                card(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            Typography::heading_small("Transaction Sent!")
                                .color(Colors::SUCCESS)
                        );
                        ui.add_space(Spacing::SM);
                        ui.label(
                            Typography::body(success)
                                .color(Colors::text_primary(&ctx))
                        );
                        ui.add_space(Spacing::LG);
                        if button_large(ui, "Back to Dashboard").clicked() {
                            navigation.go_back();
                        }
                    });
                });
                return; // Don't show the form if transaction was successful
            }

            // Form card
            card(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(Spacing::MD);

                    // Recipient address
                    ui.label(
                        Typography::body("Recipient Address")
                            .color(Colors::text_primary(&ctx))
                    );
                    ui.add_space(Spacing::XS);
                    ui.text_edit_singleline(&mut state.recipient_address);
                    ui.add_space(Spacing::MD);

                    // Amount or send max
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.is_sending_max, "Send Max");
                        if !state.is_sending_max {
                            ui.label(
                                Typography::body("Amount (BTC)")
                                    .color(Colors::text_primary(&ctx))
                            );
                            ui.add_space(Spacing::SM);
                            ui.text_edit_singleline(&mut state.amount_btc);
                        }
                    });
                    ui.add_space(Spacing::MD);

                    // Fee rate
                    ui.label(
                        Typography::body(format!("Fee Rate: {} sat/vB", state.fee_rate))
                            .color(Colors::text_primary(&ctx))
                    );
                    ui.add_space(Spacing::XS);
                    ui.add(egui::Slider::new(&mut state.fee_rate, 1..=100));
                    ui.add_space(Spacing::MD);

                    // Description (optional)
                    ui.label(
                        Typography::body("Description (optional)")
                            .color(Colors::text_primary(&ctx))
                    );
                    ui.add_space(Spacing::XS);
                    ui.text_edit_multiline(&mut state.description);
                    ui.add_space(Spacing::MD);

                    // Recovery mode
                    ui.checkbox(&mut state.is_recovery, "Recovery Mode");

                    ui.add_space(Spacing::MD);
                });
            });

            ui.add_space(Spacing::LG);

            // Buttons - centered
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if button_large(ui, "Preview Transaction").clicked() {
                        steps::build_preview(ui, app_state, state);
                    }
                    ui.add_space(Spacing::MD);
                    if button(ui, "Cancel", ButtonStyle::Secondary).clicked() {
                        navigation.go_back();
                    }
                });
            });

            // Show preview if available
            if let Some(ref preview) = state.preview {
                ui.add_space(Spacing::LG);

                card(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(Spacing::MD);

                        ui.label(
                            Typography::heading_small("Transaction Preview")
                                .color(Colors::text_primary(&ctx))
                        );
                        ui.add_space(Spacing::MD);

                        // Amount (prominent)
                        ui.horizontal(|ui| {
                            ui.label(
                                Typography::body("Amount:")
                                    .color(Colors::text_secondary(&ctx))
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    Typography::heading_small(format!("{:.8} BTC", preview.amount))
                                        .color(Colors::ERROR)
                                );
                            });
                        });
                        ui.add_space(Spacing::SM);

                        // Fee
                        ui.horizontal(|ui| {
                            ui.label(
                                Typography::body("Fee:")
                                    .color(Colors::text_secondary(&ctx))
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    Typography::body(format!("{} sats", preview.fee))
                                        .color(Colors::text_primary(&ctx))
                                );
                            });
                        });
                        ui.add_space(Spacing::SM);

                        // Recipient
                        ui.label(
                            Typography::body("Recipient:")
                                .color(Colors::text_secondary(&ctx))
                        );
                        ui.label(
                            Typography::caption(&preview.recipient)
                                .color(Colors::text_primary(&ctx))
                                .monospace()
                        );
                        ui.add_space(Spacing::SM);

                        // Description
                        if let Some(ref desc) = preview.description {
                            if !desc.is_empty() {
                                ui.label(
                                    Typography::body("Description:")
                                        .color(Colors::text_secondary(&ctx))
                                );
                                ui.label(
                                    Typography::body(desc)
                                        .color(Colors::text_primary(&ctx))
                                );
                                ui.add_space(Spacing::SM);
                            }
                        }

                        // Network badge
                        ui.horizontal(|ui| {
                            ui.label(
                                Typography::body("Network:")
                                    .color(Colors::text_secondary(&ctx))
                            );
                            badge(ui, &preview.network, BadgeStyle::Info);
                        });
                        ui.add_space(Spacing::SM);

                        // Date
                        ui.label(
                            Typography::caption(format!("Date: {}", preview.date))
                                .color(Colors::text_muted(&ctx))
                        );

                        ui.add_space(Spacing::MD);
                    });
                });

                ui.add_space(Spacing::LG);

                // Action buttons - centered
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        if button_large(ui, "Sign & Broadcast").clicked() {
                            steps::sign_and_broadcast(ui, app_state, navigation, state);
                        }
                        ui.add_space(Spacing::MD);
                        if button(ui, "Sign with Hardware Wallet", ButtonStyle::Secondary).clicked() {
                            steps::start_hardware_wallet_signing(ui, app_state, state);
                        }
                    });
                });
            }

            // Handle hardware wallet signing flow
            match state.hw_signing_mode {
                HardwareWalletSigningMode::DisplayingQR => {
                    ui.add_space(Spacing::LG);

                    card(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(Spacing::MD);
                            render_qr_display(
                                ui,
                                app_state,
                                navigation,
                                &mut state.hw_qr_display_state,
                                "Scan with Hardware Wallet",
                                "Scan this QR code with your hardware wallet to sign the transaction",
                            );
                            ui.add_space(Spacing::MD);
                        });
                    });

                    // Check if user clicked "Done" (hardware wallet has scanned)
                    if !state.hw_qr_display_state.ur_parts.is_empty() {
                        ui.add_space(Spacing::MD);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                if button(ui, "Hardware Wallet Signed - Scan QR", ButtonStyle::Primary).clicked() {
                                    state.hw_signing_mode = HardwareWalletSigningMode::ScanningQR;
                                    state.hw_qr_scanner_state =
                                        crate::ui::hardware_wallet::QrScannerState::default();
                                }
                            });
                        });
                    }
                }
                HardwareWalletSigningMode::ScanningQR => {
                    ui.add_space(Spacing::LG);

                    card(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(Spacing::MD);
                            render_qr_scanner(
                                ui,
                                app_state,
                                navigation,
                                &mut state.hw_qr_scanner_state,
                                "Scan Signed PSBT",
                                "Scan the QR code from your hardware wallet containing the signed PSBT",
                            );
                            ui.add_space(Spacing::MD);
                        });
                    });

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

            ui.add_space(Spacing::XL);
        });
    });
}
