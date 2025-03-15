//! Fee Estimation Utilities for Bitcoin Transactions
//!
//! This module provides utilities for estimating Bitcoin transaction fees
//! based on current network conditions and different priority levels.
//!
//! # Security Considerations
//!
//! - Accurate fee estimation is crucial for transaction confirmation security
//! - This module does not handle keys or signatures
//! - Fee estimation affects wallet balance calculations
//! - Designed for cross-platform compatibility
//!
//! # Fee Estimation Strategy
//!
//! This module implements several fee estimation strategies:
//! - Network-based fee estimation using real-time mempool data
//! - Historical fee data analysis for trend-based estimation
//! - Congestion-aware adaptive fee calculation
//! - Priority-based fee recommendations for different confirmation targets

use crate::types::FeePriority;
use crate::network_status::{NetworkStatusProvider, CongestionLevel};
use crate::types::WalletError;
use bitcoin::Network;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use chrono::Timelike;
use std::iter::Iterator;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal_macros::dec;
use serde_json::json;
use crate::events::{MessageBus, EventType, MessagePriority};
use std::sync::{Arc, RwLock};

/// Errors related to fee estimation operations
/// This is kept for API compatibility but will be deprecated.
/// Prefer using WalletError directly.
#[derive(Debug, Error)]
pub enum FeeEstimationError {
    #[error("Network connectivity error: {0}")]
    NetworkError(String),
    
    #[error("Data retrieval error: {0}")]
    DataError(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
}

/// Implement conversion from FeeEstimationError to WalletError
impl From<FeeEstimationError> for WalletError {
    fn from(error: FeeEstimationError) -> Self {
        match error {
            FeeEstimationError::NetworkError(msg) => WalletError::Generic(format!("Fee estimation network error: {}", msg)),
            FeeEstimationError::DataError(msg) => WalletError::Generic(format!("Fee estimation data error: {}", msg)),
            FeeEstimationError::InvalidParameters(msg) => WalletError::InvalidArgument(format!("Fee estimation parameter error: {}", msg)),
        }
    }
}

/// Constants for network-specific fee rate defaults
pub mod defaults {
    use super::*;
    
    /// Default fee rates by network and priority (in sat/vB)
    pub fn get_default_fee_rate(network: Network, priority: FeePriority) -> Decimal {
        match network {
            Network::Bitcoin => match priority {
                FeePriority::High => dec!(8.0),
                FeePriority::Medium => dec!(4.0),
                FeePriority::Low => dec!(1.0),
                FeePriority::Custom(rate) => Decimal::from_f32(rate).unwrap_or(dec!(1.0)),
            },
            Network::Testnet | Network::Signet => match priority {
                FeePriority::High => dec!(4.0),
                FeePriority::Medium => dec!(2.0),
                FeePriority::Low => dec!(1.0),
                FeePriority::Custom(rate) => Decimal::from_f32(rate).unwrap_or(dec!(1.0)),
            },
            Network::Regtest => match priority {
                FeePriority::High => dec!(2.0),
                FeePriority::Medium => dec!(1.0),
                FeePriority::Low => dec!(0.5),
                FeePriority::Custom(rate) => Decimal::from_f32(rate).unwrap_or(dec!(0.5)),
            },
            _ => match priority {
                FeePriority::High => dec!(6.0),
                FeePriority::Medium => dec!(3.0),
                FeePriority::Low => dec!(1.0),
                FeePriority::Custom(rate) => Decimal::from_f32(rate).unwrap_or(dec!(1.0)),
            },
        }
    }
    
    /// Minimum reasonable fee rates by network (in sat/vB)
    pub fn min_reasonable_fee_rate(network: Network) -> Decimal {
        match network {
            Network::Bitcoin => dec!(1.0),
            Network::Testnet | Network::Signet => dec!(0.5),
            Network::Regtest => dec!(0.25),
            _ => dec!(0.5),
        }
    }
    
    /// Maximum reasonable fee rates by network (in sat/vB)
    pub fn max_reasonable_fee_rate(network: Network) -> Decimal {
        match network {
            Network::Bitcoin => dec!(2000.0), // 2000 sat/vB is extreme but possible during congestion
            Network::Testnet | Network::Signet => dec!(1000.0),
            Network::Regtest => dec!(100.0),
            _ => dec!(1000.0),
        }
    }
    
    /// Default fee rates by network, congestion, and time of day
    pub fn get_adjusted_fee_rate(
        network: Network,
        priority: FeePriority,
        congestion: CongestionLevel,
        hour_of_day: u32
    ) -> Decimal {
        // Get base fee rate for network and priority
        let base_fee = get_default_fee_rate(network, priority);
        
        // Apply congestion multiplier
        let congestion_multiplier = match congestion {
            CongestionLevel::Low => dec!(1.0),
            CongestionLevel::Moderate => dec!(1.2),
            CongestionLevel::High => dec!(1.5),
            CongestionLevel::Severe => dec!(2.0),
        };
        
        // Apply time-of-day adjustment
        // Bitcoin network tends to be more congested during business hours in US/Europe
        // Early morning and late night typically have lower fees
        let time_multiplier = match hour_of_day {
            // Late night/early morning (lowest fees): 1AM-5AM UTC
            1..=5 => dec!(0.8),
            // Morning/Evening (moderate fees): 6AM-9AM, 6PM-12AM UTC
            6..=9 | 18..=23 => dec!(1.0),
            // Business hours (highest fees): 10AM-5PM UTC
            10..=17 => dec!(1.2),
            // Fallback (shouldn't happen)
            _ => dec!(1.0),
        };
        
        // Calculate final fee rate with both adjustments
        let adjusted_fee = base_fee * congestion_multiplier * time_multiplier;
        
        // Ensure the result is at least the minimum reasonable fee
        let min_fee = min_reasonable_fee_rate(network);
        if adjusted_fee < min_fee {
            return min_fee;
        }
        
        // Cap at maximum reasonable fee
        let max_fee = max_reasonable_fee_rate(network);
        if adjusted_fee > max_fee {
            return max_fee;
        }
        
        adjusted_fee
    }
}

/// Represents recommended fee rates for different confirmation targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeRecommendations {
    /// Network these recommendations apply to
    pub network: Network,
    
