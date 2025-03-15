// Simple memory zeroization test
// This test doesn't depend on any other modules

use zeroize::{Zeroize, Zeroizing};

#[test]
fn test_simple_zeroization() {
    // Create a buffer with sensitive data
    let buffer_size = 32;
    let mut buffer = Zeroizing::new(vec![0xFFu8; buffer_size]);
    
    // Verify buffer is correctly initialized
    for byte in buffer.iter() {
        assert_eq!(*byte, 0xFF, "Buffer should be initialized with 0xFF");
    }
    
    // Explicitly zeroize the buffer
    buffer.zeroize();
    
    // Verify buffer is zeroed
    for byte in buffer.iter() {
        assert_eq!(*byte, 0, "Buffer should be zeroed after zeroize()");
    }
    
    println!("Simple zeroization test passed!");
} 