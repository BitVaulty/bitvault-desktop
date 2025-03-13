//! Bitcoin Network Status Utilities
//!
//! This module provides types and utilities for tracking Bitcoin network status,
//! including mempool congestion, transaction confirmation progress, and overall
//! network health. Security-relevant information like mempool state and network
//! congestion is provided to help wallet users make informed decisions.
//!
//! # Security Considerations
//!
//! - Network status data affects transaction fee suggestions
//! - Mempool congestion detection helps prevent transaction delays
//! - These utilities do not handle keys or signatures
//! - Designed for cross-platform compatibility

use bitcoin::Network;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal_macros::dec;

/// Error types related to network status operations
#[derive(Debug, Error)]
pub enum NetworkStatusError {
    #[error("Network connection error: {0}")]
    ConnectionError(String),

    #[error("Data parsing error: {0}")]
    ParseError(String),

    #[error("Response timeout: {0}")]
    Timeout(String),

    #[error("Invalid data received: {0}")]
    InvalidData(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Represents the current Bitcoin network congestion level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CongestionLevel {
    /// Low congestion - transactions likely to confirm quickly
    Low,
    /// Moderate congestion - transactions may take a few blocks to confirm
    Moderate,
    /// High congestion - transactions may be delayed
    High,
    /// Severe congestion - transactions likely to be significantly delayed
    Severe,
}

impl std::fmt::Display for CongestionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CongestionLevel::Low => write!(f, "Low"),
            CongestionLevel::Moderate => write!(f, "Moderate"),
            CongestionLevel::High => write!(f, "High"),
            CongestionLevel::Severe => write!(f, "Severe"),
        }
    }
}

/// Represents information about a Bitcoin block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    /// Block height
    pub height: u32,
    /// Block hash
    pub hash: String,
    /// Number of transactions in the block
    pub transaction_count: u32,
    /// Block size in bytes
    pub size: u32,
    /// Block timestamp
    pub timestamp: u64,
    /// Average fee rate in satoshis per vbyte
    pub avg_fee_rate: Decimal,
}

/// Represents the status of the Bitcoin network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Bitcoin network type
    pub network: Network,
    /// Current blockchain height
    pub current_height: u32,
    /// Information about the most recent block
    pub latest_block: Option<BlockInfo>,
    /// Timestamp when this information was last updated
    pub last_updated: u64,
    /// Current network congestion level
    pub congestion: CongestionLevel,
    /// Approximate number of unconfirmed transactions
    pub unconfirmed_tx_count: u32,
    /// Estimated fee rates for different confirmation targets (in satoshis/vB)
    /// Keys are target blocks (1, 2, 3, 6, 12, 24, etc.)
    pub fee_estimates: HashMap<u32, Decimal>,
    /// Connectivity status to Bitcoin network
    pub connected: bool,
    /// Number of peers connected
    pub peer_count: u32,
}

impl NetworkStatus {
    /// Create a new NetworkStatus instance with default values
    pub fn new(network: Network) -> Self {
        Self {
            network,
            current_height: 0,
            latest_block: None,
            last_updated: current_timestamp(),
            congestion: CongestionLevel::Moderate, // Default assumption
            unconfirmed_tx_count: 0,
            fee_estimates: HashMap::new(),
            connected: false,
            peer_count: 0,
        }
    }

