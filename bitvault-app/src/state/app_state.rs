//! App state management
//!
//! Manages the application's global state, including:
//! - Current vault/wallet
//! - Network selection
//! - UI state (current view, modals, etc.)

use crate::services::notification_service::NotificationService;
use crate::settings::{AppTheme, Currency, SettingsManager};
use crate::state::async_commands::AsyncCommandHandler;
use crate::state::vault_data::{SharedVaultData, VaultData};
use bdk::bitcoin::Network;
use bitvault_common::wallet::VaultService;
use std::sync::Arc;
use tokio::sync::RwLock;
#[cfg(feature = "native")]
use bitvault_common::convenience::ConvenienceService;

/// Application state
pub struct AppState {
    /// Current vault service (None if no vault loaded)
    pub vault_service: Option<Arc<RwLock<VaultService>>>,
    /// Current network
    pub network: Network,
    /// Whether a vault is currently loaded
    pub has_vault: bool,
    /// Cached vault data (balance, address, etc.) - shared for async updates
    pub vault_data: SharedVaultData,
    /// Tokio runtime for async operations (needed for block_on)
    pub runtime: Option<tokio::runtime::Runtime>,
    /// Async command handler for UI -> async communication
    pub async_handler: Option<AsyncCommandHandler>,
    /// Settings manager
    pub settings_manager: SettingsManager,
    /// Current currency
    pub currency: Currency,
    /// Current theme
    pub theme: AppTheme,
    /// Notification service for desktop notifications
    pub notification_service: Arc<NotificationService>,
    /// Convenience service for backend API calls
    #[cfg(feature = "native")]
    pub convenience_service: Option<ConvenienceService>,
}

impl AppState {
    /// Create a new app state
    pub fn new(network: Network) -> Result<Self, String> {
        let settings_manager = SettingsManager::new()?;
        let currency = settings_manager.get_currency().unwrap_or(Currency::USD);
        let theme = settings_manager.get_theme().unwrap_or(AppTheme::System);

        Ok(Self {
            vault_service: None,
            network,
            has_vault: false,
            vault_data: Arc::new(std::sync::Mutex::new(VaultData::new())),
            runtime: None,
            async_handler: None,
            settings_manager,
            currency,
            theme,
            notification_service: Arc::new(NotificationService::new()),
            #[cfg(feature = "native")]
            convenience_service: Some(ConvenienceService::new(None)),
        })
    }

    /// Create a new app state without settings manager (fallback for initialization failures)
    /// This creates a minimal state with default values, but some features may be unavailable
    pub fn new_without_settings(network: Network) -> Result<Self, String> {
        // Try to create settings manager, but use defaults if it fails
        let settings_manager = match SettingsManager::new() {
            Ok(sm) => sm,
            Err(e) => {
                eprintln!("Warning: Could not create settings manager in fallback: {}", e);
                return Err(format!("Cannot create app state even with fallback: {}", e));
            }
        };

        Ok(Self {
            vault_service: None,
            network,
            has_vault: false,
            vault_data: Arc::new(std::sync::Mutex::new(VaultData::new())),
            runtime: None,
            async_handler: None,
            settings_manager,
            currency: Currency::USD,
            theme: AppTheme::System,
            notification_service: Arc::new(NotificationService::new()),
            #[cfg(feature = "native")]
            convenience_service: Some(ConvenienceService::new(None)),
        })
    }

    /// Set the runtime for async operations
    pub fn set_runtime(&mut self, runtime: tokio::runtime::Runtime) {
        self.runtime = Some(runtime);

        // Initialize async command handler
        let handler = AsyncCommandHandler::new();
        self.async_handler = Some(handler);
    }

    /// Get a reference to the runtime (for block_on operations)
    pub fn get_runtime(&self) -> Option<&tokio::runtime::Runtime> {
        self.runtime.as_ref()
    }

    /// Restart async processor when vault is loaded
    pub fn on_vault_loaded(&mut self) {
        // Get new handler for the new vault
        let new_handler = AsyncCommandHandler::new();
        self.async_handler = Some(new_handler);
    }

