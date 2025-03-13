# UTXO Selection in BitVault

This guide explains how to use the UTXO selection functionality in BitVault.

## Overview

The UTXO selection module in `bitvault-common` provides algorithms and utilities for selecting Bitcoin Unspent Transaction Outputs (UTXOs) when creating transactions. It includes various strategies optimized for different use cases like minimizing fees, maximizing privacy, or consolidating small UTXOs.

## Key Types

- `Utxo`: Represents a single UTXO with its outpoint, amount, and metadata
- `UtxoSet`: A collection of UTXOs with methods for addition, removal, and querying
- `UtxoSelector`: The main component for performing UTXO selection
- `SelectionStrategy`: Enum of available selection algorithms
- `SelectionResult`: Result of the selection process (Success or InsufficientFunds)

## Basic Usage

```rust
use bitvault_common::utxo_selection::{
    Utxo, UtxoSelector, SelectionStrategy, SelectionResult
};
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;

// Create some UTXOs
let utxos = vec![
    Utxo::new(
        OutPoint::new(
            Txid::from_str("7967a5185e907a25225574544c31f7b059c1a191d65b53dcc1554d339c4f9efc").unwrap(),
            0,
        ),
        Amount::from_sat(10_000),
        0, // confirmations
        false, // is_change
    ),
    // More UTXOs...
];

// Create a selector with the default fee rate (1 sat/vB)
let selector = UtxoSelector::new();

// Create a selector with a custom fee rate
// let selector = UtxoSelector::with_fee_rate(2.0); // 2 sat/vB

// Select UTXOs for a target amount
let target = Amount::from_sat(5_000);
let result = selector.select_utxos(&utxos, target, SelectionStrategy::MinimizeFee);

// Handle the result
match result {
    SelectionResult::Success { selected, fee_amount, change_amount } => {
        println!("Selected {} UTXOs", selected.len());
        println!("Fee: {} sats", fee_amount.to_sat());
        println!("Change: {} sats", change_amount.to_sat());
        
        // Use selected UTXOs to build transaction...
    },
    SelectionResult::InsufficientFunds { available, required } => {
        println!("Not enough funds! Have {} sats, need {} sats", 
                 available.to_sat(), required.to_sat());
    }
}
```

## Selection Strategies

The module offers several UTXO selection strategies:

1. **MinimizeFee**: Selects UTXOs to minimize transaction fees, typically by using fewer, larger inputs.

2. **MaximizePrivacy**: Prefers UTXOs from different addresses and non-change outputs for better privacy.

3. **Consolidate**: Selects smaller UTXOs first, useful for reducing UTXO count in your wallet.

4. **OldestFirst**: Prioritizes UTXOs with more confirmations, useful for spending mature coins.

5. **CoinControl**: Uses manually pre-selected UTXOs, giving users complete control.

## Advanced Selection Algorithms

For more sophisticated use cases, the module provides advanced algorithms:

```rust
use bitvault_common::utxo_selection::advanced;
use rust_decimal_macros::dec;

// Branch and Bound algorithm (tries to find exact matches)
let result = advanced::branch_and_bound(&utxos, target, dec!(1.0));

// Waste-minimizing algorithm (minimizes fee-to-value ratio waste)
let result = advanced::minimize_waste(&utxos, target, dec!(1.0));
```

## UTXO Tagging

The module allows tagging UTXOs for organization and filtering:

```rust
use bitvault_common::utxo_selection::tagging::{TaggedUtxoSet, TaggedUtxoSetImpl, UtxoTag};

// Create a tagged UTXO set
let mut tagged_set = TaggedUtxoSetImpl::new();

// Add UTXOs to the underlying set
for utxo in utxos {
    tagged_set.utxo_set.add(utxo);
}

// Tag a specific UTXO
tagged_set.tag_utxo(&outpoint, UtxoTag::Priority).unwrap();

// Tag with a custom tag
tagged_set.tag_utxo(&outpoint, UtxoTag::Custom("Important".to_string())).unwrap();

// Find UTXOs by tag
let priority_utxos = tagged_set.find_by_tag(&UtxoTag::Priority);

// Use these UTXOs for selection...
```

## Persistence

To save and load UTXO sets to/from disk:

```rust
use bitvault_common::utxo_selection::persistence;

// Save UTXO set to disk
persistence::save_utxo_set(&utxo_set, "utxos.json").unwrap();

// Load UTXO set from disk
let loaded_set = persistence::load_utxo_set("utxos.json").unwrap();
```

## Integration with BDK

The module can be used with the Bitcoin Development Kit (BDK) to convert BDK UTXOs:

```rust
use bitvault_common::utxo_selection::utils;
use bdk::wallet::Wallet;

// Assuming you have a BDK wallet
let bdk_wallet = /* ... */;

// Get UTXOs from BDK
let bdk_utxos = bdk_wallet.list_unspent().unwrap();

// Convert to our UTXO format
let utxos: Vec<Utxo> = bdk_utxos.iter().map(|utxo| {
    utils::from_bdk_utxo(
        utxo.outpoint,
        utxo.txout.value,
        utxo.confirmation_time.map(|c| c.height as u32).unwrap_or(0),
        false, // Assume it's not change for simplicity
        None, // No address for simplicity
    )
}).collect();

// Use these UTXOs with our selection algorithms...
```

## Security Considerations

- The UTXO selection module does not handle private keys or sign transactions
- The selection strategy can impact both privacy and fees
- Consider using `MaximizePrivacy` for transactions where privacy is important
- Persistence features do not encrypt UTXOs on disk

## Performance Considerations

- Branch and Bound selection can be computationally intensive for large UTXO sets
- For large wallets, consider caching selection results
- The module is optimized for typical wallet use cases with dozens to hundreds of UTXOs 