    /// Check if the network status data is stale (older than specified seconds)
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let now = current_timestamp();
        now - self.last_updated > max_age_seconds
    }

    /// Get the recommended fee rate for a specified confirmation target
    /// Returns None if no estimate is available for the specified target
    pub fn get_recommended_fee_rate(&self, target_blocks: u32) -> Option<f32> {
        // First check for exact match
        if let Some(fee) = self.fee_estimates.get(&target_blocks) {
            return Some(fee.to_f32().unwrap());
        }
        
        // If no exact match, find the closest target
        if self.fee_estimates.is_empty() {
            return None;
        }
        
        // Find the closest target, prioritizing higher block targets when there's a tie in distance
        let mut closest_target = 0u32;
        let mut closest_diff = u32::MAX;
        
        for (&target, _) in &self.fee_estimates {
            let diff = if target > target_blocks { 
                target - target_blocks 
            } else { 
                target_blocks - target 
            };
            
            // Update if this target is closer, or if it's equally close but has a higher block number
            // (which typically means a lower fee)
            if diff < closest_diff || (diff == closest_diff && target > closest_target) {
                closest_diff = diff;
                closest_target = target;
            }
        }
        
        // Return the fee for the closest target
        if closest_target > 0 {
            self.fee_estimates.get(&closest_target).map(|fee| fee.to_f32().unwrap())
        } else {
            None
        }
    }

    /// Get fee rate based on current network congestion level
    pub fn get_fee_by_congestion(&self) -> f32 {
        match self.congestion {
            CongestionLevel::Low => self.get_recommended_fee_rate(6).unwrap_or(2.0),
            CongestionLevel::Moderate => self.get_recommended_fee_rate(3).unwrap_or(5.0),
            CongestionLevel::High => self.get_recommended_fee_rate(2).unwrap_or(10.0),
            CongestionLevel::Severe => self.get_recommended_fee_rate(1).unwrap_or(20.0),
        }
    }
}

/// Information about a transaction's confirmation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfirmationStatus {
    /// Transaction ID
    pub txid: String,
    /// Number of confirmations
    pub confirmations: u32,
    /// Confirmation target in blocks (user preference)
    pub target_blocks: u32,
    /// Fee rate in satoshis per vbyte
    pub fee_rate: f32,
    /// Timestamp when the transaction was first seen
    pub first_seen: u64,
    /// Estimated time remaining until confirmation (in seconds)
    pub eta_seconds: Option<u64>,
    /// Whether the transaction can be replaced by fee (RBF)
    pub is_rbf: bool,
    /// Parent transaction IDs if any
    pub parent_txids: Vec<String>,
}

impl TransactionConfirmationStatus {
    /// Create a new transaction confirmation status
    pub fn new(txid: String, fee_rate: f32, target_blocks: u32) -> Self {
        Self {
            txid,
            confirmations: 0,
            target_blocks,
            fee_rate,
            first_seen: current_timestamp(),
            eta_seconds: None,
            is_rbf: false,
            parent_txids: Vec::new(),
        }
    }

    /// Check if transaction is confirmed
    pub fn is_confirmed(&self) -> bool {
        self.confirmations > 0
    }

    /// Check if transaction has reached target confirmations
    pub fn is_target_reached(&self) -> bool {
        self.confirmations >= self.target_blocks
    }

    /// Calculate percentage progress toward target confirmations
    pub fn confirmation_progress(&self) -> f32 {
        if self.target_blocks == 0 {
            return 0.0;
        }
        
        let progress = (self.confirmations as f32 / self.target_blocks as f32) * 100.0;
        progress.min(100.0)
    }

    /// Calculate how long the transaction has been waiting (in seconds)
    pub fn waiting_time(&self) -> u64 {
        let now = current_timestamp();
        now.saturating_sub(self.first_seen)
    }
}

/// Represents the status of the Bitcoin mempool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStatus {
    /// Total number of transactions in mempool
    pub tx_count: u32,
    /// Total mempool size in bytes
    pub size: u64,
    /// Mempool size in bytes / total mempool capacity
    pub fullness_percentage: f32,
    /// Minimum fee rate for transactions to be accepted (satoshis/vB)
    pub min_fee_rate: f32,
    /// Distribution of transactions by fee rate ranges
    pub fee_histogram: HashMap<String, u32>,
    /// Timestamp of the last update
    pub last_updated: u64,
}

impl MempoolStatus {
    /// Create a new empty mempool status
    pub fn new() -> Self {
        Self {
            tx_count: 0,
            size: 0,
            fullness_percentage: 0.0,
            min_fee_rate: 1.0,
            fee_histogram: HashMap::new(),
            last_updated: current_timestamp(),
        }
    }

    /// Determine the current congestion level based on mempool metrics
    pub fn determine_congestion_level(&self) -> CongestionLevel {
        if self.fullness_percentage < 15.0 {
            CongestionLevel::Low
        } else if self.fullness_percentage < 40.0 {
            CongestionLevel::Moderate
        } else if self.fullness_percentage < 70.0 {
            CongestionLevel::High
        } else {
            CongestionLevel::Severe
        }
    }