    /// Process async commands and results (call this from UI update loop)
    /// Returns true if repaint should be requested
    pub fn process_async(&mut self, ctx: Option<&eframe::egui::Context>) -> bool {
        let mut needs_repaint = false;

        // Process pending commands
        if let (Some(ref mut handler), Some(vault_service), Some(runtime)) = (
            self.async_handler.as_mut(),
            self.vault_service.as_ref(),
            self.runtime.as_ref(),
        ) {
            // vault_service is &Option<Arc<...>> from as_ref(), but process_pending expects &Arc<...>
            // Since we matched Some, we can safely get the inner Arc reference
            // The pattern match guarantees it's Some, so unwrap is safe
            // vault_service is already &Option<Arc<...>>, so we just need to unwrap the Option
            let vs: &std::sync::Arc<tokio::sync::RwLock<bitvault_common::wallet::VaultService>> =
                vault_service;
            let rt: &tokio::runtime::Runtime = runtime;
            #[cfg(feature = "native")]
            let convenience_service = self.convenience_service.as_ref();
            #[cfg(not(feature = "native"))]
            let convenience_service = None::<&bitvault_common::convenience::ConvenienceService>;
            
            // Get mnemonic from KeyService if available
            // For now, pass None - mnemonic can be retrieved when needed
            let mnemonic: Option<&bdk::keys::bip39::Mnemonic> = None;
            
            handler.process_pending(vs, rt, convenience_service, mnemonic);
            needs_repaint = true;
        }

        // Process results
        if let Some(ref mut handler) = self.async_handler {
            while let Some(result) = handler.try_recv_result() {
                let vault_data = self.vault_data.clone();
                match result {
                    crate::state::async_commands::AsyncResult::Balance {
                        confirmed,
                        available,
                    } => {
                        if let Ok(mut data) = vault_data.lock() {
                            data.update_balance(confirmed, available);
                        }
                        needs_repaint = true;
                    }
                    crate::state::async_commands::AsyncResult::Address(addr) => {
                        if let Ok(mut data) = vault_data.lock() {
                            data.update_address(addr);
                        }
                        needs_repaint = true;
                    }
                    crate::state::async_commands::AsyncResult::Error(e) => {
                        // Log the error for debugging
                        log::error!("Async operation failed: {}", e);

                        // Update vault data to show error state
                        if let Ok(mut data) = vault_data.lock() {
                            data.set_error(Some(e));
                        }

                        needs_repaint = true;
                    }
                    crate::state::async_commands::AsyncResult::TelegramRegistrationLink(_link) => {
                        // Handle Telegram registration link - this should be processed by the UI
                        log::info!("Received Telegram registration link");
                        needs_repaint = true;
                    }
                }
            }
        }

        if needs_repaint {
            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        }

        needs_repaint
    }

    /// Initialize a vault service with a descriptor
    pub async fn initialize_vault(
        &mut self,
        descriptor: &str,
    ) -> Result<(), bitvault_common::BitVaultError> {
        let mut vault_service = VaultService::new(self.network);
        vault_service
            .initialize_wallet(descriptor, None, None)
            .await?;

        self.vault_service = Some(Arc::new(RwLock::new(vault_service)));
        self.has_vault = true;

        // Restart async processor with new vault service
        self.on_vault_loaded();

        Ok(())
    }

    /// Load a vault from metadata
    pub async fn load_vault_from_metadata(
        &mut self,
        metadata: &bitvault_common::wallet::VaultMetadata,
    ) -> Result<(), bitvault_common::BitVaultError> {
        let vault_service = VaultService::load_vault_from_metadata(metadata).await?;

        self.vault_service = Some(Arc::new(RwLock::new(vault_service)));
        self.has_vault = true;
        self.network = metadata.network_to_bdk();

        // Restart async processor with new vault service
        self.on_vault_loaded();

        Ok(())
    }

    /// List all available vaults
    pub fn list_vaults() -> Result<Vec<bitvault_common::wallet::VaultMetadata>, String> {
        VaultService::<bdk::database::SqliteDatabase>::list_vaults()
    }

    /// Initialize vault from an existing VaultService (e.g., after setup_vault)
    pub async fn initialize_vault_from_service(
        &mut self,
        vault_service: VaultService,
    ) -> Result<(), bitvault_common::BitVaultError> {
        // Check that the vault service has a wallet initialized
        if !vault_service.is_loaded() {
            return Err(bitvault_common::BitVaultError::Config(
                "VaultService wallet not initialized".to_string(),
            ));
        }

        self.vault_service = Some(Arc::new(RwLock::new(vault_service)));
        self.has_vault = true;

        // Restart async processor with new vault service
        self.on_vault_loaded();

        Ok(())
    }

    /// Get the current vault service (if loaded)
    pub fn get_vault_service(&self) -> Option<Arc<RwLock<VaultService>>> {
        self.vault_service.clone()
    }

    /// Check if a vault is loaded
    pub fn is_vault_loaded(&self) -> bool {
        self.has_vault
    }

