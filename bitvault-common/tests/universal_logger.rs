use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex, Once};
use std::env;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::thread;

// Path to our test log file
pub const TEST_LOG_PATH: &str = "/tmp/bitvault_test.log";

// A simple once-init logger
static INIT: Once = Once::new();
static mut LOGGER: Option<Arc<Mutex<TestLogger>>> = None;

// Simple test logger implementation
pub struct TestLogger {
    file: File,
    test_start_time: Instant,
}

impl TestLogger {
    // Create a new logger
    fn new() -> Self {
        // Clear the log file if it exists
        let _ = fs::remove_file(TEST_LOG_PATH);
        
        // Create a new log file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(TEST_LOG_PATH)
            .expect("Failed to open test log file");
        
        let mut logger = TestLogger {
            file,
            test_start_time: Instant::now(),
        };
        
        // Log the test start with system info
        logger.log_system_info();
        
        logger
    }
    
    // Log message to file
    fn log(&mut self, level: &str, message: &str) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let elapsed = self.test_start_time.elapsed().as_secs_f64();
        
        let thread_id = format!("{:?}", thread::current().id());
        
        let log_line = format!(
            "[{} +{:.6}s] [{}] [{}]: {}\n",
            current_time,
            elapsed,
            thread_id,
            level,
            message
        );
        
        let _ = self.file.write_all(log_line.as_bytes());
        let _ = self.file.flush();
    }
    
    // Log system information
    fn log_system_info(&mut self) {
        let mut system_info = String::new();
        system_info.push_str("=== BitVault Test Started ===\n");
        
        // Current directory
        if let Ok(dir) = env::current_dir() {
            system_info.push_str(&format!("Current directory: {:?}\n", dir));
        }
        
        // Add timestamp
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        system_info.push_str(&format!("Timestamp: {}\n", current_time));
        
        // Write the system info
        let _ = self.file.write_all(system_info.as_bytes());
        let _ = self.file.flush();
    }
}

// Get or initialize the global logger
pub fn get_test_logger() -> Arc<Mutex<TestLogger>> {
    unsafe {
        INIT.call_once(|| {
            LOGGER = Some(Arc::new(Mutex::new(TestLogger::new())));
        });
        
        LOGGER.clone().unwrap()
    }
}

// Convenience function to log at INFO level
pub fn log_info(message: &str) {
    if let Ok(mut logger) = get_test_logger().lock() {
        logger.log("INFO", message);
    }
}

// Convenience function to log at ERROR level
pub fn log_error(message: &str) {
    if let Ok(mut logger) = get_test_logger().lock() {
        logger.log("ERROR", message);
    }
}

// Convenience function to log at DEBUG level
pub fn log_debug(message: &str) {
    if let Ok(mut logger) = get_test_logger().lock() {
        logger.log("DEBUG", message);
    }
}

// Helper to log test start
pub fn log_test_start(test_name: &str) {
    log_info(&format!("TEST START: {}", test_name));
}

// Helper to log test end
pub fn log_test_end(test_name: &str, success: bool) {
    if success {
        log_info(&format!("TEST PASSED: {}", test_name));
    } else {
        log_error(&format!("TEST FAILED: {}", test_name));
    }
}

// Test the logger itself
#[test]
fn test_logger_works() {
    log_test_start("test_logger_works");
    
    log_info("Testing the logger");
    log_debug("This is a debug message");
    log_error("This is an error message");
    
    // Also write directly to the log file to verify it exists
    let test_file = "/tmp/logger_test_verify.txt";
    fs::write(test_file, "Logger test verification").unwrap();
    
    log_info(&format!("Wrote verification file to {}", test_file));
    log_test_end("test_logger_works", true);
} 