//! Settings Tab (Dashboard)
//!
//! Basic settings accessible from dashboard tab

use crate::state::{AppState, Navigation};
use eframe::egui;

pub fn render(ui: &mut egui::Ui, app_state: &mut AppState, navigation: &mut Navigation) {
    ui.vertical(|ui| {
        ui.heading("Settings");

        ui.separator();
        ui.add_space(10.0);

        // Network selection
        ui.label("Network:");
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            let current_network = app_state.network;
            let networks = [
                (bdk::bitcoin::Network::Bitcoin, "Mainnet"),
                (bdk::bitcoin::Network::Testnet, "Testnet"),
                (bdk::bitcoin::Network::Signet, "Signet"),
                (bdk::bitcoin::Network::Regtest, "Regtest"),
            ];

            for (network, name) in networks.iter() {
                let is_selected = current_network == *network;
                if ui.selectable_label(is_selected, *name).clicked() && !is_selected {
                    // Update network and reload vault
                    match app_state.update_network(*network) {
                        Ok(()) => {
                            // If no vault loaded, navigate to vault selection
                            if !app_state.is_vault_loaded() {
                                navigation.navigate_to(crate::state::View::VaultSelection);
                            }
                        }
                        Err(e) => {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("Failed to change network: {}", crate::utils::sanitize_error_for_ui(&e)),
                            );
                        }
                    }
                }
            }
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Vault info section
        ui.label("Vault Information:");
        ui.add_space(5.0);

        if app_state.is_vault_loaded() {
            // Show current vault metadata
            if let Some(current_metadata) = app_state.get_current_vault_metadata() {
                ui.horizontal(|ui| {
                    ui.label("Current Vault:");
                    ui.label(format!(
                        "{} ({})",
                        current_metadata.name, current_metadata.network
                    ));
                });
                ui.add_space(5.0);
            }

            ui.label("Status: ✓ Loaded");

            // Show vault data if available and extract address for switcher
            let current_address = {
                let vault_data = match app_state.vault_data.lock() {
                    Ok(data) => data,
                    Err(_) => {
                        ui.label("Error: Mutex poisoned");
                        return;
                    }
                };
                let address = vault_data.receive_address.clone();
                let balance = vault_data.confirmed_balance;

                if let Some(ref addr) = address {
                    ui.horizontal(|ui| {
                        ui.label("Address:");
                        ui.label(addr);
                    });
                }
                if let Some(bal) = balance {
                    ui.label(format!("Balance: {:.8} BTC", bal as f64 / 100_000_000.0));
                }

                address
            };

            ui.add_space(10.0);

            // Vault switcher
            ui.label("Switch Vault:");
            ui.add_space(5.0);

            match AppState::list_vaults() {
                Ok(all_vaults) => {
                    // Filter vaults for current network
                    let network_vaults: Vec<_> = all_vaults
                        .iter()
                        .filter(|v| {
                            v.network
                                == match app_state.network {
                                    bdk::bitcoin::Network::Bitcoin => "mainnet",
                                    bdk::bitcoin::Network::Testnet => "testnet",
                                    bdk::bitcoin::Network::Signet => "signet",
                                    bdk::bitcoin::Network::Regtest => "regtest",
                                    _ => "mainnet",
                                }
                        })
                        .collect();

                    if network_vaults.len() > 1 {
                        // Show dropdown/combo box for vault switching
                        egui::ComboBox::from_id_source("vault_switcher")
                            .selected_text(
                                current_address
                                    .as_ref()
                                    .and_then(|addr| {
                                        network_vaults.iter().find(|v| v.address == *addr)
                                    })
                                    .map(|v| {
                                        format!(
                                            "{} ({})",
                                            v.name,
                                            &v.address[0..std::cmp::min(16, v.address.len())]
                                        )
                                    })
                                    .unwrap_or_else(|| "Select vault".to_string()),
                            )
                            .show_ui(ui, |ui| {
                                for vault in &network_vaults {
                                    let is_current = current_address
                                        .as_ref()
                                        .map(|addr| vault.address == *addr)
                                        .unwrap_or(false);

                                    let label = if is_current {
                                        format!(
                                            "✓ {} ({})",
                                            vault.name,
                                            &vault.address
                                                [0..std::cmp::min(16, vault.address.len())]
                                        )
                                    } else {
                                        format!(
                                            "{} ({})",
                                            vault.name,
                                            &vault.address
                                                [0..std::cmp::min(16, vault.address.len())]
                                        )
                                    };

                                    if ui.selectable_label(is_current, label).clicked()
                                        && !is_current
                                    {
                                        // Switch to this vault
                                        let metadata = (*vault).clone();

                                        // Unload current vault
                                        app_state.unload_vault();

                                        // Load new vault
                                        if let Some(ref runtime) = app_state.runtime {
                                            let handle = runtime.handle().clone();
                                            match handle.block_on(
                                                app_state.load_vault_from_metadata(&metadata),
                                            ) {
                                                Ok(_) => {
                                                    // Set as active vault for this network
                                                    let _ = app_state
                                                        .settings_manager
                                                        .set_active_vault(
                                                            &metadata.network,
                                                            &metadata.address,
                                                        );
                                                    // Refresh vault data
                                                    if let Some(ref mut handler) =
                                                        app_state.async_handler
                                                    {
                                                        handler.fetch_balance();
                                                        handler.fetch_address();
                                                    }
                                                }
                                                Err(e) => {
                                                    ui.colored_label(
                                                        egui::Color32::RED,
                                                        format!("Failed to load vault: {}", crate::utils::sanitize_error_for_ui(&e)),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                    } else {
                        ui.label(format!(
                            "Only one vault available on {} network",
                            app_state.network
                        ));
                    }
                }
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Failed to list vaults: {}", crate::utils::sanitize_error_for_ui(&e)));
                }
            }

            ui.add_space(10.0);

            // Vault management
            ui.horizontal(|ui| {
                if ui.button("Manage Vaults").clicked() {
                    navigation.navigate_to(crate::state::View::VaultSelection);
                }

                // Export current vault metadata
                let export_address = current_address.clone();
                if ui.button("Export Vault").clicked() {
                    if let Some(ref address) = export_address {
                        match AppState::list_vaults() {
                            Ok(vaults) => {
                                if let Some(metadata) =
                                    vaults.iter().find(|v| v.address == *address)
                                {
                                    match serde_json::to_string_pretty(metadata) {
                                        Ok(json) => {
                                            ui.output_mut(|o| {
                                                o.copied_text = json;
                                            });
                                            ui.label("✓ Vault metadata copied to clipboard!");
                                        }
                                        Err(e) => {
                                            ui.label(format!("Failed to export: {}", crate::utils::sanitize_error_for_ui(&e)));
                                        }
                                    }
                                } else {
                                    ui.label("Vault metadata not found");
                                }
                            }
                            Err(e) => {
                                ui.label(format!("Failed to list vaults: {}", crate::utils::sanitize_error_for_ui(&e)));
                            }
                        }
                    }
                }
            });
        } else {
            ui.label("Status: Not Loaded");
            ui.add_space(10.0);
            if ui.button("Create Vault").clicked() {
                navigation.navigate_to(crate::state::View::VaultCreation);
            }
            if ui.button("Load Vault").clicked() {
                navigation.navigate_to(crate::state::View::VaultSelection);
            }
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Preferences section
        ui.label("Preferences:");
        ui.add_space(5.0);

        // Currency (placeholder)
        ui.horizontal(|ui| {
            ui.label("Currency:");
            ui.label("USD (not yet configurable)");
        });

        ui.add_space(10.0);

        // Appearance (placeholder)
        ui.horizontal(|ui| {
            ui.label("Theme:");
            ui.label("System (not yet configurable)");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Actions
        ui.label("Actions:");
        ui.add_space(5.0);

        if ui.button("Refresh Vault Data").clicked() {
            if let Some(ref mut handler) = app_state.async_handler {
                handler.fetch_balance();
                handler.fetch_address();
            }
        }

        ui.add_space(10.0);

        // Recovery actions
        if app_state.is_vault_loaded() {
            ui.separator();
            ui.add_space(10.0);
            ui.label("Recovery & Maintenance:");
            ui.horizontal(|ui| {
                if ui.button("Recovery").clicked() {
                    navigation.navigate_to(crate::state::View::Recovery);
                }
                if ui.button("UTXO Refresh").clicked() {
                    navigation.navigate_to(crate::state::View::UtxoRefresh);
                }
            });
        }

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Full Settings").clicked() {
                navigation.navigate_to(crate::state::View::Settings);
            }

            if ui.button("Notifications").clicked() {
                navigation.navigate_to(crate::state::View::NotificationCenter);
            }

            if ui.button("Address Book").clicked() {
                navigation.navigate_to(crate::state::View::AddressBook);
            }

            if ui.button("Help & Support").clicked() {
                navigation.navigate_to(crate::state::View::HelpAndSupport);
            }
        });
    });
}
