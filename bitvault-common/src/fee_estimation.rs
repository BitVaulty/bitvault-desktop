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

/// Estimate fee based on priority and network provider
/// 
/// # Arguments
/// * `priority` - Fee priority level
/// * `provider` - Network status provider implementation
/// 
/// # Returns
/// * Estimated fee rate in sat/vB as Decimal, or error
pub fn estimate_fee<T: NetworkStatusProvider>(
    priority: FeePriority, 
    provider: &T
) -> Result<Decimal, WalletError> {
    // Get network status from provider
    let status = provider.get_network_status()
        .map_err(|e| WalletError::Generic(format!("Failed to get network status: {}", e)))?;
    
    // If custom rate specified, just return that with validation
    if let FeePriority::Custom(rate) = priority {
        let fee = Decimal::from_f32(rate).unwrap_or(dec!(1.0));
        let min_reasonable = defaults::min_reasonable_fee_rate(status.network);
        let max_reasonable = defaults::max_reasonable_fee_rate(status.network);
        
        if fee < min_reasonable {
            log::warn!("Custom fee rate {} too low, using minimum {}", fee, min_reasonable);
            return Ok(min_reasonable);
        }
        
        if fee > max_reasonable {
            log::warn!("Custom fee rate {} too high, capping at {}", fee, max_reasonable);
            return Ok(max_reasonable);
        }
        
        return Ok(fee);
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
                
                if provider_fee < min_reasonable {
                    log::warn!("Provider fee rate {} too low, using minimum {}", provider_fee, min_reasonable);
                    return Ok(min_reasonable);
                }
                
                if provider_fee > max_reasonable {
                    log::warn!("Provider fee rate {} too high, capping at {}", provider_fee, max_reasonable);
                    return Ok(max_reasonable);
                }
                
                return Ok(provider_fee);
            }
        }
    }
    
    // If no provider estimates available, use defaults
    Ok(fee_rate)
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