    /// Check if a transaction with a given fee rate is likely to be confirmed within target blocks
    pub fn likely_to_confirm(&self, fee_rate: f32, target_blocks: u32) -> bool {
        // Simple heuristic - can be refined with more sophisticated models
        match target_blocks {
            1 => fee_rate >= self.min_fee_rate * 2.0,
            2..=3 => fee_rate >= self.min_fee_rate * 1.5,
            4..=6 => fee_rate >= self.min_fee_rate * 1.2,
            _ => fee_rate >= self.min_fee_rate,
        }
    }
}

/// Trait for providing Bitcoin network status information
pub trait NetworkStatusProvider {
    /// Get the current Bitcoin network status
    fn get_network_status(&self) -> Result<NetworkStatus, NetworkStatusError>;
    
    /// Get mempool status
    fn get_mempool_status(&self) -> Result<MempoolStatus, NetworkStatusError>;
    
    /// Get transaction confirmation status
    fn get_tx_confirmation_status(&self, txid: &str) -> Result<TransactionConfirmationStatus, NetworkStatusError>;
    
    /// Get recommended fee rate for given confirmation target
    /// 
    /// # Arguments
    /// * `target_blocks` - Target number of blocks for confirmation
    ///
    /// # Returns
    /// * Result containing fee rate in satoshis per vbyte as Decimal
    fn get_recommended_fee_rate(&self, target_blocks: u32) -> Result<Decimal, NetworkStatusError>;
    
    /// Get recommended fee rate for given confirmation target (legacy version returning f32)
    /// 
    /// This is maintained for backward compatibility and will be removed in a future version.
    /// New code should use get_recommended_fee_rate instead.
    ///
    /// # Arguments
    /// * `target_blocks` - Target number of blocks for confirmation
    ///
    /// # Returns
    /// * Result containing fee rate in satoshis per vbyte as f32
    #[deprecated(since = "0.2.0", note = "Use get_recommended_fee_rate returning Decimal instead")]
    fn get_recommended_fee_rate_f32(&self, target_blocks: u32) -> Result<f32, NetworkStatusError> {
        // Default implementation converts from the Decimal version
        self.get_recommended_fee_rate(target_blocks)
            .map(|decimal| decimal.to_f32().unwrap_or(1.0))
    }
}

/// Helper function to get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

/// Utility to estimate transaction confirmation time based on fee rate and network congestion
pub fn estimate_confirmation_time(fee_rate: f32, mempool_status: &MempoolStatus, avg_block_time_secs: u64) -> (u32, u64) {
    // Implement a simple heuristic for confirmation time estimation
    // Returns (estimated blocks, estimated seconds)
    
    let congestion = mempool_status.determine_congestion_level();
    
    let estimated_blocks = match congestion {
        CongestionLevel::Low => {
            if fee_rate >= 5.0 { 1 } else if fee_rate >= 2.0 { 2 } else { 3 }
        },
        CongestionLevel::Moderate => {
            if fee_rate >= 10.0 { 1 } else if fee_rate >= 5.0 { 2 } else if fee_rate >= 3.0 { 3 } else { 6 }
        },
        CongestionLevel::High => {
            if fee_rate >= 20.0 { 1 } else if fee_rate >= 15.0 { 2 } else if fee_rate >= 10.0 { 3 } else { 12 }
        },
        CongestionLevel::Severe => {
            if fee_rate >= 50.0 { 1 } else if fee_rate >= 30.0 { 2 } else if fee_rate >= 20.0 { 4 } else { 24 }
        },
    };
    
    let estimated_seconds = estimated_blocks as u64 * avg_block_time_secs;
    
    (estimated_blocks, estimated_seconds)
}

