use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::fs;
use std::path::Path;
use std::env;

// A diagnostic test to check for environment issues
#[test]
fn test_environment_variables() {
    println!("--- Environment Variables ---");
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }
    println!("---------------------------");
}

// Test file system access
#[test]
fn test_filesystem_access() {
    println!("--- File System Test ---");
    
    // Get current directory
    let current_dir = env::current_dir().unwrap();
    println!("Current directory: {:?}", current_dir);
    
    // Check if we can create and write to a temporary file
    let temp_path = current_dir.join("test_temp.txt");
    println!("Attempting to write to: {:?}", temp_path);
    
    match fs::write(&temp_path, b"test content") {
        Ok(_) => println!("Successfully wrote to file"),
        Err(e) => println!("Failed to write to file: {}", e),
    }
    
    // Clean up
    if temp_path.exists() {
        match fs::remove_file(&temp_path) {
            Ok(_) => println!("Successfully cleaned up file"),
            Err(e) => println!("Failed to clean up file: {}", e),
        }
    }
    
    println!("-------------------------");
}

// Test for potential threading issues
#[test]
fn test_threading() {
    println!("--- Threading Test ---");
    
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    // Create 10 threads that increment a counter
    for i in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            println!("Thread {} started", i);
            // Simulate some work
            thread::sleep(Duration::from_millis(10));
            
            // Lock the mutex and increment the counter
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
            
            println!("Thread {} finished, counter now {}", i, *num);
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Check final counter value
    let final_count = *counter.lock().unwrap();
    println!("Final counter value: {}", final_count);
    assert_eq!(final_count, 10);
    
    println!("----------------------");
}

// Test memory allocation
#[test]
fn test_memory_allocation() {
    println!("--- Memory Test ---");
    
    // Allocate some memory
    let mut big_vec = Vec::with_capacity(1_000_000);
    for i in 0..1_000_000 {
        big_vec.push(i);
    }
    
    println!("Successfully allocated 1,000,000 integers");
    println!("-------------------");
}

// Test resource cleanup
#[test]
fn test_resource_cleanup() {
    println!("--- Resource Cleanup Test ---");
    
    // Create a temp directory
    let temp_dir = env::temp_dir().join("bitvault_test");
    
    if !temp_dir.exists() {
        match fs::create_dir(&temp_dir) {
            Ok(_) => println!("Created directory: {:?}", temp_dir),
            Err(e) => println!("Failed to create directory: {}", e),
        }
    }
    
    // Create some files
    for i in 0..5 {
        let file_path = temp_dir.join(format!("test_file_{}.txt", i));
        match fs::write(&file_path, format!("Test content {}", i)) {
            Ok(_) => println!("Created file: {:?}", file_path),
            Err(e) => println!("Failed to create file: {}", e),
        }
    }
    
    // List files
    match fs::read_dir(&temp_dir) {
        Ok(entries) => {
            println!("Files in directory:");
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  {:?}", entry.path());
                }
            }
        },
        Err(e) => println!("Failed to read directory: {}", e),
    }
    
    // Clean up
    match fs::remove_dir_all(&temp_dir) {
        Ok(_) => println!("Successfully cleaned up directory"),
        Err(e) => println!("Failed to clean up directory: {}", e),
    }
    
    println!("---------------------------");
} 