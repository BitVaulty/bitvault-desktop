use std::fs;
use std::path::Path;
use std::env;
use std::process::Command;

#[test]
fn test_write_to_tmp() {
    // Write to a file in /tmp
    let log_path = "/tmp/bitvault_test_diagnostic.log";
    
    // Get system information
    let mut content = String::new();
    content.push_str("=== BitVault Test Diagnostic Log ===\n\n");
    
    // Current directory
    if let Ok(dir) = env::current_dir() {
        content.push_str(&format!("Current directory: {:?}\n", dir));
    } else {
        content.push_str("Failed to get current directory\n");
    }
    
    // Environment variables
    content.push_str("\n=== Environment Variables ===\n");
    for (key, value) in env::vars() {
        content.push_str(&format!("{}: {}\n", key, value));
    }
    
    // System info
    content.push_str("\n=== System Info ===\n");
    if let Ok(output) = Command::new("uname").arg("-a").output() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            content.push_str(&format!("uname -a: {}\n", output_str));
        }
    }
    
    // Write permissions
    content.push_str("\n=== Write Permissions ===\n");
    let temp_dirs = vec![
        "/tmp",
        "/home/acolyte/BitVault/BitVaultWallet",
        "/home/acolyte/BitVault/BitVaultWallet/bitvault-common",
        "/home/acolyte/BitVault/BitVaultWallet/bitvault-common/tests",
    ];
    
    for dir in temp_dirs {
        let test_file = format!("{}/test_write_perm.tmp", dir);
        match fs::write(&test_file, "test") {
            Ok(_) => {
                content.push_str(&format!("Can write to {}: Yes\n", dir));
                // Clean up
                let _ = fs::remove_file(&test_file);
            },
            Err(e) => {
                content.push_str(&format!("Can write to {}: No - {}\n", dir, e));
            }
        }
    }
    
    // Write to the log file
    match fs::write(log_path, content) {
        Ok(_) => println!("Diagnostic log written to {}", log_path),
        Err(e) => println!("Failed to write diagnostic log: {}", e),
    }
    
    // This assertion should always pass
    assert!(true);
} 