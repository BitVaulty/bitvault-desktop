use std::sync::Once;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

// Static initialization for test module
static INIT_LOGGER: Once = Once::new();

fn setup() {
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });
}

#[test]
fn test_minimal() {
    setup();
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_just_file_creation() {
    println!("Starting minimal file test");
    
    // Create a temporary directory
    let dir = tempdir().expect("Failed to create temp directory");
    let file_path = dir.path().join("minimal_test.dat");
    let file_path_str = file_path.to_str().unwrap();
    println!("Test file path: {}", file_path_str);
    
    // Write a simple file
    let data = b"test data";
    fs::write(&file_path, data).expect("Failed to write test file");
    println!("Wrote test file");
    
    // Verify file exists
    assert!(Path::new(file_path_str).exists(), "File should exist");
    println!("File exists check passed");
    
    // Read it back
    let read_data = fs::read(&file_path).expect("Failed to read test file");
    println!("Read back file");
    
    // Verify contents
    assert_eq!(read_data, data, "File contents should match");
    println!("File contents match");
    
    // Clean up happens automatically when dir goes out of scope
    println!("Minimal file test passed");
} 