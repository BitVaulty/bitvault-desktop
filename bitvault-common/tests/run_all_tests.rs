use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, ExitStatus};
use std::time::Instant;
use chrono::prelude::*;
use serde::Serialize;

// Import all the tests here
#[path = "test_helpers.rs"]
mod test_helpers;
use test_helpers::{log_info, log_error, log_test_start, log_test_end};

// List of test files to run (excluding this one)
const TEST_FILES: [&str; 11] = [
    "utxo_comprehensive_test",
    "key_management_tests",
    "key_management_consolidated_test",
    "key_security_test",
    "platform_tests",
    "platform_consolidated_test",
    "transaction_tests",
    "fee_estimation_tests",
    "math_tests",
    "memory_security_test",
    "simple_memory_test"
];

// Structure to hold test results for AI analysis
#[derive(Serialize, Clone)]
struct TestResult {
    name: String,
    passed: bool,
    duration: std::time::Duration,
    log_file: String,
    error_message: Option<String>,
}

// Main function to run all tests
fn main() {
    // Check if we're in fallback mode
    let fallback_mode = env::var("BITVAULT_TEST_FALLBACK").is_ok();
    
    // Setup logging
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Print startup message
    println!("Running all tests in bitvault-common...");
    log_info("Starting test suite execution");
    
    // Log environment variables
    if let Ok(backtrace) = env::var("RUST_BACKTRACE") {
        log_info(&format!("Running with RUST_BACKTRACE={}", backtrace));
    }
    
    // Record start time
    let start_time = Instant::now();
    let start_datetime = Local::now();
    log_info(&format!("Test suite started at: {}", start_datetime.format("%Y-%m-%d %H:%M:%S")));
    
    // Determine log directory
    let log_dir = match env::var("BITVAULT_TEST_LOG_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => PathBuf::from("/tmp/bitvault_test_output"),
    };
    
    // Ensure the log directory exists
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    }
    
    // Run each test
    let mut successful_tests = 0;
    let mut failed_tests = Vec::new();
    let mut test_results = Vec::new();
    
    for test_file in TEST_FILES.iter() {
        log_test_start(&format!("Running {}", test_file));
        
        // Add documentation about consolidated tests
        if test_file == &"utxo_comprehensive_test" {
            log_info("Note: utxo_comprehensive_test consolidates functionality from multiple UTXO tests and demonstrates file-based output.");
        } else if test_file == &"platform_consolidated_test" {
            log_info("Note: platform_consolidated_test consolidates functionality from several platform test files including minimal and simple tests.");
        } else if test_file == &"key_management_consolidated_test" {
            log_info("Note: key_management_consolidated_test consolidates minimal and simple key tests with file-based output.");
        } else if test_file == &"key_security_test" {
            log_info("Note: key_security_test focuses on security aspects of key management including memory protection and key rotation.");
        }
        
        // Build the command to run the test
        let cargo_test_cmd = format!("cargo test --test {}", test_file);
        println!("Executing: {}", cargo_test_cmd);
        
        // Record test start time
        let test_start_time = Instant::now();
        
        // Run the test command
        let output = Command::new("cargo")
            .args(&["test", "--test", test_file])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        
        // Calculate test duration
        let test_duration = test_start_time.elapsed();
        
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // Write output to log file
                let log_file = format!("/tmp/bitvault_test_{}.log", test_file);
                let mut file = File::create(&log_file).expect("Failed to create log file");
                file.write_all(stdout.as_bytes()).expect("Failed to write to log file");
                file.write_all(stderr.as_bytes()).expect("Failed to write to log file");
                
                // Also write to the structured log directory
                let structured_log_file = log_dir.join(format!("{}.log", test_file));
                let mut structured_file = File::create(&structured_log_file)
                    .expect("Failed to create structured log file");
                structured_file.write_all(stdout.as_bytes()).expect("Failed to write to structured log file");
                structured_file.write_all(stderr.as_bytes()).expect("Failed to write to structured log file");
                
                // Check if the test was successful
                if output.status.success() {
                    println!("✓ {} passed", test_file);
                    log_info(&format!("Test {} passed", test_file));
                    successful_tests += 1;
                    
                    // Record success
                    test_results.push(TestResult {
                        name: test_file.to_string(),
                        passed: true,
                        duration: test_duration,
                        log_file: structured_log_file.to_string_lossy().to_string(),
                        error_message: None,
                    });
                } else {
                    println!("✗ {} failed", test_file);
                    log_error(&format!("Test {} failed: {}", test_file, stderr));
                    failed_tests.push(test_file.to_string());
                    
                    // Record failure
                    test_results.push(TestResult {
                        name: test_file.to_string(),
                        passed: false,
                        duration: test_duration,
                        log_file: structured_log_file.to_string_lossy().to_string(),
                        error_message: Some(stderr.to_string()),
                    });
                }
            },
            Err(e) => {
                println!("✗ {} failed to execute: {}", test_file, e);
                log_error(&format!("Test {} failed to execute: {}", test_file, e));
                failed_tests.push(test_file.to_string());
                
                // Record execution failure
                test_results.push(TestResult {
                    name: test_file.to_string(),
                    passed: false,
                    duration: test_duration,
                    log_file: "".to_string(),
                    error_message: Some(format!("Failed to execute: {}", e)),
                });
            }
        }
        
        // Check for file-based test results
        if test_file == &"utxo_comprehensive_test" || test_file == &"platform_consolidated_test" || test_file == &"key_management_consolidated_test" {
            let output_dir = Path::new("/tmp/bitvault_test_output");
            if output_dir.exists() {
                match fs::read_dir(output_dir) {
                    Ok(entries) => {
                        let test_specific_entries: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| {
                                if let Some(name) = e.file_name().to_str() {
                                    let test_prefix = match *test_file {
                                        "utxo_comprehensive_test" => "utxo_",
                                        "platform_consolidated_test" => "platform_",
                                        "key_management_consolidated_test" => "key_",
                                        _ => "",
                                    };
                                    name.starts_with(test_prefix)
                                } else {
                                    false
                                }
                            })
                            .collect();
                        
                        log_info(&format!("Found {} file-based test results for {}", 
                            test_specific_entries.len(), test_file));
                        
                        // Copy these test result files to the structured log directory
                        for entry in test_specific_entries {
                            let file_name = entry.file_name();
                            let destination = log_dir.join(&file_name);
                            
                            if let Err(e) = fs::copy(entry.path(), &destination) {
                                log_error(&format!("Failed to copy test result file: {}", e));
                            } else {
                                log_info(&format!("Copied test result: {} to {}", 
                                    file_name.to_string_lossy(), destination.to_string_lossy()));
                            }
                        }
                    },
                    Err(e) => {
                        log_error(&format!("Failed to read test output directory: {}", e));
                    }
                }
            } else {
                log_info("No file-based test results found");
            }
        }
        
        log_test_end(&format!("Finished {}", test_file), true);
    }
    
    // Calculate elapsed time
    let elapsed = start_time.elapsed();
    let end_datetime = Local::now();
    
    // Print summary
    println!("\nTest Suite Summary:");
    println!("------------------");
    println!("Started at: {}", start_datetime.format("%Y-%m-%d %H:%M:%S"));
    println!("Finished at: {}", end_datetime.format("%Y-%m-%d %H:%M:%S"));
    println!("Elapsed time: {:.2} seconds", elapsed.as_secs_f64());
    println!("Tests passed: {}/{}", successful_tests, TEST_FILES.len());
    
    if !failed_tests.is_empty() {
        println!("Failed tests:");
        for test in &failed_tests {
            println!("  - {}", test);
        }
    }
    
    // Log summary
    log_info(&format!("Test suite finished. Tests passed: {}/{}. Elapsed time: {:.2} seconds",
        successful_tests, TEST_FILES.len(), elapsed.as_secs_f64()));
    
    if !failed_tests.is_empty() {
        log_error(&format!("Failed tests: {}", failed_tests.join(", ")));
    }
    
    // Write a structured summary file
    write_structured_summary(&log_dir, &start_datetime, &end_datetime, elapsed, 
                            successful_tests, TEST_FILES.len(), &failed_tests);
    
    // Write AI-friendly output in JSON format
    write_ai_friendly_output(&log_dir, &test_results);
    
    // Print locations of log files
    log_info(&format!("Test log file: /tmp/bitvault_test_results.log"));
    log_info(&format!("Structured test results directory: {}", log_dir.to_string_lossy()));
    
    // Exit with error code if any tests failed
    if !failed_tests.is_empty() {
        std::process::exit(1);
    }
}

