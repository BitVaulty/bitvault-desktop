//! VaultService - Uses BDK directly, no wrappers
//!
//! This service handles wallet operations using BDK.
//! We use BDK's API directly rather than wrapping it.

use bdk::Wallet;
use bdk::database::MemoryDatabase;
use bdk::bitcoin::Network;
use bdk::wallet::AddressIndex;
use std::sync::Arc;
use tokio::sync::RwLock;

/// VaultService manages wallet operations using BDK
pub struct VaultService {
    wallet: Option<Arc<RwLock<Wallet<MemoryDatabase>>>>,
    network: Network,
}

impl VaultService {
    /// Create a new VaultService
    pub fn new(network: Network) -> Self {
        Self {
            wallet: None,
            network,
        }
    }
    
    /// Initialize a wallet from a descriptor string
    /// This is the most basic operation - just get a wallet working
    pub async fn initialize_wallet(&mut self, descriptor: &str) -> Result<(), bdk::Error> {
        // Start with MemoryDatabase - we'll switch to SqliteDatabase later
        let database = MemoryDatabase::default();
        
        // Create wallet from descriptor
        let wallet = Wallet::new(
            descriptor,
            None::<&str>, // No change descriptor for now
            self.network,
            database,
        )?;
        
        self.wallet = Some(Arc::new(RwLock::new(wallet)));
        Ok(())
    }
    
    /// Get wallet balance
    /// Returns (confirmed, available) in satoshis
    pub async fn get_balance(&self) -> Result<(u64, u64), bdk::Error> {
        let wallet = self.wallet.as_ref()
            .ok_or_else(|| bdk::Error::Generic("Wallet not initialized".into()))?;
        
        let balance = wallet.read().await.get_balance()?;
        
        Ok((
            balance.confirmed,
            balance.get_spendable(),
        ))
    }
    
    /// Get a new receive address
    pub async fn get_address(&self) -> Result<String, bdk::Error> {
        let wallet = self.wallet.as_ref()
            .ok_or_else(|| bdk::Error::Generic("Wallet not initialized".into()))?;
        
        let address_info = wallet.read().await.get_address(AddressIndex::New)?;
        
        Ok(address_info.address.to_string())
    }
    
    /// Check if wallet is loaded
    pub fn is_loaded(&self) -> bool {
        self.wallet.is_some()
    }
}
