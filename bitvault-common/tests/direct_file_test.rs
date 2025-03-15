use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::fs;

#[test]
fn test_direct_file_write() {
    // Write to a file directly
    let log_path = "/tmp/bitvault_direct_test.log";
    
    // Open file for writing
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Failed to open log file");
    
    // Write some content
    writeln!(file, "Direct test log entry").expect("Failed to write to log file");
    writeln!(file, "Another log entry").expect("Failed to write to log file");
    
    // Ensure file is written
    file.flush().expect("Failed to flush file");
    
    // Verify file exists and has content
    assert!(Path::new(log_path).exists());
    
    // Read the file content
    let content = fs::read_to_string(log_path).expect("Failed to read log file");
    
    // Check that the file contains what we wrote
    assert!(content.contains("Direct test log entry"));
    assert!(content.contains("Another log entry"));
    
    // Print the file content to stdout
    println!("Log file contents:\n{}", content);
} 