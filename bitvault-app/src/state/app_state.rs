//! App state management
//!
//! Manages the application's global state, including:
//! - Current vault/wallet
//! - Network selection
//! - UI state (current view, modals, etc.)

use crate::services::key_service::KeyService;
use crate::services::notification_service::NotificationService;
use crate::settings::{AppTheme, Currency, SettingsManager};
use crate::state::async_commands::AsyncCommandHandler;
use crate::state::vault_data::{SharedVaultData, VaultData};
use bdk::bitcoin::Network;
use bdk::keys::bip39::Mnemonic;
#[cfg(feature = "native")]
use bitvault_common::convenience::ConvenienceService;
use bitvault_common::wallet::VaultService;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    /// Key service for secure storage (mnemonics, PINs, etc.)
    pub key_service: KeyService,
    /// Cached mnemonic for current vault (for signing operations)
    pub cached_mnemonic: Option<Mnemonic>,
    /// Telegram registration link (when received from async handler)
    pub telegram_registration_link: Option<String>,
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
            key_service: KeyService::new(),
            cached_mnemonic: None,
            telegram_registration_link: None,
        })
    }

    /// Create a new app state without settings manager (fallback for initialization failures)
    /// This creates a minimal state with default values, but some features may be unavailable
    pub fn new_without_settings(network: Network) -> Result<Self, String> {
        // Try to create settings manager, but use defaults if it fails
        let settings_manager = match SettingsManager::new() {
            Ok(sm) => sm,
            Err(e) => {
                eprintln!(
                    "Warning: Could not create settings manager in fallback: {}",
                    e
                );
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
            key_service: KeyService::new(),
            cached_mnemonic: None,
            telegram_registration_link: None,
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
        // Refresh mnemonic if not cached (before borrowing self)
        if self.cached_mnemonic.is_none() {
            self.refresh_mnemonic();
        }

        // Get mnemonic from cache (before borrowing self)
        let mnemonic_ref = self.cached_mnemonic.as_ref();

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

            handler.process_pending(vs, rt, convenience_service, mnemonic_ref);
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

        // Refresh mnemonic for the initialized vault
        self.refresh_mnemonic();

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

        // Refresh mnemonic for the loaded vault
        self.refresh_mnemonic();

        // Restart async processor with new vault service
        self.on_vault_loaded();

        Ok(())
    }

    /// List all available vaults
    pub fn list_vaults() -> Result<Vec<bitvault_common::wallet::VaultMetadata>, String> {
        VaultService::<bdk::database::SqliteDatabase>::list_vaults()
    }

    /// Try to load the active vault for the current network.
    /// If active vault fails (DB missing, load error), tries other vaults for the network
    /// and sets the first working one as active (load working vault when current fails).
    /// Returns true if vault was loaded, false if no vaults or all loads failed.
    /// Requires runtime to be set.
    pub fn try_load_active_vault(&mut self) -> Result<bool, String> {
        let network_str = match self.network {
            Network::Bitcoin => "mainnet",
            Network::Testnet => "testnet",
            Network::Signet => "signet",
            Network::Regtest => "regtest",
            _ => "mainnet",
        };

        let runtime = match self.runtime.as_ref() {
            Some(rt) => rt,
            None => return Ok(false),
        };

        let all_vaults = Self::list_vaults()?;
        let vaults_for_network: Vec<_> = all_vaults
            .into_iter()
            .filter(|v| v.network == network_str && std::path::Path::new(&v.database_path).exists())
            .filter(|v| v.validate().is_ok())
            .collect();

        if vaults_for_network.is_empty() {
            return Ok(false);
        }

        // Order: active vault first, then others
        let mut to_try = vaults_for_network.clone();
        if let Ok(Some(active_addr)) = self.settings_manager.get_active_vault(network_str) {
            if let Some(pos) = to_try.iter().position(|v| v.address == active_addr) {
                let active = to_try.remove(pos);
                to_try.insert(0, active);
            }
        }

        for metadata in to_try {
            let metadata_clone = metadata.clone();
            let result = runtime
                .block_on(async { VaultService::load_vault_from_metadata(&metadata_clone).await });

            if let Ok(vault_service) = result {
                self.vault_service = Some(Arc::new(RwLock::new(vault_service)));
                self.has_vault = true;
                self.refresh_mnemonic();
                self.on_vault_loaded();
                // Set as active for next time (including when we fell back from failed active)
                let _ = self
                    .settings_manager
                    .set_active_vault(network_str, &metadata.address);
                return Ok(true);
            }
            log::warn!("Failed to load vault {}: trying next", metadata.address);
        }

        // All vaults failed - clear active if it was set (it's broken)
        let _ = self.settings_manager.clear_active_vault(network_str);
        Ok(false)
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

        // Set as active vault for this network before we move vault_service
        let vault_address = vault_service.get_address().map_err(|e| {
            bitvault_common::BitVaultError::Config(format!("Failed to get vault address: {}", e))
        })?;
        let network_str = match self.network {
            Network::Bitcoin => "mainnet",
            Network::Testnet => "testnet",
            Network::Signet => "signet",
            Network::Regtest => "regtest",
            _ => "mainnet",
        };
        let _ = self
            .settings_manager
            .set_active_vault(network_str, &vault_address);

        self.vault_service = Some(Arc::new(RwLock::new(vault_service)));
        self.has_vault = true;

        // Refresh mnemonic for the initialized vault
        self.refresh_mnemonic();

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

    /// Refresh cached mnemonic from KeyService
    pub fn refresh_mnemonic(&mut self) {
        if let Some(metadata) = self.get_current_vault_metadata() {
            let network_str = &metadata.network; // "mainnet", "testnet", etc.
            let vault_id = &metadata.address; // Vault address is used as vault_id

            if let Ok(backup_info) = self.key_service.get_backup_info(vault_id, network_str) {
                self.cached_mnemonic = backup_info.mnemonic.parse().ok();
            } else {
                self.cached_mnemonic = None;
            }
        } else {
            self.cached_mnemonic = None;
        }
    }

    /// Unload current vault (for switching)
    pub fn unload_vault(&mut self) {
        self.vault_service = None;
        self.has_vault = false;
        // Clear cached vault data
        if let Ok(mut data) = self.vault_data.lock() {
            *data = crate::state::vault_data::VaultData::new();
        }
        // Clear cached mnemonic
        self.cached_mnemonic = None;
        // Clear Telegram registration link
        self.telegram_registration_link = None;
    }

    /// Update network and reload vault for the new network
    /// If a vault exists for the new network, it will be loaded automatically
    /// Uses active vault first, then falls back to other vaults if load fails
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

        // Try to load a vault for the new network
        if let Some(ref runtime) = self.runtime {
            let all_vaults = Self::list_vaults()?;
            let vaults_for_network: Vec<_> = all_vaults
                .iter()
                .filter(|v| {
                    v.network == network_str && std::path::Path::new(&v.database_path).exists()
                })
                .filter(|v| v.validate().is_ok())
                .cloned()
                .collect();

            // Try active vault first, then others
            let mut to_try: Vec<_> = vaults_for_network.clone();
            if let Ok(Some(active_addr)) = self.settings_manager.get_active_vault(network_str) {
                // Put active vault first
                if let Some(pos) = to_try.iter().position(|v| v.address == active_addr) {
                    let active = to_try.remove(pos);
                    to_try.insert(0, active);
                }
            }

            for metadata in to_try {
                let metadata_clone = metadata.clone();
                let result = runtime.block_on(async {
                    VaultService::load_vault_from_metadata(&metadata_clone).await
                });

                if let Ok(vault_service) = result {
                    self.vault_service = Some(Arc::new(RwLock::new(vault_service)));
                    self.has_vault = true;
                    self.refresh_mnemonic();
                    self.on_vault_loaded();
                    // Set as active for next time
                    let _ = self
                        .settings_manager
                        .set_active_vault(network_str, &metadata.address);
                    return Ok(());
                }
            }
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
