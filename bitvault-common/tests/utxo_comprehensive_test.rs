use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::utxo_management::UtxoManager;
use bitvault_common::events::{MessageBus, UtxoEventBus};
use bitcoin::{Amount, OutPoint, Txid, Network};
use std::str::FromStr;
use std::sync::Once;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::time::{Duration, Instant};
use bitvault_common::utxo_selection::strategies::minimize_change::MinimizeChangeStrategy;

// Import test helpers
mod test_helpers;
use test_helpers::{
    log_test_start, log_test_end, log_info, log_error,
    write_test_output, read_test_output, test_output_exists,
    create_test_logger
};

// Initialize for this test file
static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        // Initialize logging or other one-time setup
        log_info("Initializing UTXO comprehensive test");
    });
}

// Helper to create standard test UTXOs
fn create_test_utxos() -> Vec<Utxo> {
    vec![
        Utxo::new(
            OutPoint::new(
                Txid::from_str("7967a5185e907a25225574544c31f7b059c1a191d65b53dcc1554d339c4f9efc").unwrap(),
                0
            ),
            Amount::from_sat(10_000),
            1,
            false,
        ),
        Utxo::new(
            OutPoint::new(
                Txid::from_str("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap(),
                0
            ),
            Amount::from_sat(20_000),
            2,
            true, // change output
        ),
        Utxo::new(
            OutPoint::new(
                Txid::from_str("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").unwrap(),
                0
            ),
            Amount::from_sat(100_000),
            5,
            false,
        ),
        Utxo::new(
            OutPoint::new(
                Txid::from_str("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap(),
                0
            ),
            Amount::from_sat(50_000),
            10,
            false,
        ),
    ]
}

// Test basic UTXO selection with file output
#[test]
fn test_basic_selection_with_file_output() {
    setup();
    log_test_start("basic_selection_with_file_output");
    
    // Create a string buffer for test output
    let mut output = String::new();
    writeln!(output, "UTXO Selection Test Results").unwrap();
    writeln!(output, "=========================").unwrap();
    
    // Create test UTXOs
    let utxos = create_test_utxos();
    writeln!(output, "Created {} test UTXOs:", utxos.len()).unwrap();
    for (i, utxo) in utxos.iter().enumerate() {
        writeln!(output, "  UTXO {}: {} sats, {} confirmations", 
                i + 1, utxo.amount.to_sat(), utxo.confirmations).unwrap();
    }
    
    // Create a selector and select UTXOs
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(75_000);
    writeln!(output, "\nPerforming selection for target amount: {} sats", target.to_sat()).unwrap();
    
    let result = selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeFee, None, None);
    
    // Process the result and write to the output
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            let selected_amount: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            
            writeln!(output, "\nSelection successful:").unwrap();
            writeln!(output, "  Selected {} UTXOs", selected.len()).unwrap();
            writeln!(output, "  Total selected: {} sats", selected_amount).unwrap();
            writeln!(output, "  Fee amount: {} sats", fee_amount.to_sat()).unwrap();
            writeln!(output, "  Change amount: {} sats", change_amount.to_sat()).unwrap();
            
            // Verify the balance equation: selected = target + fee + change
            let sum = target.to_sat() + fee_amount.to_sat() + change_amount.to_sat();
            if sum == selected_amount {
                writeln!(output, "\nBalance equation verified: {} = {} + {} + {}", 
                        selected_amount, target.to_sat(), fee_amount.to_sat(), change_amount.to_sat()).unwrap();
            } else {
                writeln!(output, "\nBALANCE EQUATION ERROR: {} != {} + {} + {}", 
                        selected_amount, target.to_sat(), fee_amount.to_sat(), change_amount.to_sat()).unwrap();
                log_error("Balance equation failed");
            }
            
            // Standard test assertions
            assert_eq!(sum, selected_amount, "Balance equation must be satisfied");
            assert!(fee_amount.to_sat() > 0, "Fee should be positive");
            assert!(selected_amount >= target.to_sat() + fee_amount.to_sat(), "Selected amount should cover target + fee");
        },
        SelectionResult::InsufficientFunds { available, required } => {
            writeln!(output, "\nSelection failed with insufficient funds:").unwrap();
            writeln!(output, "  Available: {} sats", available.to_sat()).unwrap();
            writeln!(output, "  Required: {} sats", required.to_sat()).unwrap();
            
            log_error("Selection failed with insufficient funds");
            panic!("Expected successful selection but got insufficient funds");
        }
    }
    
    // Write the test output to a file
    if let Err(e) = write_test_output("utxo_basic_selection", &output) {
        log_error(&format!("Failed to write test output: {}", e));
    }
    
    log_test_end("basic_selection_with_file_output", true);
}

