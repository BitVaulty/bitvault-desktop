//! Async command handling for egui
//!
//! Simple pattern: queue commands and process them in the UI thread
//! using block_on (acceptable for quick operations like balance/address)

use tokio::sync::mpsc;

/// Commands that can be sent to async tasks
#[derive(Debug, Clone)]
pub enum AsyncCommand {
    FetchBalance,
    FetchAddress,
    RequestTelegramRegistration,
}

/// Results from async tasks
#[derive(Debug)]
pub enum AsyncResult {
    Balance { confirmed: u64, available: u64 },
    Address(String),
    TelegramRegistrationLink(String),
    Error(String),
}

/// Async command handler
///
/// For egui's immediate mode with non-Send types (MemoryDatabase),
/// we process commands directly using block_on in the UI thread.
/// This is acceptable for quick operations.
pub struct AsyncCommandHandler {
    pending_commands: Vec<AsyncCommand>,
    result_rx: mpsc::UnboundedReceiver<AsyncResult>,
    result_tx: mpsc::UnboundedSender<AsyncResult>,
}

impl AsyncCommandHandler {
    /// Create a new command handler
    pub fn new() -> Self {
        let (result_tx, result_rx) = mpsc::unbounded_channel();

        Self {
            pending_commands: Vec::new(),
            result_rx,
            result_tx,
        }
    }

    /// Queue a command to fetch balance
    pub fn fetch_balance(&mut self) {
        self.pending_commands.push(AsyncCommand::FetchBalance);
    }

    /// Queue a command to fetch address
    pub fn fetch_address(&mut self) {
        self.pending_commands.push(AsyncCommand::FetchAddress);
    }

    /// Queue a command to request Telegram registration
    pub fn request_telegram_registration(&mut self) {
        self.pending_commands.push(AsyncCommand::RequestTelegramRegistration);
    }

    /// Process pending commands (call from UI update loop)
    /// Uses block_on to handle non-Send types
    /// Note: This blocks the UI thread briefly, but operations are fast
    pub fn process_pending(
        &mut self,
        vault_service: &std::sync::Arc<tokio::sync::RwLock<bitvault_common::wallet::VaultService>>,
        runtime: &tokio::runtime::Runtime,
    ) {
        if self.pending_commands.is_empty() {
            return;
        }

        let commands = std::mem::take(&mut self.pending_commands);
        let result_tx = self.result_tx.clone();

        for cmd in commands {
            let vs: std::sync::Arc<tokio::sync::RwLock<bitvault_common::wallet::VaultService>> =
                vault_service.clone();
            let tx = result_tx.clone();

            match cmd {
                AsyncCommand::FetchBalance => {
                    // Use block_on directly (acceptable for quick operations)
                    let result: std::result::Result<(u64, u64), bitvault_common::BitVaultError> =
                        runtime.block_on(async {
                            let guard = vs.read().await;
                            guard.get_balance().await
                        });
                    match result {
                        Ok((confirmed, available)) => {
                            let _ = tx.send(AsyncResult::Balance {
                                confirmed,
                                available,
                            });
                        }
                        Err(e) => {
                            // BitVaultError now provides user-friendly messages
                            let _ = tx.send(AsyncResult::Error(format!(
                                "Failed to fetch balance: {}",
                                e
                            )));
                        }
                    }
                }
                AsyncCommand::FetchAddress => {
                    let result: std::result::Result<String, bitvault_common::BitVaultError> =
                        runtime.block_on(async {
                            let guard = vs.read().await;
                            guard.get_new_address().await
                        });
                    match result {
                        Ok(addr) => {
                            let _ = tx.send(AsyncResult::Address(addr));
                        }
                        Err(e) => {
                            // BitVaultError now provides user-friendly messages
                            let _ = tx.send(AsyncResult::Error(format!(
                                "Failed to fetch address: {}",
                                e
                            )));
                        }
                    }
                }
                AsyncCommand::RequestTelegramRegistration => {
                    // Request Telegram registration link from ConvenienceService
                    // This requires access to ConvenienceService, which is in AppState
                    // For now, we'll need to pass convenience_service to process_pending
                    // or handle this differently
                    // TODO: Add convenience_service parameter or handle via AppState
                    let _ = tx.send(AsyncResult::Error(
                        "Telegram registration not yet implemented in async handler".to_string(),
                    ));
                }
            }
        }
    }

    /// Try to receive a result (non-blocking)
    pub fn try_recv_result(&mut self) -> Option<AsyncResult> {
        self.result_rx.try_recv().ok()
    }
}