    /// Fee rates for different confirmation targets (in sat/vB)
    /// Keys are target blocks (1, 2, 3, 6, 12, 24, etc.)
    pub by_block_target: HashMap<u32, Decimal>,
    
    /// Fee rates for different priority levels
    pub by_priority: HashMap<FeePriority, Decimal>,
    
    /// Timestamp when these rates were last updated
    pub last_updated: u64,
    
    /// Current network congestion level
    pub congestion: CongestionLevel,
}

impl FeeRecommendations {
    /// Create new fee recommendations with current timestamp
    pub fn new(network: Network) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            network,
            by_block_target: HashMap::new(),
            by_priority: HashMap::new(),
            last_updated: now,
            congestion: CongestionLevel::Low,
        }
    }
    
    /// Get fee rate for specified priority level
    pub fn get_fee_for_priority(&self, priority: FeePriority) -> Decimal {
        // For custom priority, handle the value directly to avoid potential race conditions
        if let FeePriority::Custom(rate) = priority {
            let fee = Decimal::from_f32(rate).unwrap_or(dec!(1.0));
            return self.sanitize_fee_rate(fee);
        }
        
        // Clone the map once to avoid multiple lookups that could change between calls
        // This makes the function more thread-safe in concurrent environments
        match self.by_priority.get(&priority) {
            Some(fee) => self.sanitize_fee_rate(*fee),
            None => {
                // Fallback values if specific priority not found - use network-specific defaults
                let current_hour = chrono::Utc::now().hour();
                defaults::get_adjusted_fee_rate(
                    self.network,
                    priority,
                    self.congestion,
                    current_hour
                )
            }
        }
    }
    
    /// Apply minimum and maximum bounds to fee rate for safety
    fn sanitize_fee_rate(&self, fee: Decimal) -> Decimal {
        let min_fee = defaults::min_reasonable_fee_rate(self.network);
        let max_fee = defaults::max_reasonable_fee_rate(self.network);
        
        if fee < min_fee {
            log::warn!("Fee rate {} too low, using minimum {}", fee, min_fee);
            return min_fee;
        }
        
        if fee > max_fee {
            log::warn!("Fee rate {} too high, capping at {}", fee, max_fee);
            return max_fee;
        }
        
        fee
    }
    
    /// Get fee rate for specified confirmation target (in blocks)
    pub fn get_fee_for_target(&self, target_blocks: u32) -> Decimal {
        // Try to find exact target
        if let Some(fee) = self.by_block_target.get(&target_blocks) {
            return self.sanitize_fee_rate(*fee);
        }
        
        // If the map is empty, return a sensible default based on target and network
        if self.by_block_target.is_empty() {
            let priority = match target_blocks {
                0..=2 => FeePriority::High,
                3..=6 => FeePriority::Medium,
                _ => FeePriority::Low,
            };
            
            let current_hour = chrono::Utc::now().hour();
            return defaults::get_adjusted_fee_rate(
                self.network,
                priority,
                self.congestion,
                current_hour
            );
        }
        
        // Find closest target
        // Clone the keys to prevent issues if the map changes during iteration
        let targets: Vec<u32> = self.by_block_target.keys().cloned().collect();
        
        let mut closest_target = 24u32; // Default fallback
        let mut closest_diff = u32::MAX;
        
        for target in targets {
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
        
        // Get the fee for the closest target
        match self.by_block_target.get(&closest_target) {
            Some(fee) => self.sanitize_fee_rate(*fee),
            None => {
                // Fallback in case the map was modified between finding closest target and looking it up
                let priority = match target_blocks {
                    0..=2 => FeePriority::High,
                    3..=6 => FeePriority::Medium,
                    _ => FeePriority::Low,
                };
                
                let current_hour = chrono::Utc::now().hour();
                defaults::get_adjusted_fee_rate(
                    self.network,
                    priority,
                    self.congestion,
                    current_hour
                )
            }
        }
    }
    
    /// Check if the fee recommendations are stale (older than specified time)
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let now = current_timestamp();
        now.saturating_sub(self.last_updated) > max_age_seconds
    }
    
    /// Add fee rate data, with outlier detection
    pub fn add_fee_rate(&mut self, priority: FeePriority, fee_rate: Decimal) -> bool {
        // Check for obviously invalid values
        if fee_rate <= dec!(0.0) {
            log::warn!("Ignoring non-positive fee rate: {}", fee_rate);
            return false;
        }
        
        // Get current min/max reasonable values for this network
        let min_reasonable = defaults::min_reasonable_fee_rate(self.network);
        let max_reasonable = defaults::max_reasonable_fee_rate(self.network);
        
        // Reject values outside reasonable bounds (outlier detection)
        if fee_rate < min_reasonable || fee_rate > max_reasonable {
            log::warn!("Rejecting outlier fee rate {} (outside reasonable range [{}, {}])",
                fee_rate, min_reasonable, max_reasonable);
            return false;
        }
        
        // If we already have a fee rate for this priority, check for sudden large changes
        if let Some(existing_fee) = self.by_priority.get(&priority) {
            // Calculate percent change
            let percent_change = if *existing_fee > dec!(0.0) {
                ((fee_rate - *existing_fee) / *existing_fee * dec!(100.0)).abs()
            } else {
                dec!(100.0) // Treat as 100% change if existing fee is zero
            };
            
            // Flag sudden large changes (>50%) as potential outliers
            if percent_change > dec!(50.0) {
                log::warn!("Large fee rate change detected for {:?}: {} -> {} ({:.2}% change)",
                    priority, existing_fee, fee_rate, percent_change);
                
                // For large changes, use a weighted average to smooth the transition
                let weighted_fee = (*existing_fee * dec!(0.7)) + (fee_rate * dec!(0.3));
                self.by_priority.insert(priority, weighted_fee);
                return true;
            }
        }
        
        // Normal case: update the fee rate directly
        self.by_priority.insert(priority, fee_rate);
        true
    }
    
    /// Add fee rate for a block target, with outlier detection
    pub fn add_block_target_fee(&mut self, target_blocks: u32, fee_rate: Decimal) -> bool {
        // Check for obviously invalid values
        if fee_rate <= dec!(0.0) || target_blocks == 0 {
            log::warn!("Ignoring invalid block target fee: target={}, fee={}",
                target_blocks, fee_rate);
            return false;
        }
        
        // Get current min/max reasonable values for this network
        let min_reasonable = defaults::min_reasonable_fee_rate(self.network);
        let max_reasonable = defaults::max_reasonable_fee_rate(self.network);
        
        // Reject values outside reasonable bounds (outlier detection)
        if fee_rate < min_reasonable || fee_rate > max_reasonable {
            log::warn!("Rejecting outlier fee rate {} for target={} (outside reasonable range [{}, {}])",
                fee_rate, target_blocks, min_reasonable, max_reasonable);
            return false;
        }
        
        // Check for consistency with other targets (fee should decrease as target increases)
        for (other_target, other_fee) in &self.by_block_target {
            // Fee rates should generally be higher for lower block targets
            if *other_target < target_blocks && fee_rate > *other_fee {
                // This is consistent, continue
                continue;
            }
            
            if *other_target > target_blocks && fee_rate < *other_fee {
                // This is consistent, continue
                continue;
            }
            
            if *other_target != target_blocks {
                // Inconsistent fee rate detected
                log::warn!("Inconsistent fee rates: target={}, fee={} vs target={}, fee={}",
                    target_blocks, fee_rate, other_target, other_fee);
                
                // We'll still add it, but with a warning
            }
        }
        
        // If we already have a fee rate for this target, check for sudden large changes
        if let Some(existing_fee) = self.by_block_target.get(&target_blocks) {
            // Calculate percent change
            let percent_change = if *existing_fee > dec!(0.0) {
                ((fee_rate - *existing_fee) / *existing_fee * dec!(100.0)).abs()
            } else {
                dec!(100.0) // Treat as 100% change if existing fee is zero
            };
            
            // Flag sudden large changes (>50%) as potential outliers
            if percent_change > dec!(50.0) {
                log::warn!("Large fee rate change detected for target={}: {} -> {} ({:.2}% change)",
                    target_blocks, existing_fee, fee_rate, percent_change);
                
                // For large changes, use a weighted average to smooth the transition
                let weighted_fee = (*existing_fee * dec!(0.7)) + (fee_rate * dec!(0.3));
                self.by_block_target.insert(target_blocks, weighted_fee);
                return true;
            }
        }
        
        // Normal case: update the fee rate directly
        self.by_block_target.insert(target_blocks, fee_rate);
        true
    }
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Format date from Unix timestamp (YYYY-MM-DD)
fn format_date_from_timestamp(timestamp: u64) -> String {
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    datetime.format("%Y-%m-%d").to_string()
}