// Test performance with various selection strategies
#[test]
fn test_selection_performance() {
    setup();
    log_test_start("selection_performance");
    
    // Create a file-specific logger
    let mut logger = create_test_logger("selection_performance");
    
    // Create a larger set of UTXOs for performance testing
    let mut utxos = Vec::with_capacity(100);
    
    for i in 0..100 {
        // Create a deterministic but unique txid
        let txid_hex = format!("{:064x}", 0x1000000000000000u64 + (i as u64));
        let txid = Txid::from_str(&txid_hex).unwrap();
        
        // Create UTXOs with varying amounts
        let amount = match i % 5 {
            0 => 5_000,       // Small UTXOs
            1 => 10_000,      // Medium UTXOs
            2 => 50_000,      // Larger UTXOs
            3 => 100_000,     // Large UTXOs
            _ => 1_000_000,   // Very large UTXOs
        };
        
        // Add some variety to confirmations
        let confirmations = (i % 10) as u32;
        
        // Make some UTXOs change outputs
        let is_change = i % 3 == 0;
        
        utxos.push(Utxo::new(
            OutPoint::new(txid, 0),
            Amount::from_sat(amount),
            confirmations,
            is_change
        ));
    }
    
    writeln!(logger, "Created {} test UTXOs", utxos.len()).unwrap();
    
    // Test each selection strategy and measure performance
    let strategies = [
        SelectionStrategy::MinimizeFee,
        SelectionStrategy::MinimizeChange,
        SelectionStrategy::OldestFirst,
        SelectionStrategy::PrivacyFocused,
        SelectionStrategy::MaximizePrivacy,
        SelectionStrategy::Consolidate,
    ];
    
    let target = Amount::from_sat(200_000);
    writeln!(logger, "Target amount: {} sats", target.to_sat()).unwrap();
    
    let selector = UtxoSelector::new();
    
    for strategy in &strategies {
        writeln!(logger, "\nTesting {:?} strategy...", strategy).unwrap();
        
        let start = Instant::now();
        let result = selector.select_utxos(&utxos, target, *strategy, None, None);
        let duration = start.elapsed();
        
        match result {
            SelectionResult::Success { selected, fee_amount, change_amount } => {
                let selected_amount: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
                
                writeln!(logger, "  Selection time: {:?}", duration).unwrap();
                writeln!(logger, "  Selected {} UTXOs", selected.len()).unwrap();
                writeln!(logger, "  Total selected: {} sats", selected_amount).unwrap();
                writeln!(logger, "  Fee amount: {} sats", fee_amount.to_sat()).unwrap();
                writeln!(logger, "  Change amount: {} sats", change_amount.to_sat()).unwrap();
                
                // Make sure selection completed within a reasonable time
                assert!(duration < Duration::from_secs(5), 
                        "Selection with {:?} strategy took too long: {:?}", strategy, duration);
                
                // Validate selection
                assert!(selected_amount >= target.to_sat() + fee_amount.to_sat(),
                        "Selected amount should cover target + fee");
                assert_eq!(selected_amount, target.to_sat() + fee_amount.to_sat() + change_amount.to_sat(),
                        "Balance equation must be satisfied");
            },
            SelectionResult::InsufficientFunds { available, required } => {
                writeln!(logger, "  Selection failed: Insufficient funds").unwrap();
                writeln!(logger, "  Available: {} sats", available.to_sat()).unwrap();
                writeln!(logger, "  Required: {} sats", required.to_sat()).unwrap();
                writeln!(logger, "  Duration: {:?}", duration).unwrap();
                
                log_error(&format!("Strategy {:?} failed with insufficient funds", strategy));
            }
        }
    }
    
    log_test_end("selection_performance", true);
}

