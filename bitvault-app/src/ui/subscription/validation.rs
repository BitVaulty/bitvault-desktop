//! Subscription Validation
//!
//! Validates subscription before critical operations

use crate::state::AppState;

/// Check subscription before performing a critical action
/// Returns Ok(()) if subscription is valid, Err(message) if expired
///
/// This should be called before:
/// - Creating a new vault (mainnet only)
/// - Sending a transaction (mainnet only)
/// - Other subscription-required operations
pub fn check_subscription_before_action(
    app_state: &mut AppState,
    action_name: &str,
) -> Result<(), String> {
    // Only validate subscription on mainnet (testnet is free)
    if app_state.network != bdk::bitcoin::Network::Bitcoin {
        return Ok(());
    }

    if let (Some(vault_service), Some(runtime)) =
        (app_state.vault_service.as_ref(), app_state.runtime.as_ref())
    {
        let result = runtime.block_on(async {
            let vs = vault_service.read().await;
            vs.validate_subscription().await
        });

        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(format!(
                "Subscription expired. Please renew your subscription to {}.\n\nError: {}",
                action_name, e
            )),
        }
    } else {
        // If vault not loaded, allow action (validation will happen when vault is created)
        Ok(())
    }
}
