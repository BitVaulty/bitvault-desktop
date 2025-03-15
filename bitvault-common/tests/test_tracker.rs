// Import the universal logger
mod universal_logger;
use universal_logger::{log_info, log_error, log_test_start, log_test_end};

// Create a macro that wraps tests to log their execution
#[macro_export]
macro_rules! tracked_test {
    ($(#[$attr:meta])* $name:ident, $body:expr) => {
        $(#[$attr])*
        #[test]
        fn $name() {
            let test_name = stringify!($name);
            $crate::universal_logger::log_test_start(test_name);
            
            let result = std::panic::catch_unwind(|| {
                $body();
            });
            
            match result {
                Ok(_) => {
                    $crate::universal_logger::log_test_end(test_name, true);
                }
                Err(e) => {
                    let panic_msg = if let Some(s) = e.downcast_ref::<&str>() {
                        format!("Panic: {}", s)
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        format!("Panic: {}", s)
                    } else {
                        "Unknown panic".to_string()
                    };
                    
                    $crate::universal_logger::log_error(&format!("Test panic: {}", panic_msg));
                    $crate::universal_logger::log_test_end(test_name, false);
                    
                    // Re-panic to ensure test is marked as failed
                    std::panic::resume_unwind(e);
                }
            }
        }
    };
}

// Example of using the macro
#[cfg(test)]
mod tests {
    use super::*;
    
    tracked_test!(basic_pass, {
        log_info("This test should pass");
        assert_eq!(2 + 2, 4);
    });
    
    // Uncomment to test failure tracking
    /*
    tracked_test!(basic_fail, {
        log_info("This test should fail");
        assert_eq!(2 + 2, 5);
    });
    */
} 