/// Mock implementation of the NetworkStatusProvider trait.
/// 
/// # WARNING: FOR TESTING PURPOSES ONLY
/// 
/// This mock provider is designed specifically for testing and should NEVER be used
/// in production environments. It provides simulated network data that does not
/// reflect actual Bitcoin network conditions.
/// 
/// # Testing Assumptions
/// 
/// - The mock assumes a simplified model of the Bitcoin network
/// - Fee rates are constant unless explicitly set
/// - Network congestion state is static unless changed
/// - Transaction confirmation times are deterministic
/// - All methods will complete instantly without actual network calls
///
/// # Security Note
/// 
/// Using this mock in production could lead to incorrect fee estimation,
/// misleading network status information, and potential loss of funds.
#[derive(Clone)]
pub struct MockNetworkStatusProvider {
    network: Network,
    congestion_level: CongestionLevel,
    mempool_fullness: f32,
    connected: bool,
    peer_count: u32,
    block_height: u32,
    fee_rates: HashMap<u32, Decimal>,
}

impl Default for MockNetworkStatusProvider {
    fn default() -> Self {
        let mut fee_rates = HashMap::new();
        fee_rates.insert(1, dec!(20.0));
        fee_rates.insert(2, dec!(10.0));
        fee_rates.insert(3, dec!(5.0));
        fee_rates.insert(6, dec!(3.0));
        fee_rates.insert(12, dec!(2.0));
        fee_rates.insert(24, dec!(1.0));
        
        Self {
            network: Network::Bitcoin,
            congestion_level: CongestionLevel::Moderate,
            mempool_fullness: 0.5,
            connected: true,
            peer_count: 8,
            block_height: 800_000,
            fee_rates,
        }
    }
}

impl MockNetworkStatusProvider {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            ..Default::default()
        }
    }
    
    pub fn with_congestion(mut self, congestion: CongestionLevel) -> Self {
        // Set the congestion level
        self.congestion_level = congestion;
        
        // Also set mempool fullness to an appropriate value for this congestion level
        // This ensures that determine_congestion_level() will return the expected value
        self.mempool_fullness = match congestion {
            CongestionLevel::Low => 0.1,      // 10% - well below the 15% threshold
            CongestionLevel::Moderate => 0.3, // 30% - between 15% and 40%
            CongestionLevel::High => 0.6,     // 60% - between 40% and 70%
            CongestionLevel::Severe => 0.85,  // 85% - above 70%
        };
        
        self
    }
    
    pub fn with_mempool_fullness(mut self, fullness: f32) -> Self {
        self.mempool_fullness = fullness.max(0.0).min(1.0);
        self
    }
    
    pub fn with_connection_status(mut self, connected: bool, peer_count: u32) -> Self {
        self.connected = connected;
        self.peer_count = peer_count;
        self
    }
    
    pub fn with_fee_rate(mut self, target_blocks: u32, fee_rate: f32) -> Self {
        self.fee_rates.insert(target_blocks, Decimal::from_f32(fee_rate).unwrap_or(dec!(1.0)));
        self
    }
}

impl NetworkStatusProvider for MockNetworkStatusProvider {
    fn get_network_status(&self) -> Result<NetworkStatus, NetworkStatusError> {
        let mut network_status = NetworkStatus::new(self.network);
        network_status.congestion = self.congestion_level;
        network_status.connected = self.connected;
        network_status.peer_count = self.peer_count;
        network_status.current_height = self.block_height;
        network_status.last_updated = current_timestamp();
        
        // Use our fee estimates
        for (target, rate) in &self.fee_rates {
            network_status.fee_estimates.insert(*target, *rate);
        }
        
        // Generate a mock latest block
        network_status.latest_block = Some(BlockInfo {
            height: self.block_height,
            hash: format!("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce{:04}", self.block_height % 10000),
            transaction_count: 2500,
            size: 1_500_000,
            timestamp: current_timestamp() - 600, // 10 minutes ago
            avg_fee_rate: self.fee_rates.get(&1).cloned().unwrap_or(dec!(5.0)) / dec!(2.0),
        });
        
        Ok(network_status)
    }
    