// Test interoperability between UtxoManager and UtxoSelector
#[test]
fn test_utxo_interoperability() {
    setup();
    log_test_start("utxo_interoperability");
    
    // Create a string buffer for test output
    let mut output = String::new();
    writeln!(output, "UTXO Interoperability Test Results").unwrap();
    writeln!(output, "================================").unwrap();
    
    // Create a test UTXO
    let utxo = Utxo::new(
        OutPoint::new(
            Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            0
        ),
        Amount::from_sat(100_000),
        10, // confirmations
        false // not change
    );
    
    // TEST CASE 1: Direct UtxoSelector
    writeln!(output, "\nTEST CASE 1: Direct UtxoSelector").unwrap();
    let selector = UtxoSelector::new();
    let target = Amount::from_sat(50_000);
    
    writeln!(output, "Target amount: {} sats", target.to_sat()).unwrap();
    
    let selector_result = selector.select_utxos(&[utxo.clone()], target, SelectionStrategy::MinimizeFee, None, None);
    
    // TEST CASE 2: Via UtxoManager
    writeln!(output, "\nTEST CASE 2: Via UtxoManager").unwrap();
    let mut manager = UtxoManager::new();
    manager.add_utxo(utxo.clone());
    
    let manager_result = manager.select_utxos(target, SelectionStrategy::MinimizeFee, None, None);
    
    // Compare results
    writeln!(output, "\nRESULT COMPARISON:").unwrap();
    
    match (selector_result, manager_result) {
        (SelectionResult::Success { selected: sel_selected, fee_amount: sel_fee, change_amount: sel_change },
         SelectionResult::Success { selected: mgr_selected, fee_amount: mgr_fee, change_amount: mgr_change }) => {
            
            writeln!(output, "Both approaches succeeded").unwrap();
            
            // Compare selected UTXOs
            writeln!(output, "  Selected UTXOs: {} vs {}", sel_selected.len(), mgr_selected.len()).unwrap();
            
            // Compare fees
            writeln!(output, "  Fee amount: {} sats vs {} sats", sel_fee.to_sat(), mgr_fee.to_sat()).unwrap();
            
            // Compare change
            writeln!(output, "  Change amount: {} sats vs {} sats", sel_change.to_sat(), mgr_change.to_sat()).unwrap();
            
            // Test equality
            let fees_match = sel_fee.to_sat() == mgr_fee.to_sat();
            let change_match = sel_change.to_sat() == mgr_change.to_sat();
            
            writeln!(output, "\nEquality checks:").unwrap();
            writeln!(output, "  Fees match: {}", fees_match).unwrap();
            writeln!(output, "  Change matches: {}", change_match).unwrap();
            
            // Formal assertions
            assert_eq!(sel_selected.len(), mgr_selected.len(), "Selected UTXO count should match");
            assert_eq!(sel_fee.to_sat(), mgr_fee.to_sat(), "Fee amounts should match");
            assert_eq!(sel_change.to_sat(), mgr_change.to_sat(), "Change amounts should match");
            
            writeln!(output, "\nInteroperability test successful").unwrap();
        },
        (_, _) => {
            writeln!(output, "ERROR: Results do not match").unwrap();
            log_error("Interoperability test failed - results do not match");
            panic!("Interoperability test failed");
        }
    }
    
    // Write the test output to a file
    if let Err(e) = write_test_output("utxo_interoperability", &output) {
        log_error(&format!("Failed to write test output: {}", e));
    }
    
    log_test_end("utxo_interoperability", true);
}

// Test reading the test output from previous runs
#[test]
fn test_reading_file_output() {
    setup();
    log_test_start("reading_file_output");
    
    // The tests above must run first
    // Wait a moment to ensure the files are written
    std::thread::sleep(Duration::from_millis(100));
    
    // Read the output from the basic selection test
    if test_output_exists("utxo_basic_selection") {
        match read_test_output("utxo_basic_selection") {
            Ok(content) => {
                log_info("Successfully read from file output");
                assert!(content.contains("UTXO Selection Test Results"), "Expected output content not found");
                assert!(content.contains("Selection successful"), "Expected success message not found");
            },
            Err(e) => {
                log_error(&format!("Failed to read test output: {}", e));
                panic!("Failed to read test output");
            }
        }
    } else {
        log_error("Test output file doesn't exist. Please run the basic selection test first.");
        panic!("Test output file doesn't exist");
    }
    
    log_test_end("reading_file_output", true);
}

