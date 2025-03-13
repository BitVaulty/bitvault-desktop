use std::sync::{Mutex, Once, MutexGuard};
use log::{info, warn, error, LevelFilter};
use lazy_static::lazy_static;

// Initialize logging once
static INIT: Once = Once::new();

// Mutex to ensure tests run one at a time
lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

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
    
    // Run the test
    let result = test_fn();
    
    println!("Completed serialized test: {}", test_name);
    
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