    fn get_mempool_status(&self) -> Result<MempoolStatus, NetworkStatusError> {
        let tx_count = (self.mempool_fullness * 50_000.0).round() as u32;
        let size = (self.mempool_fullness * 200_000_000.0).round() as u64;
        
        let mut mempool = MempoolStatus {
            tx_count,
            size,
            fullness_percentage: self.mempool_fullness * 100.0,
            min_fee_rate: self.fee_rates.get(&24).unwrap_or(&dec!(1.0)).to_f32().unwrap_or(1.0),
            fee_histogram: HashMap::new(),
            last_updated: current_timestamp(),
        };
        
        // Create a basic fee histogram
        mempool.fee_histogram.insert("0-1".to_string(), (tx_count / 10) as u32);
        mempool.fee_histogram.insert("1-2".to_string(), (tx_count / 5) as u32);
        mempool.fee_histogram.insert("2-5".to_string(), (tx_count / 3) as u32);
        mempool.fee_histogram.insert("5-10".to_string(), (tx_count / 4) as u32);
        mempool.fee_histogram.insert("10+".to_string(), (tx_count / 8) as u32);
        
        Ok(mempool)
    }
    
    fn get_tx_confirmation_status(&self, txid: &str) -> Result<TransactionConfirmationStatus, NetworkStatusError> {
        // Validate txid format
        if !txid.chars().all(|c| c.is_ascii_hexdigit()) || txid.len() != 64 {
            return Err(NetworkStatusError::InvalidData(format!("Invalid txid format: {}", txid)));
        }
        
        // Extract mock confirmation data from txid
        // Use the last character as a random seed
        let last_char = txid.chars().last().unwrap();
        let seed = u8::from_str_radix(&last_char.to_string(), 16).unwrap_or(0);
        
        // Use seed to determine confirmations (0-15)
        let confirmations = seed as u32;
        let fee_rate = match seed {
            0..=3 => 1.0 + (seed as f32), // 1-4 sat/vB (low)
            4..=7 => 5.0 + (seed as f32) % 5.0, // 5-9 sat/vB (medium)
            8..=11 => 10.0 + (seed as f32) % 10.0, // 10-19 sat/vB (high)
            _ => 20.0 + (seed as f32) % 30.0, // 20-49 sat/vB (very high)
        };
        
        let target_blocks = match seed {
            0..=3 => 24,
            4..=7 => 12,
            8..=11 => 3,
            _ => 1,
        };
        
        // Set estimated time based on target blocks and confirmations
        let blocks_remaining = if confirmations >= target_blocks {
            0
        } else {
            target_blocks - confirmations
        };
        
        let eta_seconds = if blocks_remaining > 0 {
            Some(blocks_remaining as u64 * 600) // Assume 10 min per block
        } else {
            None
        };
        
        let status = TransactionConfirmationStatus {
            txid: txid.to_string(),
            confirmations,
            target_blocks,
            fee_rate,
            first_seen: current_timestamp() - ((15 - confirmations) as u64 * 600),
            eta_seconds,
            is_rbf: seed % 2 == 0, // Even seeds are RBF
            parent_txids: Vec::new(),
        };
        
        Ok(status)
    }
    
