use std::sync::{Mutex, Once, MutexGuard};
use log::{info, warn, error, LevelFilter};
use lazy_static::lazy_static;
use bitcoin::Amount;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::env;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::thread;
use std::sync::Arc;

// Initialize logging once
static INIT: Once = Once::new();

// Mutex to ensure tests run one at a time
lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

// Get project-relative test output paths
fn get_project_test_dir() -> PathBuf {
    let mut project_dir = env::current_dir().expect("Failed to get current directory");
    // Create test_results directory within the project
    project_dir.push("test_results");
    project_dir
}

// Constants for test output - using project-relative paths
lazy_static! {
    static ref TEST_LOG_PATH: PathBuf = {
        let mut path = get_project_test_dir();
        path.push("bitvault_test_results.log");
        path
    };
    
    static ref TEST_OUTPUT_PATH: PathBuf = {
        get_project_test_dir()
    };
}

// Initialize once
static LOGGER_INIT: Once = Once::new();
static mut TEST_LOGGER: Option<Arc<Mutex<FileLogger>>> = None;

/// Initialize logging for tests
pub fn init_test_logging() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .init();
        println!("Test logging initialized");
    });
}

/// Run a test in a serialized manner to prevent race conditions
pub fn run_serialized_test<F, R>(test_name: &str, test_fn: F) -> R
where
    F: FnOnce() -> R,
{
    println!("Starting serialized test: {}", test_name);
    
    // Acquire lock to ensure exclusive access
    let _guard = TEST_MUTEX.lock().unwrap();
    
    // Initialize logging
    init_test_logging();
    
    // Log test start
    log_test_start(test_name);
    
    // Run the test
    let result = test_fn();
    
    // Log test end
    log_test_end(test_name, true);
    
    result
}