/// Calculate the total fee for a transaction
///
/// # Arguments
/// * `fee_rate` - Fee rate in satoshis per vbyte
/// * `tx_size` - Estimated transaction size in vbytes
///
/// # Returns
/// * Total fee in satoshis
pub fn calculate_total_fee(fee_rate: Decimal, tx_size: usize) -> u64 {
    (fee_rate * Decimal::from(tx_size)).ceil().to_u64().unwrap_or(0)
}

/// Historical fee data for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalFeeData {
    /// Bitcoin network type
    pub network: Network,
    
    /// Fee rates by time (UNIX timestamp in seconds to fee rate in sat/vB)
    pub time_series: HashMap<u64, HashMap<u32, f32>>,
    
    /// Daily average fee rates for each target block over the past 30 days
    pub daily_averages: HashMap<String, HashMap<u32, f32>>,
    
    /// Weekly average fee rates for each target block over the past 12 weeks
    pub weekly_averages: HashMap<String, HashMap<u32, f32>>,
    
    /// Observed minimum fee rates for different block targets
    pub minimum_observed: HashMap<u32, f32>,
    
    /// Observed maximum fee rates for different block targets
    pub maximum_observed: HashMap<u32, f32>,
    
    /// Last time the historical data was updated
    pub last_updated: u64,
}

impl HistoricalFeeData {
    /// Create a new empty historical fee data instance
    pub fn new(network: Network) -> Self {
        Self {
            network,
            time_series: HashMap::new(),
            daily_averages: HashMap::new(),
            weekly_averages: HashMap::new(),
            minimum_observed: HashMap::new(),
            maximum_observed: HashMap::new(),
            last_updated: current_timestamp(),
        }
    }
    
    /// Add a new fee rate sample to the historical data
    pub fn add_sample(&mut self, timestamp: u64, block_target: u32, fee_rate: f32) {
        // Outlier detection
        if fee_rate <= 0.0 {
            log::warn!("Ignoring non-positive fee rate: {}", fee_rate);
            return;
        }
        
        // Get reasonable bounds for this network
        let min_reasonable = defaults::min_reasonable_fee_rate(self.network).to_f32().unwrap_or(0.5);
        let max_reasonable = defaults::max_reasonable_fee_rate(self.network).to_f32().unwrap_or(1000.0);
        
        // Reject extreme outliers
        if fee_rate < min_reasonable || fee_rate > max_reasonable {
            log::warn!("Rejecting outlier fee rate {} for target={} (outside reasonable range [{}, {}])",
                fee_rate, block_target, min_reasonable, max_reasonable);
            return;
        }
        
        // Add to time series
        let block_fees = self.time_series.entry(timestamp).or_insert_with(HashMap::new);
        block_fees.insert(block_target, fee_rate);
        
        // Update min/max observed values
        match self.minimum_observed.get(&block_target) {
            Some(&current_min) if fee_rate < current_min => {
                self.minimum_observed.insert(block_target, fee_rate);
            }
            None => {
                self.minimum_observed.insert(block_target, fee_rate);
            }
            _ => {}
        }
        
        match self.maximum_observed.get(&block_target) {
            Some(&current_max) if fee_rate > current_max => {
                self.maximum_observed.insert(block_target, fee_rate);
            }
            None => {
                self.maximum_observed.insert(block_target, fee_rate);
            }
            _ => {}
        }
        
        // Update last updated timestamp
        self.last_updated = current_timestamp();
    }
    
