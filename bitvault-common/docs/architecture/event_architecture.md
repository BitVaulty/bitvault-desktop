# Event-Driven Architecture in BitVault

This document describes the event-driven architecture used in the BitVault wallet, specifically focusing on the UTXO selection and management components.

## Overview

BitVault implements an event-driven architecture to provide:

- Loose coupling between components
- Better observability of system state changes
- Enhanced testability
- Improved modularity
- Security boundary management

The event system consists of two main components:

1. **General Message Bus** (`MessageBus`) - For system-wide events
2. **Domain-Specific Event Buses** - For targeted functionality (e.g., `UtxoEventBus`, `KeyManagementBus`)

## Key Components

### MessageBus

The `MessageBus` is a general-purpose event bus that handles events of different types across the entire system. It provides:

- Event publishing with priority levels
- Subscription to specific event types
- Rate limiting for event flow control
- Dead letter channel for failed event delivery
- Event persistence for critical events

```rust
// Create and start a message bus
let mut message_bus = MessageBus::new();
message_bus.start();

// Subscribe to specific events
let receiver = message_bus.subscribe(EventType::TransactionReceived);

// Publish an event
message_bus.publish(
    EventType::TransactionReceived,
    &json!({ "txid": "abc123", "amount": 10000 }).to_string(),
    MessagePriority::High
);
```

### Domain-Specific Event Buses

Domain-specific event buses provide more targeted functionality for specific domains:

- `UtxoEventBus` for UTXO-related events
- `KeyManagementBus` for key management events

These buses can operate independently or connect to the general message bus:

```rust
// Create a standalone UTXO event bus
let utxo_bus = UtxoEventBus::new();

// Or connect to the general message bus
let utxo_bus = UtxoEventBus::with_general_bus(Arc::new(message_bus));

// Subscribe to specific UTXO events
let selected_receiver = utxo_bus.subscribe("selected");
let all_receiver = utxo_bus.subscribe_all();

// Publish a domain-specific event
utxo_bus.publish(UtxoEvent::Frozen {
    outpoint: OutPointInfo { txid: "abc123".to_string(), vout: 0 }
});
```

## Event Types

### General Events (EventType)

System-wide events are classified by the `EventType` enum:

- `WalletUpdate`, `TransactionReceived`, `TransactionSent`, etc. - General wallet events
- `SecurityAlert` - Security-related events
- `UtxoSelected`, `UtxoStatusChanged` - UTXO-related events
- `ConfigUpdate` - Configuration changes
- `CoreRequest`, `CoreResponse` - Events crossing security boundaries

### Domain-Specific Events

#### UTXO Events (UtxoEvent)

- `Selected` - When UTXOs are selected for a transaction
- `Frozen` - When a UTXO is marked as unavailable for selection
- `Unfrozen` - When a UTXO is marked as available for selection
- `SelectionFailed` - When UTXO selection fails (e.g., insufficient funds)
- `StatusChanged` - When a UTXO's status changes

#### Key Management Events (KeyManagementEvent)

- `KeyGenerated` - When a new key is generated
- `KeyEncrypted` - When a key is encrypted
- `KeyDecryptionFailed` - When key decryption fails

## Integration with UTXO Selection and Management

### UtxoSelector

The `UtxoSelector` uses the event system to publish events during UTXO selection:

```rust
// Create a selector and event bus
let (selector, event_bus) = UtxoSelector::with_event_bus(Arc::new(UtxoEventBus::new()));

// Select UTXOs with event publishing
let result = selector.select_utxos(
    &utxos,
    amount,
    strategy,
    Some(&message_bus),
    Some(&event_bus)
);

// Or use the simplified API
let (result, created_bus) = selector.select_utxos_with_events(
    &utxos,
    amount,
    strategy,
    Some(&message_bus)
);
```

### UtxoManager

The `UtxoManager` uses the event system to publish events for all UTXO operations:

```rust
// Create a manager with event bus
let (manager, event_bus) = UtxoManager::with_new_event_bus();

// Subscribe to events
let receiver = event_bus.subscribe_all();

// Add UTXOs, freeze/unfreeze, and select UTXOs
// Events will be published automatically
manager.add_utxo(utxo);
manager.freeze_utxo(&outpoint);
let result = manager.select_utxos(amount, strategy, Some(&message_bus));
```

## Security Considerations

The event-driven architecture provides several security benefits but also introduces considerations:

- **Event Data Protection**: Events may contain sensitive information about wallet state
- **Security Boundary Crossing**: Events marked as `CoreRequest` or `CoreResponse` cross security boundaries and require special handling
- **Event Persistence**: Critical events are persisted and may contain sensitive data
- **Rate Limiting**: Prevents excessive event flow that could lead to resource exhaustion

## Best Practices

1. **Always Subscribe Before Publishing**: Ensure subscribers are registered before events are published to avoid missing critical events

2. **Error Handling**: Use try/catch when processing events to prevent cascading failures

3. **Timeout Handling**: Always use timeouts when waiting for events to avoid blocking indefinitely

4. **Event Granularity**: Create specific, granular events rather than general-purpose ones

5. **Security-Aware Event Design**: Be mindful of what data is included in events, especially those crossing security boundaries

## Example: Complete Event Flow

```rust
// 1. Create general message bus
let mut message_bus = MessageBus::new();
message_bus.start();

// 2. Create domain-specific event bus
let utxo_bus = Arc::new(UtxoEventBus::with_general_bus(Arc::new(message_bus.clone())));

// 3. Create UTXO manager with event bus
let mut manager = UtxoManager::with_event_bus(Arc::clone(&utxo_bus));

// 4. Subscribe to events
let selected_receiver = utxo_bus.subscribe("selected");
let general_receiver = message_bus.subscribe(EventType::UtxoSelected);

// 5. Add UTXOs and select them
manager.add_utxo(utxo1);
manager.add_utxo(utxo2);
let result = manager.select_utxos(amount, strategy, Some(&message_bus));

// 6. Process the selection result and handle events
match result {
    SelectionResult::Success { selected, fee_amount, change_amount } => {
        // Use selected UTXOs to build transaction
    },
    SelectionResult::InsufficientFunds { available, required } => {
        // Handle insufficient funds
    }
}

// 7. Process events from receivers
match selected_receiver.recv_timeout(Duration::from_millis(100)) {
    Ok(event) => {
        // Handle the event
    },
    Err(_) => {
        // Handle timeout
    }
}
```

## Conclusion

The event-driven architecture in BitVault provides a flexible and powerful foundation for building a secure, maintainable Bitcoin wallet. By separating concerns through domain-specific event buses and leveraging the publisher-subscriber pattern, components can communicate efficiently without tight coupling.

This design enhances testability, allows for better security isolation, and provides a clear path for extending functionality while maintaining the integrity of the system. 