/// Run a test with retries to handle intermittent failures
pub fn run_with_retries<F>(max_retries: usize, test_fn: F) -> Result<(), String>
where
    F: Fn() -> Result<(), String>,
{
    let mut last_error = String::new();
    
    for attempt in 1..=max_retries {
        match test_fn() {
            Ok(()) => {
                println!("Test succeeded on attempt {}", attempt);
                return Ok(());
            }
            Err(e) => {
                println!("Test failed on attempt {}: {}", attempt, e);
                last_error = e;
                
                // Wait a bit before retrying
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
    
    Err(format!("Test failed after {} attempts. Last error: {}", max_retries, last_error))
}

/// Helper function to execute a test in a serialized manner
/// Ensures that the test doesn't run concurrently with other serialized tests
pub fn run_serialized_test_with_name<F, R>(test_name: &str, test_func: F) -> R
where
    F: FnOnce() -> R,
{
    init_test_logging();
    info!("Starting serialized test: {}", test_name);
    
    // Acquire the lock to ensure exclusive execution
    let _guard = serialize_test();
    
    // Run the actual test
    let result = test_func();
    
    info!("Completed serialized test: {}", test_name);
    result
}

/// Helper function to delay execution for a given number of milliseconds
/// Useful for tests that need to wait for certain conditions
#[allow(dead_code)]
pub fn delay_ms(milliseconds: u64) {
    std::thread::sleep(std::time::Duration::from_millis(milliseconds));
}

/// Executes a test with retries if it fails
/// Useful for tests that may fail intermittently
#[allow(dead_code)]
pub fn run_with_retries_with_name<F, R, E>(test_name: &str, retries: usize, test_func: F) -> Result<R, E>
where
    F: Fn() -> Result<R, E>,
    E: std::fmt::Display,
{
    init_test_logging();
    info!("Starting test with retries: {}", test_name);
    
    let mut last_error = None;
    
    for attempt in 0..=retries {
        if attempt > 0 {
            info!("Retry attempt {} for test: {}", attempt, test_name);
            // Add a small delay between retries
            delay_ms(100 * attempt as u64);
        }
        
        match test_func() {
            Ok(result) => {
                info!("Test completed successfully on attempt {}: {}", attempt + 1, test_name);
                return Ok(result);
            }
            Err(e) => {
                warn!("Test failed on attempt {}: {} - Error: {}", attempt + 1, test_name, e);
                last_error = Some(e);
                continue;
            }
        }
    }
    
    // If we get here, all retries failed
    error!("All {} retry attempts failed for test: {}", retries, test_name);
    Err(last_error.unwrap())
}

/// Get a lock on the test mutex to serialize test execution
/// This can help prevent race conditions in tests
pub fn serialize_test() -> MutexGuard<'static, ()> {
    TEST_MUTEX.lock().unwrap()
}

/// Checks if two amounts are approximately equal within a tolerance
/// 
/// This is useful for tests where rounding errors might occur due to fee calculations
pub fn assert_amounts_approximately_equal(a: Amount, b: Amount, tolerance_sats: u64) {
    let a_sats = a.to_sat();
    let b_sats = b.to_sat();
    
    let diff = if a_sats > b_sats {
        a_sats - b_sats
    } else {
        b_sats - a_sats
    };
    
    if diff > tolerance_sats {
        panic!("Amounts differ by {} sats, which exceeds tolerance of {} sats: {} vs {}", 
               diff, tolerance_sats, a_sats, b_sats);
    }
}

/// Checks if the balance equation holds: total_selected = target + fee + change
/// 
/// This is useful for verifying that UTXO selection correctly accounts for all funds
pub fn assert_balance_equation(total_selected: Amount, target: Amount, fee: Amount, change: Amount, tolerance_sats: u64) {
    let left_side = total_selected.to_sat();
    let right_side = target.to_sat() + fee.to_sat() + change.to_sat();
    
    let diff = if left_side > right_side {
        left_side - right_side
    } else {
        right_side - left_side
    };
    
    if diff > tolerance_sats {
        panic!("Balance equation doesn't hold within tolerance of {} sats: total_selected ({}) != target ({}) + fee ({}) + change ({}), diff = {}",
               tolerance_sats, left_side, target.to_sat(), fee.to_sat(), change.to_sat(), diff);
    }
}

// File-based logger for tests
pub struct FileLogger {
    file: File,
    test_start_time: Instant,
}

impl FileLogger {
    // Create a new logger
    fn new() -> Self {
        // Create the test output directory if it doesn't exist
        let path = TEST_OUTPUT_PATH.as_path();
        let _ = fs::create_dir_all(path);
        
        // Open the log file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(TEST_LOG_PATH.as_path())
            .expect("Failed to open test log file");
        
        FileLogger {
            file,
            test_start_time: Instant::now(),
        }
    }
    
    // Log a message to the file
    fn log(&mut self, level: &str, message: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let elapsed = self.test_start_time.elapsed().as_secs_f64();
        
        let thread_id = format!("{:?}", thread::current().id());
        
        let log_line = format!(
            "[{} +{:.6}s] [{}] [{}]: {}\n",
            timestamp,
            elapsed,
            thread_id,
            level,
            message
        );
        
        let _ = self.file.write_all(log_line.as_bytes());
        let _ = self.file.flush();
    }
    
    // Write test results to a separate file
    fn write_test_result(&mut self, test_name: &str, result: &str) -> std::io::Result<()> {
        let mut result_path = TEST_OUTPUT_PATH.clone();
        result_path.push(format!("{}_result.txt", test_name));
        
        let mut result_file = File::create(&result_path)?;
        writeln!(result_file, "{}", result)?;
        
        self.log("INFO", &format!("Test result written to {}", result_path.display()));
        Ok(())
    }
}

// Get or initialize the test logger
pub fn get_test_logger() -> Arc<Mutex<FileLogger>> {
    unsafe {
        LOGGER_INIT.call_once(|| {
            TEST_LOGGER = Some(Arc::new(Mutex::new(FileLogger::new())));
        });
        
        TEST_LOGGER.clone().unwrap()
    }
}

// Public logging functions
pub fn log_info(message: &str) {
    if let Ok(mut logger) = get_test_logger().lock() {
        logger.log("INFO", message);
    }
}

pub fn log_error(message: &str) {
    if let Ok(mut logger) = get_test_logger().lock() {
        logger.log("ERROR", message);
    }
}

pub fn log_test_start(test_name: &str) {
    if let Ok(mut logger) = get_test_logger().lock() {
        logger.log("TEST", &format!("=== Starting test: {} ===", test_name));
    }
}

pub fn log_test_end(test_name: &str, success: bool) {
    let status = if success { "PASSED" } else { "FAILED" };
    if let Ok(mut logger) = get_test_logger().lock() {
        logger.log("TEST", &format!("=== Test {} {} ===", test_name, status));
    }
}

// Write test output to a file that can be read later
pub fn write_test_output(test_name: &str, output: &str) -> std::io::Result<()> {
    let mut output_path = TEST_OUTPUT_PATH.clone();
    output_path.push(format!("{}_output.txt", test_name));
    
    let mut output_file = File::create(&output_path)?;
    writeln!(output_file, "{}", output)?;
    
    log_info(&format!("Test output written to {}", output_path.display()));
    Ok(())
}

// Read test output from a file
pub fn read_test_output(test_name: &str) -> std::io::Result<String> {
    let mut output_path = TEST_OUTPUT_PATH.clone();
    output_path.push(format!("{}_output.txt", test_name));
    
    fs::read_to_string(&output_path)
}

// Check if test output exists
pub fn test_output_exists(test_name: &str) -> bool {
    let mut output_path = TEST_OUTPUT_PATH.clone();
    output_path.push(format!("{}_output.txt", test_name));
    
    output_path.exists()
}

// Custom TestLogger for our tests
pub struct TestLogger {
    file: File
}

impl TestLogger {
    pub fn new(test_name: &str) -> Self {
        // Create the output directory if it doesn't exist
        let path = TEST_OUTPUT_PATH.as_path();
        let _ = fs::create_dir_all(path);
        
        let mut log_path = TEST_OUTPUT_PATH.clone();
        log_path.push(format!("{}.log", test_name));
        
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
            .expect("Failed to create test-specific log file");
            
        TestLogger { file }
    }
    
    pub fn log_amount(&mut self, label: &str, amount: Amount) {
        writeln!(self.file, "{}: {} satoshis", label, amount.to_sat()).unwrap();
    }
    
    pub fn log_success(&mut self) {
        writeln!(self.file, "TEST SUCCEEDED").unwrap();
    }
    
    pub fn log_failure(&mut self, message: &str) {
        writeln!(self.file, "TEST FAILED: {}", message).unwrap();
    }
}

impl Write for TestLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

// Create a dedicated logger for a specific test
pub fn create_test_logger(test_name: &str) -> TestLogger {
    TestLogger::new(test_name)
}

// Test the test logger itself
#[test]
fn test_logger_works() {
    log_test_start("test_logger_works");
    
    log_info("Testing the logger");
    log_error("This is an error message");
    
    // Also write directly to the log file to verify it exists
    let mut test_file = TEST_OUTPUT_PATH.clone();
    test_file.push("logger_test_verify.txt");
    
    fs::write(&test_file, "Logger test verification").unwrap();
    
    log_info(&format!("Wrote verification file to {}", test_file.display()));
    log_test_end("test_logger_works", true);
} 