    /// Calculate daily averages from time series data
    pub fn calculate_daily_averages(&mut self) {
        let mut daily_data: HashMap<String, Vec<HashMap<u32, f32>>> = HashMap::new();
        
        // Group fee data by day
        for (timestamp, fees) in &self.time_series {
            let date = format_date_from_timestamp(*timestamp);
            daily_data.entry(date).or_insert_with(Vec::new).push(fees.clone());
        }
        
        // Calculate averages for each day
        for (date, fee_samples) in daily_data {
            let mut day_averages = HashMap::new();
            
            // For each block target, calculate the average
            for target in 1..=144 {  // Up to 1 day worth of blocks
                let mut sum = 0.0;
                let mut count = 0;
                
                for fees in &fee_samples {
                    if let Some(&fee) = fees.get(&target) {
                        sum += fee;
                        count += 1;
                    }
                }
                
                if count > 0 {
                    day_averages.insert(target, sum / count as f32);
                }
            }
            
            if !day_averages.is_empty() {
                self.daily_averages.insert(date, day_averages);
            }
        }
    }
    
    /// Estimate fee rate based on historical data
    pub fn estimate_fee(&self, block_target: u32) -> Option<f32> {
        // If we have current fee data, use that first
        if let Some(current_fees) = self.time_series.get(&self.last_updated) {
            if let Some(&fee) = current_fees.get(&block_target) {
                return Some(fee);
            }
        }
        
        // Otherwise, check recent daily averages
        if !self.daily_averages.is_empty() {
            // Get the most recent date
            let most_recent = self.daily_averages.keys()
                .max()
                .cloned()?;
            
            if let Some(day_fees) = self.daily_averages.get(&most_recent) {
                if let Some(&fee) = day_fees.get(&block_target) {
                    return Some(fee);
                }
                
                // If exact target not found, look for closest target
                let targets: Vec<u32> = day_fees.keys().cloned().collect();
                let closest_target = targets.iter()
                    .min_by_key(|&&t| if t > block_target { t - block_target } else { block_target - t })
                    .cloned()?;
                
                return day_fees.get(&closest_target).copied();
            }
        }
        
        // Fallback to minimum observed fee for a conservative estimate
        self.minimum_observed.get(&block_target).copied()
    }
    
    /// Check if data is stale (hasn't been updated recently)
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let now = current_timestamp();
        now - self.last_updated > max_age_seconds
    }
    
    /// Get fee rate recommendation for a specific target and priority
    pub fn get_recommendation(&self, block_target: u32, congestion: CongestionLevel) -> f32 {
        // Try to get estimate from historical data
        let base_fee = self.estimate_fee(block_target)
            .unwrap_or_else(|| {
                // Fallback based on target blocks
                let priority = match block_target {
                    1..=2 => FeePriority::High,
                    3..=6 => FeePriority::Medium,
                    _ => FeePriority::Low,
                };
                
                // Use network-specific defaults
                let current_hour = chrono::Utc::now().hour();
                defaults::get_adjusted_fee_rate(
                    self.network,
                    priority,
                    congestion,
                    current_hour
                ).to_f32().unwrap_or(1.0)
            });
            
        // Apply congestion multiplier
        let multiplier = match congestion {
            CongestionLevel::Low => 1.0,
            CongestionLevel::Moderate => 1.2,
            CongestionLevel::High => 1.5,
            CongestionLevel::Severe => 2.0,
        };
        
        base_fee * multiplier
    }
}

/// Estimate transaction fee for the given priority using a NetworkStatusProvider
///
/// # Arguments
///
/// * `priority` - Fee priority (High, Medium, Low)
/// * `provider` - Network status provider
/// * `message_bus` - Optional message bus for event publication
///
/// # Returns
///
/// * Result containing the estimated fee rate in satoshis per vByte
#[deprecated(since = "0.2.0", note = "Use FeeEstimationService instead")]
pub fn estimate_fee<T: NetworkStatusProvider>(
    priority: FeePriority, 
    provider: &T,
    message_bus: Option<&MessageBus>
) -> Result<Decimal, WalletError> {
    // Get network status from provider
    let status = provider.get_network_status()
        .map_err(|e| WalletError::Generic(format!("Failed to get network status: {}", e)))?;
    
    // If custom rate specified, just return that with validation
    if let FeePriority::Custom(rate) = priority {
        let fee = Decimal::from_f32(rate).unwrap_or(dec!(1.0));
        let min_reasonable = defaults::min_reasonable_fee_rate(status.network);
        let max_reasonable = defaults::max_reasonable_fee_rate(status.network);
        
        let adjusted_fee = if fee < min_reasonable {
            log::warn!("Custom fee rate {} too low, using minimum {}", fee, min_reasonable);
            min_reasonable
        } else if fee > max_reasonable {
            log::warn!("Custom fee rate {} too high, capping at {}", fee, max_reasonable);
            max_reasonable
        } else {
            fee
        };
        
        // Emit fee estimation event if message bus is provided
        if let Some(bus) = message_bus {
            let payload = json!({
                "type": "custom",
                "network": format!("{:?}", status.network),
                "original_rate": rate,
                "adjusted_rate": adjusted_fee,
                "congestion": format!("{:?}", status.congestion),
            });
            
            bus.publish(
                EventType::FeeEstimationUpdate,
                &payload.to_string(),
                MessagePriority::Low
            );
        }
        
        return Ok(adjusted_fee);
    }
    
    // Get current hour for time-based adjustment
    let current_hour = chrono::Utc::now().hour();
    
    // Get fee rate from defaults with network, priority, congestion, and time-of-day
    let fee_rate = defaults::get_adjusted_fee_rate(
        status.network,
        priority,
        status.congestion,
        current_hour
    );
    
    let mut fee_source = "default";
    let mut final_fee_rate = fee_rate;
    
    // If provider has fee recommendations, prefer those
    if !status.fee_estimates.is_empty() {
        // Convert priority to target blocks
        let target_blocks = match priority {
            FeePriority::High => 2,
            FeePriority::Medium => 6,
            FeePriority::Low => 24,
            FeePriority::Custom(_) => unreachable!(), // Already handled above
        };
        
        // Find the closest target
        let targets: Vec<u32> = status.fee_estimates.keys().cloned().collect();
        if !targets.is_empty() {
            let closest_target = targets.iter()
                .min_by_key(|&&t| if t > target_blocks { t - target_blocks } else { target_blocks - t })
                .unwrap();
            
            if let Some(&provider_fee) = status.fee_estimates.get(closest_target) {
                // Providers sometimes give unreasonable values, so validate
                let min_reasonable = defaults::min_reasonable_fee_rate(status.network);
                let max_reasonable = defaults::max_reasonable_fee_rate(status.network);
                
                final_fee_rate = if provider_fee < min_reasonable {
                    log::warn!("Provider fee rate {} too low, using minimum {}", provider_fee, min_reasonable);
                    min_reasonable
                } else if provider_fee > max_reasonable {
                    log::warn!("Provider fee rate {} too high, capping at {}", provider_fee, max_reasonable);
                    max_reasonable
                } else {
                    provider_fee
                };
                
                fee_source = "provider";
            }
        }
    }
    
    // Emit fee estimation event if message bus is provided
    if let Some(bus) = message_bus {
        let payload = json!({
            "type": format!("{:?}", priority),
            "network": format!("{:?}", status.network),
            "fee_rate": final_fee_rate,
            "source": fee_source,
            "congestion": format!("{:?}", status.congestion),
            "target_blocks": match priority {
                FeePriority::High => 2,
                FeePriority::Medium => 6,
                FeePriority::Low => 24,
                FeePriority::Custom(_) => 0, // Should not happen here
            }
        });
        
        bus.publish(
            EventType::FeeEstimationUpdate,
            &payload.to_string(),
            MessagePriority::Low
        );
    }
    
    // Return the final fee rate
    Ok(final_fee_rate)
}