// Write a structured summary file
fn write_structured_summary(
    log_dir: &Path, 
    start_time: &DateTime<Local>,
    end_time: &DateTime<Local>,
    elapsed: std::time::Duration,
    passed: usize,
    total: usize,
    failed_tests: &[String]
) {
    let summary_path = log_dir.join("test_summary.log");
    let mut summary_file = File::create(&summary_path)
        .expect("Failed to create summary file");
    
    writeln!(summary_file, "BitVault Test Suite Summary").unwrap();
    writeln!(summary_file, "==========================").unwrap();
    writeln!(summary_file, "").unwrap();
    writeln!(summary_file, "Started at: {}", start_time.format("%Y-%m-%d %H:%M:%S")).unwrap();
    writeln!(summary_file, "Finished at: {}", end_time.format("%Y-%m-%d %H:%M:%S")).unwrap();
    writeln!(summary_file, "Elapsed time: {:.2} seconds", elapsed.as_secs_f64()).unwrap();
    writeln!(summary_file, "").unwrap();
    writeln!(summary_file, "Tests passed: {}/{}", passed, total).unwrap();
    writeln!(summary_file, "Success rate: {:.1}%", (passed as f64 / total as f64) * 100.0).unwrap();
    
    if !failed_tests.is_empty() {
        writeln!(summary_file, "").unwrap();
        writeln!(summary_file, "Failed tests:").unwrap();
        for test in failed_tests {
            writeln!(summary_file, "  - {}", test).unwrap();
        }
    }
}

// Write JSON-formatted output for AI tools
fn write_ai_friendly_output(log_dir: &Path, results: &[TestResult]) {
    #[derive(Serialize)]
    struct AiTestOutput {
        timestamp: String,
        total_tests: usize,
        passed_tests: usize,
        failed_tests: Vec<String>,
        test_details: Vec<TestResult>,
    }
    
    let passed_count = results.iter().filter(|r| r.passed).count();
    let failed_tests: Vec<String> = results.iter()
        .filter(|r| !r.passed)
        .map(|r| r.name.clone())
        .collect();
    
    let output = AiTestOutput {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        total_tests: results.len(),
        passed_tests: passed_count,
        failed_tests,
        test_details: results.to_vec(),
    };
    
    let json_path = log_dir.join("test_results.json");
    let json_str = serde_json::to_string_pretty(&output)
        .expect("Failed to serialize test results to JSON");
    
    fs::write(json_path, json_str).expect("Failed to write JSON results");
} 