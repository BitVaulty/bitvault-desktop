// Test for the math module

use bitvault_common::math;

#[test]
fn test_is_dust_amount() {
    // Test that dust amount detection works correctly
    assert!(math::is_dust_amount(546));
    assert!(!math::is_dust_amount(547));
} 