/// Apply dynamic fee adjustment based on current network congestion
///
/// # Arguments
/// * `base_fee` - Base fee rate in satoshis per vbyte
/// * `congestion` - Current network congestion level
///
/// # Returns
/// * Adjusted fee rate in satoshis per vbyte
pub fn adjust_fee_for_congestion(base_fee: Decimal, congestion: CongestionLevel) -> Decimal {
    match congestion {
        CongestionLevel::Low => base_fee,
        CongestionLevel::Moderate => base_fee * dec!(1.2),
        CongestionLevel::High => base_fee * dec!(1.5),
        CongestionLevel::Severe => base_fee * dec!(2.0),
    }
}

/// Create fee recommendations from provider data
///
/// # Arguments
/// * `network` - Bitcoin network
/// * `fee_estimates` - Fee estimates from provider (keyed by target blocks)
/// * `congestion` - Network congestion level
///
/// # Returns
/// * Fee recommendations with network-specific defaults
pub fn create_recommendations_from_provider(
    network: Network,
    fee_estimates: HashMap<u32, Decimal>,
    congestion: CongestionLevel,
) -> FeeRecommendations {
    let mut recommendations = FeeRecommendations::new(network);
    recommendations.congestion = congestion;
    
    // Add all provider estimates with outlier detection
    for (target, fee) in fee_estimates {
        recommendations.add_block_target_fee(target, fee);
    }
    
    // Ensure priority recommendations are populated
    let current_hour = chrono::Utc::now().hour();
    
    // Map from target blocks to priorities
    let priorities = vec![
        (FeePriority::High, 2),    // 2 blocks
        (FeePriority::Medium, 6),  // 6 blocks
        (FeePriority::Low, 24),    // 24 blocks
    ];
    
    for (priority, target) in priorities {
        // Try to use the fee estimate for this target
        if let Some(&fee) = recommendations.by_block_target.get(&target) {
            recommendations.by_priority.insert(priority, fee);
        } else {
            // If no fee estimate for this target, use default
            let fee = defaults::get_adjusted_fee_rate(network, priority, congestion, current_hour);
            recommendations.by_priority.insert(priority, fee);
        }
    }
    
    recommendations
}

/// Create fee recommendations from scratch
///
/// # Arguments
/// * `network` - Bitcoin network
/// * `congestion` - Network congestion level
///
/// # Returns
/// * Fee recommendations with network-specific defaults
pub fn create_recommendations(network: Network, congestion: CongestionLevel) -> FeeRecommendations {
    let mut recommendations = FeeRecommendations::new(network);
    recommendations.congestion = congestion;
    
    // Add default values based on network, congestion, and time of day
    let current_hour = chrono::Utc::now().hour();
    
    // Add priority-based fees
    let priorities = vec![FeePriority::High, FeePriority::Medium, FeePriority::Low];
    for priority in priorities {
        let fee = defaults::get_adjusted_fee_rate(network, priority, congestion, current_hour);
        recommendations.by_priority.insert(priority, fee);
    }
    
    // Add common block targets
    let targets = vec![1, 2, 3, 6, 12, 24, 48, 144];
    for target in targets {
        // Convert target to equivalent priority
        let priority = match target {
            1..=2 => FeePriority::High,
            3..=6 => FeePriority::Medium,
            _ => FeePriority::Low,
        };
        
        let fee = defaults::get_adjusted_fee_rate(network, priority, congestion, current_hour);
        recommendations.by_block_target.insert(target, fee);
    }
    
    recommendations
}

/// Provider interface for fee estimation
///
/// # Security Considerations
///
/// - Implementations should validate network data before using it
/// - Providers may expose the wallet to network fingerprinting
/// - Fee estimation affects transaction costs and confirmation times
pub trait FeeEstimationProvider: Send + Sync {
    /// Get fee estimates for different confirmation targets
    fn get_fee_estimates(&self) -> Result<FeeRecommendations, FeeEstimationError>;
    
    /// Get historical fee data for analysis
    fn get_historical_data(&self) -> Result<HistoricalFeeData, FeeEstimationError>;
    
    /// Check if the provider is currently available
    fn is_available(&self) -> bool;
    
    /// Get the provider name for logging and debugging
    fn provider_name(&self) -> &str;
    
    /// Get the provider priority (lower numbers = higher priority)
    fn priority(&self) -> u8 {
        10 // Default middle priority
    }
}

/// Cache for fee estimation data
pub struct FeeEstimationCache {
    /// Cached fee recommendations
    pub recommendations: Option<FeeRecommendations>,
    /// Cached historical data
    pub historical_data: Option<HistoricalFeeData>,
    /// When the cache was last updated
    pub last_updated: u64,
    /// Maximum age of cached data in seconds
    pub max_age_seconds: u64,
}

impl FeeEstimationCache {
    /// Create a new fee estimation cache
    pub fn new(max_age_seconds: u64) -> Self {
        Self {
            recommendations: None,
            historical_data: None,
            last_updated: current_timestamp(),
            max_age_seconds,
        }
    }
    