// Test timeout handling for UTXO selection algorithms
#[test]
fn test_selection_timeout_handling() {
    setup();
    log_test_start("selection_timeout_handling");
    
    // Create a string buffer for test output
    let mut output = String::new();
    writeln!(output, "UTXO Selection Timeout Test Results").unwrap();
    writeln!(output, "=================================").unwrap();
    
    // Create a larger set of UTXOs for timeout testing
    let mut utxos = Vec::with_capacity(500);
    
    for i in 0..500 {
        // Create a deterministic but unique txid
        let txid_hex = format!("{:064x}", 0x2000000000000000u64 + (i as u64));
        let txid = Txid::from_str(&txid_hex).unwrap();
        
        // Create UTXOs with varying amounts
        let amount = match i % 5 {
            0 => 5_000,       // Small UTXOs
            1 => 10_000,      // Medium UTXOs
            2 => 50_000,      // Larger UTXOs
            3 => 100_000,     // Large UTXOs
            _ => 1_000_000,   // Very large UTXOs
        };
        
        utxos.push(Utxo::new(
            OutPoint::new(txid, 0),
            Amount::from_sat(amount),
            1,
            false
        ));
    }
    
    writeln!(output, "Created {} test UTXOs", utxos.len()).unwrap();
    
    // Create a selector with a very short timeout
    let selector = UtxoSelector::with_minimize_change_strategy(
        MinimizeChangeStrategy::with_timeout(1) // 1ms timeout - should trigger timeout
    );
    
    // Target amount that requires multiple UTXOs
    let target = Amount::from_sat(10_000_000); // 10M sats
    
    writeln!(output, "Running selection with very short timeout (1ms)").unwrap();
    writeln!(output, "Target amount: {} sats", target.to_sat()).unwrap();
    
    let start = Instant::now();
    
    // Run the selection
    let result = selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None);
    let duration = start.elapsed();
    
    writeln!(output, "Selection returned in {:?}", duration).unwrap();
    
    // The selection should still return a valid result, even with timeout
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            writeln!(output, "Selection completed despite timeout").unwrap();
            writeln!(output, "Selected {} UTXOs", selected.len()).unwrap();
            
            let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
            writeln!(output, "Total selected amount: {} sats", total_selected).unwrap();
            writeln!(output, "Fee amount: {} sats", fee_amount.to_sat()).unwrap();
            writeln!(output, "Change amount: {} sats", change_amount.to_sat()).unwrap();
            
            // Verify selection still satisfies the balance equation
            assert_eq!(
                total_selected,
                target.to_sat() + fee_amount.to_sat() + change_amount.to_sat(),
                "Balance equation must be satisfied even after timeout"
            );
            
            writeln!(output, "Balance equation verified despite timeout").unwrap();
        },
        SelectionResult::InsufficientFunds { available, required } => {
            writeln!(output, "Selection failed with insufficient funds:").unwrap();
            writeln!(output, "  Available: {} sats", available.to_sat()).unwrap();
            writeln!(output, "  Required: {} sats", required.to_sat()).unwrap();
            
            // This is also an acceptable result if there genuinely aren't enough funds
            if available.to_sat() < required.to_sat() {
                writeln!(output, "Insufficient funds is a valid result").unwrap();
            } else {
                log_error("Unexpected insufficient funds result");
                panic!("Unexpected insufficient funds result when timeout occurred");
            }
        }
    }
    
    // Try with a more reasonable timeout
    writeln!(output, "\nRunning selection with reasonable timeout (100ms)").unwrap();
    
    let selector = UtxoSelector::with_minimize_change_strategy(
        MinimizeChangeStrategy::with_timeout(100) // 100ms should be enough for basic selection
    );
    
    let start = Instant::now();
    
    // Run the selection
    let result = selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None);
    let duration = start.elapsed();
    
    writeln!(output, "Selection returned in {:?}", duration).unwrap();
    
    // The selection should return a valid result
    match result {
        SelectionResult::Success { selected, .. } => {
            writeln!(output, "Selection completed successfully with reasonable timeout").unwrap();
            writeln!(output, "Selected {} UTXOs", selected.len()).unwrap();
        },
        SelectionResult::InsufficientFunds { available, required } => {
            writeln!(output, "Selection failed with insufficient funds:").unwrap();
            writeln!(output, "  Available: {} sats", available.to_sat()).unwrap();
            writeln!(output, "  Required: {} sats", required.to_sat()).unwrap();
        }
    }
    
    // Write the test output to a file
    if let Err(e) = write_test_output("utxo_timeout_selection", &output) {
        log_error(&format!("Failed to write test output: {}", e));
    }
    
    log_test_end("selection_timeout_handling", true);
} 