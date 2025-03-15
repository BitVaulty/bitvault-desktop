# UTXO Selection Module

This module provides UTXO selection functionality for the BitVault wallet.

## Overview

The UTXO selection module is responsible for selecting appropriate UTXOs (Unspent Transaction Outputs) for transactions based on various strategies and requirements. It implements the Strategy design pattern to allow for flexible and extensible selection algorithms.

## Architecture

The module is organized as follows:

```
bitvault-common/src/utxo_selection/
├── mod.rs             # Main exports and documentation
├── types.rs           # Utxo, UtxoSet, and SelectionResult types
├── strategies/        # Folder for selection strategies
│   ├── mod.rs         # Exports the strategy trait
│   ├── base.rs        # Common utilities for strategies
│   ├── minimize_fee.rs
│   ├── maximize_privacy.rs
│   ├── consolidate.rs
│   ├── oldest_first.rs
│   ├── minimize_change.rs
│   └── avoid_change.rs
└── selector.rs        # Main selector implementation
```

## Event-Driven Architecture

The UTXO selection module now implements an event-driven architecture using both the general MessageBus and a domain-specific UtxoEventBus. This allows for:

1. Better testability through event inspection
2. Looser coupling between components
3. Improved ability to audit actions
4. Consistent communication pattern

### Domain-Specific Events

The module uses the following domain-specific events:

- `UtxoEvent::Selected`: Triggered when UTXOs are selected with a specific strategy
- `UtxoEvent::Frozen`: Triggered when a UTXO is frozen
- `UtxoEvent::Unfrozen`: Triggered when a UTXO is unfrozen
- `UtxoEvent::SelectionFailed`: Triggered when a selection operation fails
- `UtxoEvent::StatusChanged`: Triggered when a UTXO's status changes

### Event Bus Integration

The UtxoSelector and UtxoManager classes can work with both:

1. The general MessageBus for backward compatibility
2. The domain-specific UtxoEventBus for more detailed and structured events

Events published to the domain-specific bus are also forwarded to the general bus when appropriate.

## Usage

### Basic Usage

```rust
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::utxo_selection::types::{SelectionStrategy, SelectionResult};
use bitcoin::Amount;

// Create a selector with default settings
let selector = UtxoSelector::new();

// Select UTXOs for a transaction
let result = selector.select_utxos(
    &utxos,
    Amount::from_sat(50_000), // target amount
    SelectionStrategy::MinimizeFee, // strategy
    None, // Optional message bus
    None, // Optional domain-specific event bus
);

// Process the result
match result {
    SelectionResult::Success { selected, fee_amount, change_amount } => {
        // Use the selected UTXOs to create a transaction
    },
    SelectionResult::InsufficientFunds { available, required } => {
        // Handle insufficient funds case
    }
}
```

### With Event Bus

```rust
use bitvault_common::events::{MessageBus, UtxoEventBus};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::utxo_selection::types::{SelectionStrategy, SelectionResult};
use bitcoin::Amount;
use std::sync::Arc;

// Create event buses
let message_bus = MessageBus::new();
let utxo_bus = Arc::new(UtxoEventBus::with_general_bus(Arc::new(message_bus)));

// Create a selector
let selector = UtxoSelector::new();

// Select UTXOs with event publishing
let result = selector.select_utxos(
    &utxos,
    Amount::from_sat(50_000),
    SelectionStrategy::MinimizeFee,
    Some(&message_bus),
    Some(&utxo_bus),
);

// Subscribe to events
let receiver = utxo_bus.subscribe("selected");
while let Ok(event) = receiver.recv() {
    // Process events
}
```

## Selection Strategies

The module provides several selection strategies:

- `MinimizeFee`: Selects UTXOs to minimize the transaction fee
- `MinimizeChange`: Selects UTXOs to minimize the change amount
- `OldestFirst`: Selects the oldest UTXOs first
- `PrivacyFocused`: Selects UTXOs with privacy considerations
- `MaximizePrivacy`: Maximizes privacy by selecting UTXOs from different sources
- `Consolidate`: Consolidates many small UTXOs into fewer larger ones
- `AvoidChange`: Tries to select UTXOs that avoid creating change
- `CoinControl`: Uses pre-selected UTXOs (manual selection) 