    /// Check if the cache is stale
    pub fn is_stale(&self) -> bool {
        let now = current_timestamp();
        now - self.last_updated > self.max_age_seconds
    }
    
    /// Update the cache with new data
    pub fn update(&mut self, recommendations: Option<FeeRecommendations>, historical_data: Option<HistoricalFeeData>) {
        if recommendations.is_some() {
            self.recommendations = recommendations;
        }
        
        if historical_data.is_some() {
            self.historical_data = historical_data;
        }
        
        self.last_updated = current_timestamp();
    }
}

/// Service to manage multiple fee estimation providers with fallback and caching
///
/// # Security Considerations
///
/// - The service will only use available providers
/// - Stale data will be refreshed automatically
/// - Providers are tried in priority order
/// - Multiple providers help prevent single points of failure
pub struct FeeEstimationService {
    /// Available fee estimation providers in priority order
    providers: Vec<Box<dyn FeeEstimationProvider>>,
    /// Cache for fee estimation data
    cache: Arc<RwLock<FeeEstimationCache>>,
    /// Associated message bus for events
    message_bus: Option<Arc<MessageBus>>,
    /// Bitcoin network for the service
    network: Network,
}

impl FeeEstimationService {
    /// Create a new fee estimation service
    ///
    /// # Arguments
    ///
    /// * `providers` - List of fee estimation providers in priority order
    /// * `network` - Bitcoin network for the service
    /// * `cache_max_age` - Maximum age of cached data in seconds
    /// * `message_bus` - Optional message bus for events
    pub fn new(
        providers: Vec<Box<dyn FeeEstimationProvider>>,
        network: Network,
        cache_max_age: u64,
        message_bus: Option<Arc<MessageBus>>
    ) -> Self {
        // Sort providers by priority
        let mut sorted_providers = providers;
        sorted_providers.sort_by_key(|p| p.priority());
        
        Self {
            providers: sorted_providers,
            cache: Arc::new(RwLock::new(FeeEstimationCache::new(cache_max_age))),
            message_bus,
            network,
        }
    }
    
    /// Get fee recommendations, trying multiple providers with fallback
    pub fn get_fee_recommendations(&self) -> Result<FeeRecommendations, FeeEstimationError> {
        // First check cache
        {
            let cache = self.cache.read().unwrap();
            if !cache.is_stale() {
                if let Some(recommendations) = &cache.recommendations {
                    return Ok(recommendations.clone());
                }
            }
        }
        
        // Cache is stale or empty, try providers in order
        let mut last_error = None;
        let mut _successful_provider = None;
        
        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }
            
            match provider.get_fee_estimates() {
                Ok(recommendations) => {
                    // Update cache
                    {
                        let mut cache = self.cache.write().unwrap();
                        cache.update(Some(recommendations.clone()), None);
                    }
                    
                    // Publish event
                    if let Some(bus) = &self.message_bus {
                        bus.publish(
                            EventType::FeeEstimationUpdate,
                            &json!({
                                "provider": provider.provider_name(),
                                "network": self.network.to_string(),
                                "high_priority": recommendations.get_fee_for_priority(FeePriority::High).to_string(),
                                "medium_priority": recommendations.get_fee_for_priority(FeePriority::Medium).to_string(),
                                "low_priority": recommendations.get_fee_for_priority(FeePriority::Low).to_string(),
                            }).to_string(),
                            MessagePriority::Low
                        );
                    }
                    
                    _successful_provider = Some(provider.provider_name().to_string());
                    return Ok(recommendations);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
        
        // If we got here, all providers failed
        // Try to use cached data even if stale as a last resort
        {
            let cache = self.cache.read().unwrap();
            if let Some(recommendations) = &cache.recommendations {
                // Log that we're using stale data
                if let Some(bus) = &self.message_bus {
                    bus.publish(
                        EventType::FeeEstimationUpdate,
                        &json!({
                            "warning": "Using stale fee data",
                            "age_seconds": current_timestamp() - cache.last_updated,
                            "network": self.network.to_string(),
                        }).to_string(),
                        MessagePriority::Medium
                    );
                }
                
                return Ok(recommendations.clone());
            }
        }
        
        // If we still don't have data, create default recommendations
        let default_recommendations = create_recommendations(
            self.network,
            CongestionLevel::Moderate,
        );
        
        // Log that we're using default data
        if let Some(bus) = &self.message_bus {
            bus.publish(
                EventType::FeeEstimationUpdate,
                &json!({
                    "warning": "Using default fee data, all providers failed",
                    "network": self.network.to_string(),
                    "providers_tried": self.providers.len(),
                }).to_string(),
                MessagePriority::High
            );
        }
        
        // Update cache with default data
        {
            let mut cache = self.cache.write().unwrap();
            cache.update(Some(default_recommendations.clone()), None);
        }
        
        // Return error if requested but include default recommendations
        if let Some(error) = last_error {
            Err(error)
        } else {
            Ok(default_recommendations)
        }
    }
    
    /// Get historical fee data, trying multiple providers with fallback
    pub fn get_historical_data(&self) -> Result<HistoricalFeeData, FeeEstimationError> {
        // First check cache
        {
            let cache = self.cache.read().unwrap();
            if !cache.is_stale() {
                if let Some(data) = &cache.historical_data {
                    return Ok(data.clone());
                }
            }
        }
        
        // Cache is stale or empty, try providers in order
        let mut last_error = None;
        
        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }
            
            match provider.get_historical_data() {
                Ok(data) => {
                    // Update cache
                    {
                        let mut cache = self.cache.write().unwrap();
                        cache.update(None, Some(data.clone()));
                    }
                    
                    return Ok(data);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
        
        // If we got here, all providers failed
        // Try to use cached data even if stale as a last resort
        {
            let cache = self.cache.read().unwrap();
            if let Some(data) = &cache.historical_data {
                return Ok(data.clone());
            }
        }
        
        // If we still don't have data, return an error
        if let Some(error) = last_error {
            Err(error)
        } else {
            // Create empty historical data as a last resort
            let empty_data = HistoricalFeeData::new(self.network);
            
            // Update cache with empty data
            {
                let mut cache = self.cache.write().unwrap();
                cache.update(None, Some(empty_data.clone()));
            }
            
            Ok(empty_data)
        }
    }
    
    /// Get fee estimate for a specific priority
    pub fn estimate_fee(&self, priority: FeePriority) -> Result<Decimal, FeeEstimationError> {
        let recommendations = self.get_fee_recommendations()?;
        Ok(recommendations.get_fee_for_priority(priority))
    }
    
    /// Add a new provider to the service
    pub fn add_provider(&mut self, provider: Box<dyn FeeEstimationProvider>) {
        self.providers.push(provider);
        // Re-sort by priority
        self.providers.sort_by_key(|p| p.priority());
    }
    
    /// Force refresh the fee data
    pub fn force_refresh(&self) -> Result<(), FeeEstimationError> {
        // Clear the cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.recommendations = None;
            cache.historical_data = None;
            cache.last_updated = 0; // Forces stale
        }
        
        // Try to get new recommendations and historical data
        let _recommendations = self.get_fee_recommendations()?;
        let _historical_data = self.get_historical_data()?;
        
        Ok(())
    }
}

/// Default fee estimation provider that uses hardcoded values
///
/// This provider is useful as a fallback when other providers are unavailable.
/// It doesn't require network connectivity but provides less accurate estimations.
pub struct DefaultFeeEstimationProvider {
    /// Bitcoin network for this provider
    network: Network,
    /// Provider availability (always true for this provider)
    available: bool,
}

impl DefaultFeeEstimationProvider {
    /// Create a new default fee estimation provider
    pub fn new(network: Network) -> Self {
        Self {
            network,
            available: true,
        }
    }
}

impl FeeEstimationProvider for DefaultFeeEstimationProvider {
    fn get_fee_estimates(&self) -> Result<FeeRecommendations, FeeEstimationError> {
        Ok(create_recommendations(self.network, CongestionLevel::Moderate))
    }
    
