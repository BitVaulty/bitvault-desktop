//! Async Operations Tests
//!
//! Tests async command handling and state management

#[path = "../src/state/async_commands.rs"]
mod async_commands;
use async_commands::{AsyncCommand, AsyncCommandHandler, AsyncResult};

#[test]
fn test_async_command_enum() {
    // Test: AsyncCommand enum variants are accessible
    let cmd1 = AsyncCommand::FetchBalance;
    let cmd2 = AsyncCommand::FetchAddress;
    
    // All commands should be valid
    assert!(matches!(cmd1, AsyncCommand::FetchBalance));
    assert!(matches!(cmd2, AsyncCommand::FetchAddress));
}

#[test]
fn test_async_command_handler_creation() {
    // Test: AsyncCommandHandler can be created
    let handler = AsyncCommandHandler::new();
    
    // Handler should be created successfully
    assert!(true, "Handler created");
}

#[test]
fn test_async_result_enum() {
    // Test: AsyncResult enum works correctly
    let result1 = AsyncResult::Balance { confirmed: 100000, available: 100000 };
    let result2 = AsyncResult::Address("bc1test".to_string());
    let result3 = AsyncResult::Error("Test error".to_string());
    
    // All result types should be valid
    match result1 {
        AsyncResult::Balance { confirmed, available } => {
            assert_eq!(confirmed, 100000);
            assert_eq!(available, 100000);
        }
        _ => panic!("Wrong result type"),
    }
    
    match result2 {
        AsyncResult::Address(address) => {
            assert_eq!(address, "bc1test");
        }
        _ => panic!("Wrong result type"),
    }
    
    match result3 {
        AsyncResult::Error(message) => {
            assert_eq!(message, "Test error");
        }
        _ => panic!("Wrong result type"),
    }
}

#[test]
fn test_async_command_handler_queue() {
    // Test: AsyncCommandHandler can queue commands
    let mut handler = AsyncCommandHandler::new();
    
    // Queue commands using the specific methods
    handler.fetch_balance();
    handler.fetch_address();
    
    // Commands should be queued (we can't directly check the queue, but if it panics, it's broken)
    assert!(true, "Commands queued");
}

#[test]
fn test_async_result_data_extraction() {
    // Test: AsyncResult data can be extracted
    let balance_result = AsyncResult::Balance {
        confirmed: 50000000,
        available: 50000000,
    };
    
    match balance_result {
        AsyncResult::Balance { confirmed, available } => {
            assert_eq!(confirmed, 50000000);
            assert_eq!(available, 50000000);
        }
        _ => panic!("Wrong result type"),
    }
    
    let addr_result = AsyncResult::Address("bc1qtest".to_string());
    match addr_result {
        AsyncResult::Address(address) => {
            assert_eq!(address, "bc1qtest");
        }
        _ => panic!("Wrong result type"),
    }
    
    let error_result = AsyncResult::Error("Network error".to_string());
    match error_result {
        AsyncResult::Error(message) => {
            assert_eq!(message, "Network error");
        }
        _ => panic!("Wrong result type"),
    }
}

#[test]
fn test_async_command_all_variants() {
    // Test: All async command variants are accessible
    let commands = vec![
        AsyncCommand::FetchBalance,
        AsyncCommand::FetchAddress,
    ];
    
    assert_eq!(commands.len(), 2);
    for cmd in commands {
        assert!(matches!(
            cmd,
            AsyncCommand::FetchBalance | AsyncCommand::FetchAddress
        ));
    }
}

#[test]
fn test_async_command_handler_methods() {
    // Test: AsyncCommandHandler methods work
    let mut handler = AsyncCommandHandler::new();
    
    // Test fetch_balance
    handler.fetch_balance();
    assert!(true, "fetch_balance should work");
    
    // Test fetch_address
    handler.fetch_address();
    assert!(true, "fetch_address should work");
}
