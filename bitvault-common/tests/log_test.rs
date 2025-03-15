use log::{debug, error, info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::fs;
use std::path::Path;
use std::sync::Once;

// Setup logging once
static INIT: Once = Once::new();

fn setup_logging() {
    INIT.call_once(|| {
        // Create a file appender
        let log_path = "/tmp/bitvault_test_rust_log.log";
        
        // Remove log file if it exists
        let _ = fs::remove_file(log_path);
        
        let file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build(log_path)
            .unwrap();
        
        // Build a log configuration
        let config = Config::builder()
            .appender(Appender::builder().build("file", Box::new(file_appender)))
            .build(Root::builder().appender("file").build(LevelFilter::Debug))
            .unwrap();
        
        // Initialize the logger
        log4rs::init_config(config).unwrap();
        
        info!("Logging initialized");
    });
}

#[test]
fn test_logging() {
    setup_logging();
    
    info!("This is an info log");
    debug!("This is a debug log");
    error!("This is an error log");
    
    // Verify log file exists
    assert!(Path::new("/tmp/bitvault_test_rust_log.log").exists());
    
    // Print the contents of the log file
    match fs::read_to_string("/tmp/bitvault_test_rust_log.log") {
        Ok(contents) => println!("Log file contents:\n{}", contents),
        Err(e) => println!("Failed to read log file: {}", e),
    }
}

// Add a test to run each major component
#[test]
fn test_each_component() {
    setup_logging();
    
    info!("Running test_each_component");
    
    // Test Config component
    info!("Testing Config component");
    // ... (add tests here)
    
    // Test UTXO Selection
    info!("Testing UTXO Selection component");
    // ... (add tests here)
    
    // Test UTXO Management
    info!("Testing UTXO Management component");
    // ... (add tests here)
    
    info!("test_each_component complete");
}

#[test]
fn test_config_manager() {
    setup_logging();
    
    info!("Testing ConfigManager");
    
    // Create a temporary directory structure
    let temp_dir = tempfile::tempdir().unwrap();
    info!("Created temp directory: {:?}", temp_dir.path());
    
    // ... (add specific tests here)
    
    info!("ConfigManager test complete");
} 