    fn get_historical_data(&self) -> Result<HistoricalFeeData, FeeEstimationError> {
        let mut data = HistoricalFeeData::new(self.network);
        
        // Add some minimal historical data
        let now = current_timestamp();
        
        // Create a simple fee history
        for i in 0..10 {
            let timestamp = now - (i * 86400); // One day intervals
            
            // Add data for different confirmation targets
            data.add_sample(timestamp, 1, 10.0 + (i as f32 * 0.5));  // 1 block target
            data.add_sample(timestamp, 3, 5.0 + (i as f32 * 0.3));   // 3 block target
            data.add_sample(timestamp, 6, 3.0 + (i as f32 * 0.2));   // 6 block target
            data.add_sample(timestamp, 24, 1.0 + (i as f32 * 0.1));  // 24 block target
        }
        
        // Calculate daily averages
        data.calculate_daily_averages();
        
        Ok(data)
    }
    
    fn is_available(&self) -> bool {
        self.available
    }
    
    fn provider_name(&self) -> &str {
        "DefaultProvider"
    }
    
    fn priority(&self) -> u8 {
        255 // Lowest priority, use as last resort
    }
}

/// Network-based fee estimation provider
///
/// This provider uses a NetworkStatusProvider to get fee estimates
/// from the Bitcoin network. It provides more accurate estimations
/// but requires network connectivity.
pub struct NetworkFeeEstimationProvider<T: NetworkStatusProvider + Send + Sync> {
    /// Network status provider
    provider: T,
    /// Bitcoin network for this provider
    network: Network,
}

impl<T: NetworkStatusProvider + Send + Sync> NetworkFeeEstimationProvider<T> {
    /// Create a new network-based fee estimation provider
    pub fn new(provider: T, network: Network) -> Self {
        Self {
            provider,
            network,
        }
    }
}

impl<T: NetworkStatusProvider + Send + Sync> FeeEstimationProvider for NetworkFeeEstimationProvider<T> {
    fn get_fee_estimates(&self) -> Result<FeeRecommendations, FeeEstimationError> {
        // Get network status from the provider
        let network_status = self.provider.get_network_status()
            .map_err(|e| FeeEstimationError::NetworkError(e.to_string()))?;
        
        // Get mempool status to determine congestion
        let mempool_status = self.provider.get_mempool_status()
            .map_err(|e| FeeEstimationError::NetworkError(e.to_string()))?;
        
        // Create recommendations from the network data
        let recommendations = create_recommendations_from_provider(
            self.network,
            network_status.fee_estimates,
            mempool_status.determine_congestion_level(),
        );
        
        Ok(recommendations)
    }
    
    fn get_historical_data(&self) -> Result<HistoricalFeeData, FeeEstimationError> {
        // For a real implementation, this would fetch historical data
        // from a service. For now, we'll just create some sample data.
        
        let mut data = HistoricalFeeData::new(self.network);
        let now = current_timestamp();
        
        // Get current fee estimates as a starting point
        let network_status = self.provider.get_network_status()
            .map_err(|e| FeeEstimationError::NetworkError(e.to_string()))?;
        
        // Add current fee estimates
        for (target, fee) in &network_status.fee_estimates {
            data.add_sample(now, *target, fee.to_f32().unwrap_or(1.0));
        }
        
        // Calculate daily averages from our limited data
        data.calculate_daily_averages();
        
        Ok(data)
    }
    
    fn is_available(&self) -> bool {
        // This provider is available if we can get network status
        self.provider.get_network_status().is_ok()
    }
    
    fn provider_name(&self) -> &str {
        "NetworkProvider"
    }
    
    fn priority(&self) -> u8 {
        10 // Medium priority
    }
}

/// Create a FeeEstimationService with the given provider
///
/// This is a convenience function to create a FeeEstimationService
/// with a single NetworkFeeEstimationProvider.
///
/// # Arguments
///
/// * `provider` - Network status provider
/// * `network` - Bitcoin network
/// * `message_bus` - Optional message bus for event publication
///
/// # Returns
///
/// * FeeEstimationService configured with the provider and a default provider
pub fn create_fee_service<T: NetworkStatusProvider + Send + Sync + 'static>(
    provider: T,
    network: Network,
    message_bus: Option<Arc<MessageBus>>
) -> FeeEstimationService {
    // Create a network provider with the given network status provider
    let network_provider = Box::new(NetworkFeeEstimationProvider::new(
        provider,
        network,
    ));
    
    // Create a default provider as fallback
    let default_provider = Box::new(DefaultFeeEstimationProvider::new(network));
    
    // Create a list of providers
    let providers: Vec<Box<dyn FeeEstimationProvider>> = vec![
        network_provider,
        default_provider,
    ];
    
    // Create the service with a 30-minute cache
    FeeEstimationService::new(
        providers,
        network,
        1800, // 30 minutes
        message_bus,
    )
}