    /// Get current vault metadata (if loaded)
    pub fn get_current_vault_metadata(&self) -> Option<bitvault_common::wallet::VaultMetadata> {
        // Only return metadata if vault is actually loaded (has vault_service)
        if !self.has_vault || self.vault_service.is_none() {
            return None;
        }

        // Get vault address from vault data
        if let Ok(vault_data) = self.vault_data.lock() {
            if let Some(ref address) = vault_data.receive_address {
                // Find vault metadata by address
                if let Ok(vaults) = Self::list_vaults() {
                    if let Some(metadata) = vaults.into_iter().find(|v| v.address == *address) {
                        // Verify the database file actually exists
                        if std::path::Path::new(&metadata.database_path).exists() {
                            return Some(metadata);
                        }
                    }
                }
            }
        }

        None
    }

    /// Unload current vault (for switching)
    pub fn unload_vault(&mut self) {
        self.vault_service = None;
        self.has_vault = false;
        // Clear cached vault data
        if let Ok(mut data) = self.vault_data.lock() {
            *data = crate::state::vault_data::VaultData::new();
        }
    }

    /// Update network and reload vault for the new network
    /// If a vault exists for the new network, it will be loaded automatically
    /// If no vault exists, the vault will be unloaded and user should see vault selection
    pub fn update_network(&mut self, new_network: Network) -> Result<(), String> {
        // Save network preference
        let network_str = match new_network {
            Network::Bitcoin => "mainnet",
            Network::Testnet => "testnet",
            Network::Signet => "signet",
            Network::Regtest => "regtest",
            _ => "mainnet",
        };
        self.settings_manager.set_network(network_str.to_string())?;

        // Update network
        self.network = new_network;

        // Unload current vault (it's for the old network)
        self.unload_vault();

        // Try to find and load a vault for the new network
        if let Some(ref runtime) = self.runtime {
            // List all vaults
            let all_vaults = Self::list_vaults()?;

            // Find first vault for the new network
            if let Some(vault_metadata) = all_vaults.iter().find(|v| v.network == network_str) {
                // Clone metadata to avoid borrowing issues
                let metadata_clone = vault_metadata.clone();

                // Load this vault
                let result = runtime.block_on(async {
                    // Create a temporary AppState-like operation
                    // We need to load the vault, but can't borrow self in async block
                    // So we'll do it synchronously by creating a new vault service
                    VaultService::load_vault_from_metadata(&metadata_clone).await
                });

                match result {
                    Ok(vault_service) => {
                        // Initialize vault from the loaded service
                        self.vault_service = Some(Arc::new(RwLock::new(vault_service)));
                        self.has_vault = true;
                        self.on_vault_loaded();
                    }
                    Err(e) => {
                        return Err(format!("Failed to load vault: {}", e));
                    }
                }
            }
            // If no vault found for this network, that's okay - user will see vault selection
        }

        Ok(())
    }
}

/// Process async commands in background task
#[allow(dead_code)]
async fn process_async_commands(
    vault_service: Arc<RwLock<VaultService>>,
    mut command_rx: tokio::sync::mpsc::UnboundedReceiver<
        crate::state::async_commands::AsyncCommand,
    >,
    result_tx: tokio::sync::mpsc::UnboundedSender<crate::state::async_commands::AsyncResult>,
) {
    while let Some(cmd) = command_rx.recv().await {
        match cmd {
            crate::state::async_commands::AsyncCommand::FetchBalance => {
                let result = {
                    let guard = vault_service.read().await;
                    guard.get_balance().await
                };
                match result {
                    Ok((confirmed, available)) => {
                        let _ =
                            result_tx.send(crate::state::async_commands::AsyncResult::Balance {
                                confirmed,
                                available,
                            });
                    }
                    Err(e) => {
                        let _ = result_tx.send(crate::state::async_commands::AsyncResult::Error(
                            format!("Failed to fetch balance: {}", e),
                        ));
                    }
                }
            }
            crate::state::async_commands::AsyncCommand::FetchAddress => {
                let result = {
                    let guard = vault_service.read().await;
                    guard.get_new_address().await
                };
                match result {
                    Ok(addr) => {
                        let _ = result_tx
                            .send(crate::state::async_commands::AsyncResult::Address(addr));
                    }
                    Err(e) => {
                        let _ = result_tx.send(crate::state::async_commands::AsyncResult::Error(
                            format!("Failed to fetch address: {}", e),
                        ));
                    }
                }
            }
            crate::state::async_commands::AsyncCommand::RequestTelegramRegistration => {
                // Telegram registration is handled elsewhere - this is a placeholder
                log::info!("Telegram registration requested");
            }
        }
    }
}
