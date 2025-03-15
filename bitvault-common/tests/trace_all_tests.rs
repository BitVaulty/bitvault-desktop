mod universal_logger;
use universal_logger::{log_info, log_error, log_debug};

use std::fs;
use std::process::Command;
use std::path::Path;
use std::env;

// This is a meta-test that runs all the other tests and tracks failures
#[test]
#[ignore] // Ignored by default, run with --include-ignored
fn trace_all_tests() {
    log_info("Starting comprehensive test tracing");
    
    // Get current directory
    let current_dir = env::current_dir().unwrap();
    log_info(&format!("Current directory: {:?}", current_dir));
    
    // Run cargo test with various configurations
    run_test_group("All Tests", &[], false);
    run_test_group("Library Tests", &["--lib"], false);
    run_test_group("Documentation Tests", &["--doc"], false);
    
    // Run individual test modules
    let test_modules = find_test_modules();
    for module in test_modules {
        let test_name = module.file_stem().unwrap().to_string_lossy().to_string();
        run_test_group(&format!("Module: {}", test_name), &["--test", &test_name], false);
    }
    
    log_info("Test tracing complete. Check /tmp/bitvault_test.log for detailed results");
}

// Run a group of tests and log the results
fn run_test_group(group_name: &str, args: &[&str], capture_output: bool) {
    log_info(&format!("Running test group: {}", group_name));
    
    // Prepare the command
    let mut cmd = Command::new("cargo");
    cmd.arg("test")
        .arg("-p")
        .arg("bitvault-common");
    
    // Add any additional args
    for arg in args {
        cmd.arg(arg);
    }
    
    // Add nocapture if requested
    if !capture_output {
        cmd.arg("--")
           .arg("--nocapture");
    }
    
    // Log the command
    log_debug(&format!("Running command: {:?}", cmd));
    
    // Run the command
    match cmd.output() {
        Ok(output) => {
            let status = output.status;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            log_info(&format!("Test group '{}' exit code: {}", group_name, status.code().unwrap_or(-1)));
            
            if !stdout.trim().is_empty() {
                log_debug(&format!("stdout for '{}': {}", group_name, stdout));
            }
            
            if !stderr.trim().is_empty() {
                log_error(&format!("stderr for '{}': {}", group_name, stderr));
            }
            
            // Check for test failures
            if !status.success() {
                log_error(&format!("Test group '{}' FAILED", group_name));
            } else {
                log_info(&format!("Test group '{}' PASSED", group_name));
            }
        },
        Err(e) => {
            log_error(&format!("Failed to run test group '{}': {}", group_name, e));
        }
    }
    
    log_info(&format!("Completed test group: {}", group_name));
}

// Find all test modules in the project
fn find_test_modules() -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    
    // Try to find test modules
    if let Ok(entries) = fs::read_dir("bitvault-common/tests") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                // Skip our utility test files
                let file_name = path.file_name().unwrap().to_string_lossy();
                if !file_name.contains("universal_logger") && 
                   !file_name.contains("test_tracker") && 
                   !file_name.contains("trace_all_tests") {
                    result.push(path);
                }
            }
        }
    }
    
    result
} 