# UTXO Selection Examples

This document provides examples of how to use the UTXO selection module in BitVault, demonstrating various selection strategies and their usage patterns.

## Basic UTXO Selection

The simplest way to use the UTXO selection system is with the default parameters:

```rust
use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;

// Create a simple UTXO set
let utxo = Utxo::new(
    OutPoint::new(
        Txid::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        0,
    ),
    Amount::from_sat(100_000),
    1,  // confirmations
    false,  // is_change
);
let utxos = vec![utxo];

// Create a selector with default settings
let selector = UtxoSelector::new();

// Select UTXOs for a transaction
let target = Amount::from_sat(50_000);
let result = selector.select_utxos(
    &utxos,
    target, 
    SelectionStrategy::MinimizeFee,  // strategy
    None,  // Optional message bus
    None,  // Optional domain-specific event bus
);

// Process the result
match result {
    SelectionResult::Success { selected, fee_amount, change_amount } => {
        // Use the selected UTXOs, fee, and change amount to build transaction
        println!("Selected {} UTXOs with total amount of {} sats", 
                 selected.len(), 
                 selected.iter().map(|u| u.amount.to_sat()).sum::<u64>());
        println!("Fee: {} sats", fee_amount.to_sat());
        println!("Change: {} sats", change_amount.to_sat());
    },
    SelectionResult::InsufficientFunds { available, required } => {
        println!("Insufficient funds! Available: {}, Required: {}", 
                 available.to_sat(), required.to_sat());
    }
}
```

## Using Custom Selection Strategies

BitVault supports several selection strategies that can be customized:

### MinimizeChange Strategy with Custom Timeout

The MinimizeChange strategy can be performance-intensive, so you may want to set a timeout:

```rust
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::utxo_selection::strategies::minimize_change::MinimizeChangeStrategy;

// Create a selector with a custom MinimizeChange strategy
let selector = UtxoSelector::with_minimize_change_strategy(
    MinimizeChangeStrategy::with_timeout(300)  // 300ms timeout
);

// Use the selector as normal...
let result = selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None);
```

### Using Event Buses

For event-driven applications, you can connect the UTXO selection process to event buses:

```rust
use bitvault_common::events::{MessageBus, UtxoEventBus};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use std::sync::Arc;

// Create general message bus for system-wide events
let message_bus = MessageBus::new();

// Create UTXO-specific event bus
let utxo_bus = Arc::new(UtxoEventBus::new());

// Select UTXOs with event publishing
let result = selector.select_utxos(
    &utxos,
    target,
    SelectionStrategy::MinimizeFee,
    Some(&message_bus),
    Some(&utxo_bus),
);

// Or use the simplified API
let (result, created_bus) = selector.select_utxos_with_events(
    &utxos,
    target,
    SelectionStrategy::MinimizeFee,
    Some(&message_bus),
);

// Subscribe to selection events
let receiver = utxo_bus.subscribe("selected");
while let Ok(event) = receiver.recv() {
    // Process events...
}
```

## Complete UTXO Selection Example

Here's a more complete example demonstrating UTXO selection with a larger set of UTXOs:

```rust
use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::utxo_selection::strategies::minimize_change::MinimizeChangeStrategy;
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use std::time::{Duration, Instant};

// Generate test UTXOs
let mut utxos = Vec::with_capacity(100);
for i in 0..100 {
    // Create unique txid
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
    
    // Add some variety to confirmations and change status
    let confirmations = (i % 10) as u32;
    let is_change = i % 3 == 0;
    
    utxos.push(Utxo::new(
        OutPoint::new(txid, 0),
        Amount::from_sat(amount),
        confirmations,
        is_change
    ));
}

// Target amount that requires multiple UTXOs
let target = Amount::from_sat(200_000);

// Create selector with timeout
let selector = UtxoSelector::with_minimize_change_strategy(
    MinimizeChangeStrategy::with_timeout(300) // 300ms timeout
);

let start = Instant::now();

// Run the selection
match selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeChange, None, None) {
    SelectionResult::Success { selected, fee_amount, change_amount } => {
        let duration = start.elapsed();
        println!("Selection completed in {:?}", duration);
        
        let total_selected: u64 = selected.iter().map(|u| u.amount.to_sat()).sum();
        println!("Selected {} UTXOs with total amount of {} sats", selected.len(), total_selected);
        println!("Fee: {} sats", fee_amount.to_sat());
        println!("Change: {} sats", change_amount.to_sat());
        
        // Validate selection
        assert!(total_selected >= target.to_sat() + fee_amount.to_sat());
        assert!(duration < Duration::from_secs(5));
    },
    SelectionResult::InsufficientFunds { available, required } => {
        println!("Selection failed with insufficient funds!");
        println!("Available: {} sats", available.to_sat());
        println!("Required: {} sats", required.to_sat());
    }
}
```

## Selection Strategy Comparison

| Strategy | Use Case | Advantages | Disadvantages |
|----------|----------|------------|--------------|
| **MinimizeFee** | General purpose | Lower transaction fees | May result in more change outputs over time |
| **MinimizeChange** | Value preservation | Reduces change dust | Can take longer to compute |
| **OldestFirst** | UTXO consolidation | Helps prevent dust accumulation | May result in higher fees |
| **PrivacyFocused** | Enhanced privacy | Better transaction fingerprinting protection | May use more inputs than necessary |
| **MaximizePrivacy** | Maximum privacy | Uses UTXOs from different origins | Highest fees |
| **Consolidate** | Wallet maintenance | Reduces UTXO count | Creates larger transactions |
| **AvoidChange** | Exact payments | Cleaner transaction graph | Limited applicability |

## Related Documentation

- [UTXO Management](utxo_management.md) - Overview of UTXO handling
- [UTXO Implementation](utxo_implementation.md) - Technical details of UTXO selection
- [Architecture Overview](../architecture/updated_architecture.md) - UTXO's role in the wallet 