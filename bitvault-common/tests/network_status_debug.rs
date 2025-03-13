use bitvault_common::network_status::NetworkStatus;
use bitcoin::Network;
use rust_decimal_macros::dec;

fn main() {
    println!("Running network_status debug test");
    
    // Recreate the scenario from the test
    let mut network_status = NetworkStatus::new(Network::Bitcoin);
    
    // Add some fee estimates with Decimal values
    network_status.fee_estimates.insert(1, dec!(20.0));
    network_status.fee_estimates.insert(3, dec!(10.0));
    network_status.fee_estimates.insert(6, dec!(5.0));
    
    println!("Fee estimates: {:?}", network_status.fee_estimates);
    
    // Test closest match - this is where the problem is
    let target_2 = network_status.get_recommended_fee_rate(2);
    println!("Result for target 2: {:?} (expected: Some(10.0))", target_2);
    
    // Let's try to understand why it's returning 20.0 instead of 10.0
    // The issue might be in how the comparison is implemented
    
    // Manual check of the algorithm
    println!("\nManual closest target calculation:");
    let mut closest_target = 0u32;
    let mut closest_diff = u32::MAX;
    
    for &target in [1, 3, 6].iter() {
        let diff = (target as i32 - 2).abs() as u32;
        println!("Target: {}, Diff: {}", target, diff);
        
        if diff < closest_diff || (diff == closest_diff && target > closest_target) {
            closest_diff = diff;
            closest_target = target;
            println!("  -> New closest: {} (diff: {})", closest_target, closest_diff);
        }
    }
    
    println!("Manual final closest: {} (expected: 3)", closest_target);
} 