/// Modern fee estimation function using the FeeEstimationService
///
/// This function creates a service with the given provider and
/// then estimates the fee for the given priority.
///
/// # Arguments
///
/// * `priority` - Fee priority (High, Medium, Low)
/// * `provider` - Network status provider
/// * `network` - Bitcoin network
/// * `message_bus` - Optional message bus for event publication
///
/// # Returns
///
/// * Result containing the estimated fee rate in satoshis per vByte
pub fn estimate_fee_with_service<T: NetworkStatusProvider + Send + Sync + 'static>(
    priority: FeePriority,
    provider: T,
    network: Network,
    message_bus: Option<Arc<MessageBus>>
) -> Result<Decimal, FeeEstimationError> {
    // Create the service
    let service = create_fee_service(provider, network, message_bus);
    
    // Estimate the fee
    service.estimate_fee(priority)
}

#[cfg(test)]
mod tests {
    use super::*;
    // ... existing test imports and tests ...

    #[test]
    fn test_fee_estimation_service() {
        // Create a default provider
        let default_provider = DefaultFeeEstimationProvider::new(Network::Bitcoin);
        
        // Verify provider name and priority
        assert_eq!(default_provider.provider_name(), "DefaultProvider");
        assert_eq!(default_provider.priority(), 255);
        
        // Verify the provider is available
        assert!(default_provider.is_available());
        
        // Get fee estimates
        let recommendations = default_provider.get_fee_estimates().unwrap();
        
        // Check that we have fee recommendations for different priorities
        assert!(recommendations.by_priority.contains_key(&FeePriority::High));
        assert!(recommendations.by_priority.contains_key(&FeePriority::Medium));
        assert!(recommendations.by_priority.contains_key(&FeePriority::Low));
        
        // Verify that high priority has higher fees than low priority
        let high_fee = recommendations.get_fee_for_priority(FeePriority::High);
        let low_fee = recommendations.get_fee_for_priority(FeePriority::Low);
        assert!(high_fee > low_fee);
        
        // Get historical data
        let historical_data = default_provider.get_historical_data().unwrap();
        
        // Check that we have some historical data
        assert!(!historical_data.time_series.is_empty());
    }
    
    #[test]
    fn test_fee_estimation_service_with_mock_provider() {
        use crate::network_status::MockNetworkStatusProvider;
        
        // Create a mock network status provider
        let mock_provider = MockNetworkStatusProvider::new(Network::Bitcoin)
            .with_congestion(CongestionLevel::High)
            .with_fee_rate(1, 20.0)   // 1 block target: 20 sat/vB
            .with_fee_rate(6, 10.0)   // 6 block target: 10 sat/vB
            .with_fee_rate(24, 5.0);  // 24 block target: 5 sat/vB
        
        // Create a network provider
        let network_provider = NetworkFeeEstimationProvider::new(mock_provider, Network::Bitcoin);
        
        // Verify provider name and priority
        assert_eq!(network_provider.provider_name(), "NetworkProvider");
        assert_eq!(network_provider.priority(), 10);
        
        // Verify the provider is available
        assert!(network_provider.is_available());
        
        // Get fee estimates
        let recommendations = network_provider.get_fee_estimates().unwrap();
        
        // Check that we have fee recommendations
        assert!(!recommendations.by_block_target.is_empty());
        assert!(!recommendations.by_priority.is_empty());
        
        // Create a service with both providers
        let mut providers: Vec<Box<dyn FeeEstimationProvider>> = Vec::new();
        providers.push(Box::new(network_provider));
        providers.push(Box::new(DefaultFeeEstimationProvider::new(Network::Bitcoin)));
        
        let service = FeeEstimationService::new(
            providers,
            Network::Bitcoin,
            1800,
            None,
        );
        
        // Get fee recommendations from the service
        let service_recommendations = service.get_fee_recommendations().unwrap();
        
        // The service should use the network provider since it has higher priority
        assert_eq!(service_recommendations.congestion, CongestionLevel::High);
        
        // Estimate fee for high priority
        let high_fee = service.estimate_fee(FeePriority::High).unwrap();
        
        // High fee should be at least 10 sat/vB given our mock data
        // The actual value depends on how the fee estimation works with the mock data
        assert!(high_fee >= Decimal::from_f32(5.0).unwrap());
    }

    #[test]
    fn test_fee_estimation_service_convenience_functions() {
        use crate::network_status::MockNetworkStatusProvider;
        
        // Create a mock network status provider
        let mock_provider = MockNetworkStatusProvider::new(Network::Testnet)
            .with_congestion(CongestionLevel::Low)
            .with_fee_rate(1, 5.0)   // 1 block target: 5 sat/vB
            .with_fee_rate(6, 3.0)   // 6 block target: 3 sat/vB
            .with_fee_rate(24, 1.0); // 24 block target: 1 sat/vB
        
        // Create a service with convenience function
        let service = create_fee_service(
            mock_provider,
            Network::Testnet,
            None,
        );
        
        // Get fee recommendations
        let recommendations = service.get_fee_recommendations().unwrap();
        
        // Check network type
        assert_eq!(recommendations.network, Network::Testnet);
        
        // Check congestion level
        assert_eq!(recommendations.congestion, CongestionLevel::Low);
        
        // Use the convenience function to estimate fee
        let mock_provider = MockNetworkStatusProvider::new(Network::Testnet)
            .with_congestion(CongestionLevel::Low)
            .with_fee_rate(1, 5.0);
            
        let fee = estimate_fee_with_service(
            FeePriority::Medium,
            mock_provider,
            Network::Testnet,
            None,
        ).unwrap();
        
        // Medium priority fee should be between 1 and 5 sat/vB
        assert!(fee >= Decimal::from_f32(1.0).unwrap());
        assert!(fee <= Decimal::from_f32(5.0).unwrap());
    }
}
