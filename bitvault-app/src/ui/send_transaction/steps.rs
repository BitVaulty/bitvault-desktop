//! Send transaction step implementations

use crate::state::{AppState, Navigation};
use crate::ui::send_transaction::{HardwareWalletSigningMode, SendTransactionState};
use eframe::egui;

/// Build transaction preview
pub fn build_preview(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut SendTransactionState,
) {
    if state.is_building {
        return;
    }

    // Validate inputs
    if state.recipient_address.is_empty() {
        state.error = Some("Recipient address is required".to_string());
        return;
    }

    if !state.is_sending_max && state.amount_btc.is_empty() {
        state.error = Some("Amount is required (or select 'Send Max')".to_string());
        return;
    }

    // Parse amount
    let amount_btc = if state.is_sending_max {
        0.0 // Will be calculated
    } else {
        match state.amount_btc.parse::<f64>() {
            Ok(amt) if amt > 0.0 => amt,
            _ => {
                state.error = Some("Invalid amount".to_string());
                return;
            }
        }
    };

    state.is_building = true;
    state.error = None;

    // Build preview using VaultService
    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let recipient = state.recipient_address.clone();
        let fee_rate = state.fee_rate;
        let description = if state.description.is_empty() {
            None
        } else {
            Some(state.description.clone())
        };
        let is_sending_max = state.is_sending_max;
        let is_recovery = state.is_recovery;

        let result = runtime.block_on(async {
            let mut vs = vault_service.write().await;
            vs.build_transaction_preview(
                &recipient,
                amount_btc,
                fee_rate,
                description.as_deref(),
                is_sending_max,
                is_recovery,
                None, // utxos_to_spend - None means BDK selects automatically
            )
            .await
        });

        match result {
            Ok(preview) => {
                state.preview = Some(preview);
                state.is_building = false;
            }
            Err(e) => {
                state.error = Some(format!("Failed to build transaction: {}", e));
                state.is_building = false;
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.is_building = false;
    }
}

/// Sign and broadcast transaction
pub fn sign_and_broadcast(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    _navigation: &mut Navigation,
    state: &mut SendTransactionState,
) {
    // Check PIN before signing (if PIN is set)
    let pin_service = bitvault_common::PinService::new();
    if pin_service.has_pin() && !state.pin_verification.is_verified() {
        // Show PIN verification modal
        state.pin_verification.show();
        return;
    }

    // Validate subscription before sending (mainnet only)
    if let Err(error_msg) = crate::ui::subscription::validation::check_subscription_before_action(
        app_state,
        "send transactions",
    ) {
        state.error = Some(error_msg);
        return;
    }
    if state.is_signing {
        return;
    }

    if let Some(ref preview) = state.preview {
        state.is_signing = true;
        state.error = None;

        // Get the transaction parameters from state
        let destination = state.recipient_address.clone();
        let amount_btc_str = state.amount_btc.clone();
        let amount_btc = amount_btc_str.parse::<f64>().unwrap_or(0.0);
        let fee_rate = state.fee_rate;
        let description = if state.description.is_empty() {
            None
        } else {
            Some(state.description.clone())
        };
        let is_sending_max = state.is_sending_max;
        let is_recovery = state.is_recovery;

        // Get runtime handle
        if let Some(ref runtime) = app_state.runtime {
            let handle = runtime.handle().clone();

            // Get vault service
            if let Some(vault_service) = app_state.get_vault_service() {
                // Sign and send transaction using the new method
                // This builds, signs, and sends in one operation, avoiding type conversion issues
                let result = handle.block_on(async {
                    let mut service_guard = vault_service.write().await;
                    service_guard
                        .sign_and_send_transaction(
                            &destination,
                            amount_btc,
                            fee_rate,
                            description.as_deref(),
                            is_sending_max,
                            is_recovery,
                        )
                        .await
                });

                match result {
                    Ok(response) => {
                        // Transaction sent successfully
                        if let Some(txid) = response.txid {
                            state.error = None;
                            state.success_message =
                                Some(format!("Transaction sent successfully! TXID: {}", txid));

                            // Show desktop notification
                            let notification_service = app_state.notification_service.clone();
                            let txid_clone = txid.clone();
                            let amount_btc = amount_btc;
                            if let Some(ref runtime) = app_state.runtime {
                                let handle = runtime.handle().clone();
                                handle.spawn(async move {
                                    let _ = notification_service
                                        .notify_transaction_sent(&txid_clone, amount_btc)
                                        .await;
                                });
                            }
                        } else if let Some(message) = response.message {
                            state.error = None;
                            state.success_message = Some(message);
                        } else {
                            state.error = None;
                            state.success_message =
                                Some("Transaction sent successfully!".to_string());
                        }
                        state.is_signing = false;
                        // Reset PIN verification after successful transaction
                        state.pin_verification.reset();
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to send transaction: {}", e));
                        state.is_signing = false;
                        // Reset PIN verification on error
                        state.pin_verification.reset();
                    }
                }
            } else {
                state.error = Some("Vault service not available".to_string());
                state.is_signing = false;
                state.pin_verification.reset();
            }
        } else {
            state.error = Some("Runtime not available".to_string());
            state.is_signing = false;
            state.pin_verification.reset();
        }
    } else {
        state.error = Some("No transaction preview available".to_string());
    }
}

/// Start hardware wallet signing flow
/// Encodes PSBT to UR parts and displays QR codes
pub fn start_hardware_wallet_signing(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    state: &mut SendTransactionState,
) {
    if let Some(ref preview) = state.preview {
        if let (Some(vault_service), Some(runtime)) =
            (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
        {
            let psbt_base64 = preview.psbt.clone();

            let result = runtime.block_on(async {
                let vs = vault_service.read().await;
                vs.encode_psbt_to_ur_parts(&psbt_base64, Some(200))
            });

            match result {
                Ok(ur_parts) => {
                    state.hw_qr_display_state.ur_parts = ur_parts;
                    state.hw_qr_display_state.current_part = 0;
                    state.hw_signing_mode = HardwareWalletSigningMode::DisplayingQR;
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(format!("Failed to encode PSBT for hardware wallet: {}", e));
                }
            }
        } else {
            state.error = Some("Vault not loaded or runtime not available".to_string());
        }
    } else {
        state.error =
            Some("No transaction preview available. Please build preview first.".to_string());
    }
}

/// Send hardware wallet signed PSBT to convenience service
pub fn send_hardware_wallet_signed_psbt(
    _ui: &mut egui::Ui,
    app_state: &mut AppState,
    _navigation: &mut Navigation,
    state: &mut SendTransactionState,
    signed_psbt_base64: &str,
) {
    if state.is_signing {
        return;
    }

    state.is_signing = true;
    state.error = None;

    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let psbt_base64 = signed_psbt_base64.to_string();
        // Get vault address from vault service
        let vault_address = runtime.block_on(async {
            let vs = vault_service.read().await;
            vs.get_address().unwrap_or_else(|_| String::new())
        });
        let network = app_state.network;

        let result = runtime.block_on(async {
            let convenience_service = bitvault_common::ConvenienceService::new(None);
            let network_id = match network {
                bdk::bitcoin::Network::Bitcoin => 0,
                bdk::bitcoin::Network::Testnet => 1,
                bdk::bitcoin::Network::Signet => 1,
                bdk::bitcoin::Network::Regtest => 1,
                _ => 1,
            };
            convenience_service
                .send_signed_psbt(&vault_address, &psbt_base64, network_id)
                .await
        });

        match result {
            Ok(response) => {
                state.error = None;
                if let Some(message) = response.message {
                    state.success_message = Some(message);
                } else {
                    state.success_message = Some("Transaction sent successfully!".to_string());
                }

                // Show desktop notification for hardware wallet signed transaction
                let notification_service = app_state.notification_service.clone();
                if let Some(ref runtime) = app_state.runtime {
                    let handle = runtime.handle().clone();
                    // Extract amount from preview if available
                    let amount_btc = if let Some(ref preview) = state.preview {
                        preview.amount
                    } else {
                        0.0
                    };
                    handle.spawn(async move {
                        let _ = notification_service
                            .notify_transaction_sent("HW Signed", amount_btc)
                            .await;
                    });
                }

                state.is_signing = false;
                state.hw_signing_mode = HardwareWalletSigningMode::None;
                state.pin_verification.reset();
            }
            Err(e) => {
                state.error = Some(format!("Failed to send transaction: {}", e));
                state.is_signing = false;
                state.hw_signing_mode = HardwareWalletSigningMode::None;
                state.pin_verification.reset();
            }
        }
    } else {
        state.error = Some("Vault not loaded or runtime not available".to_string());
        state.is_signing = false;
        state.hw_signing_mode = HardwareWalletSigningMode::None;
    }
}