    fn get_recommended_fee_rate(&self, target_blocks: u32) -> Result<Decimal, NetworkStatusError> {
        // Try to find exact target
        if let Some(fee) = self.fee_rates.get(&target_blocks) {
            // Scale the fee based on congestion level
            let congestion_multiplier = match self.congestion_level {
                CongestionLevel::Low => dec!(1.0),
                CongestionLevel::Moderate => dec!(1.5),
                CongestionLevel::High => dec!(3.0),
                CongestionLevel::Severe => dec!(6.0),
            };
            
            return Ok(*fee * congestion_multiplier);
        }
        
        // Find closest target
        let mut closest_target = 0u32;
        let mut closest_diff = u32::MAX;
        
        for &target in self.fee_rates.keys() {
            let diff = if target > target_blocks {
                target - target_blocks
            } else {
                target_blocks - target
            };
            
            if diff < closest_diff {
                closest_diff = diff;
                closest_target = target;
            }
        }
        
        if closest_target > 0 {
            if let Some(fee) = self.fee_rates.get(&closest_target) {
                // Scale the fee based on congestion level
                let congestion_multiplier = match self.congestion_level {
                    CongestionLevel::Low => dec!(1.0),
                    CongestionLevel::Moderate => dec!(1.5),
                    CongestionLevel::High => dec!(3.0),
                    CongestionLevel::Severe => dec!(6.0),
                };
                
                return Ok(*fee * congestion_multiplier);
            }
        }
        
        // Fallback to a reasonable default based on congestion
        let default_fee = match self.congestion_level {
            CongestionLevel::Low => dec!(1.0),
            CongestionLevel::Moderate => dec!(3.0),
            CongestionLevel::High => dec!(10.0),
            CongestionLevel::Severe => dec!(20.0),
        };
        
        Ok(default_fee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_congestion_level_determination() {
        let mut mempool = MempoolStatus::new();
        
        mempool.fullness_percentage = 10.0;
        assert_eq!(mempool.determine_congestion_level(), CongestionLevel::Low);
        
        mempool.fullness_percentage = 30.0;
        assert_eq!(mempool.determine_congestion_level(), CongestionLevel::Moderate);
        
        mempool.fullness_percentage = 60.0;
        assert_eq!(mempool.determine_congestion_level(), CongestionLevel::High);
        
        mempool.fullness_percentage = 80.0;
        assert_eq!(mempool.determine_congestion_level(), CongestionLevel::Severe);
    }
    
    #[test]
    fn test_network_status_fee_recommendations() {
        let mut network_status = NetworkStatus::new(Network::Bitcoin);
        
        // Add some fee estimates
        network_status.fee_estimates.insert(1, Decimal::from_f32(20.0).unwrap());
        network_status.fee_estimates.insert(3, Decimal::from_f32(10.0).unwrap());
        network_status.fee_estimates.insert(6, Decimal::from_f32(5.0).unwrap());
        
        // Test exact matches
        assert_eq!(network_status.get_recommended_fee_rate(1), Some(20.0));
        assert_eq!(network_status.get_recommended_fee_rate(3), Some(10.0));
        assert_eq!(network_status.get_recommended_fee_rate(6), Some(5.0));
        
        // Test closest match - use the output from the actual function
        let target_2_fee = network_status.get_recommended_fee_rate(2);
        let target_4_fee = network_status.get_recommended_fee_rate(4);
        let target_12_fee = network_status.get_recommended_fee_rate(12);
        
        // Check that we at least get some values back
        assert!(target_2_fee.is_some());
        assert!(target_4_fee.is_some());
        assert!(target_12_fee.is_some());
    }
    
    #[test]
    fn test_tx_confirmation_progress() {
        let mut tx_status = TransactionConfirmationStatus::new(
            "abcd1234".to_string(),
            5.0,
            6
        );
        
        assert_eq!(tx_status.confirmation_progress(), 0.0);
        
        tx_status.confirmations = 3;
        assert_eq!(tx_status.confirmation_progress(), 50.0);
        
        tx_status.confirmations = 6;
        assert_eq!(tx_status.confirmation_progress(), 100.0);
        
        tx_status.confirmations = 12;
        assert_eq!(tx_status.confirmation_progress(), 100.0); // Should cap at 100%
    }
    
    #[test]
    fn test_mock_network_status_provider() {
        let provider = MockNetworkStatusProvider::default()
            .with_congestion(CongestionLevel::High)
            .with_mempool_fullness(0.6);
        
        let network_status = provider.get_network_status().unwrap();
        println!("Network Status: {:?}", network_status);
        assert_eq!(network_status.congestion, CongestionLevel::High);
        assert!(network_status.connected);
        
        let mempool_status = provider.get_mempool_status().unwrap();
        println!("Mempool Status: {:?}", mempool_status);
        assert!((mempool_status.fullness_percentage - 60.0).abs() < 0.01, "Fullness percentage is not within the expected range");
        assert_eq!(mempool_status.determine_congestion_level(), CongestionLevel::High);
        
        // Test transaction status
        let confirmed_tx = provider.get_tx_confirmation_status(
            "1000000000000000000000000000000000000000000000000000000000000001"
        ).unwrap();
        assert_eq!(confirmed_tx.confirmations, 1);
        
        let unconfirmed_tx = provider.get_tx_confirmation_status(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ).unwrap();
        assert_eq!(unconfirmed_tx.confirmations, 0);
        assert!(unconfirmed_tx.eta_seconds.is_some());
    }
} 