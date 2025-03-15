use bitvault_common::events::{MessageBus, UtxoEventBus, UtxoEvent, OutPointInfo, EventType, MessagePriority};
use bitvault_common::utxo_selection::types::{Utxo, SelectionStrategy, SelectionResult};
use bitvault_common::utxo_selection::selector::UtxoSelector;
use bitvault_common::utxo_management::UtxoManager;
use bitcoin::{Amount, OutPoint, Txid};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::time::Duration;

// Helper function to create test UTXOs
fn create_test_utxos() -> Vec<Utxo> {
    let mut utxos = Vec::new();
    
    // Create several UTXOs with different amounts
    for i in 0..5 {
        let txid_str = format!("{:064x}", i + 1);
        utxos.push(Utxo::new(
            OutPoint::new(
                Txid::from_str(&txid_str).unwrap(),
                0
            ),
            Amount::from_sat((i + 1) * 10_000),
            ((i + 1) * 5) as u32, // Different confirmation counts - convert to u32
            i % 2 == 0, // Alternate between change/non-change
        ));
    }
    
    utxos
}

#[test]
fn test_utxo_event_bus_basic() {
    // Create a UTXO event bus
    let bus = UtxoEventBus::new();
    
    // Create a test outpoint
    let outpoint_info = OutPointInfo {
        txid: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        vout: 0,
    };
    
    // Create an event
    let event = UtxoEvent::Frozen {
        outpoint: outpoint_info.clone(),
    };
    
    // Subscribe to frozen events
    let receiver = bus.subscribe("frozen");
    
    // Publish the event
    bus.publish(event.clone());
    
    // Verify that the event was received
    match receiver.recv_timeout(Duration::from_millis(100)) {
        Ok(received_event) => {
            // Verify it's the same event we sent
            match received_event {
                UtxoEvent::Frozen { outpoint } => {
                    assert_eq!(outpoint.txid, outpoint_info.txid);
                    assert_eq!(outpoint.vout, outpoint_info.vout);
                },
                _ => panic!("Received wrong event type: {:?}", received_event),
            }
        },
        Err(_) => panic!("Did not receive event within timeout"),
    }
}

#[test]
fn test_utxo_event_bus_subscribe_all() {
    // Create a UTXO event bus
    let bus = UtxoEventBus::new();
    
    // Create a test outpoint
    let outpoint_info = OutPointInfo {
        txid: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        vout: 0,
    };
    
    // Create different types of events
    let event1 = UtxoEvent::Frozen { 
        outpoint: outpoint_info.clone() 
    };
    
    let event2 = UtxoEvent::Unfrozen { 
        outpoint: outpoint_info.clone() 
    };
    
    let event3 = UtxoEvent::StatusChanged { 
        outpoint: outpoint_info.clone(),
        status: "confirmed".to_string(),
    };
    
    // Subscribe to all events
    let receiver = bus.subscribe_all();
    
    // Publish the events
    bus.publish(event1.clone());
    bus.publish(event2.clone());
    bus.publish(event3.clone());
    
    // Verify that all events were received in order
    let received_event1 = receiver.recv_timeout(Duration::from_millis(100))
        .expect("Did not receive first event");
    let received_event2 = receiver.recv_timeout(Duration::from_millis(100))
        .expect("Did not receive second event");
    let received_event3 = receiver.recv_timeout(Duration::from_millis(100))
        .expect("Did not receive third event");
    
    assert!(matches!(received_event1, UtxoEvent::Frozen { .. }));
    assert!(matches!(received_event2, UtxoEvent::Unfrozen { .. }));
    assert!(matches!(received_event3, UtxoEvent::StatusChanged { .. }));
}

#[test]
fn test_utxo_event_bus_with_general_bus() {
    // Create a general message bus
    let message_bus = Arc::new(MessageBus::new());
    
    // Create a domain-specific event bus connected to the general bus
    let utxo_bus = UtxoEventBus::with_general_bus(Arc::clone(&message_bus));
    
    // Create a test outpoint
    let outpoint_info = OutPointInfo {
        txid: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        vout: 0,
    };
    
    // Create a selection event
    let event = UtxoEvent::Selected {
        utxos: vec![outpoint_info.clone()],
        strategy: "MinimizeFee".to_string(),
        target_amount: 10_000,
        fee_amount: 1_000,
        change_amount: Some(2_000),
    };
    
    // Subscribe to both event buses
    let utxo_receiver = utxo_bus.subscribe("selected");
    let general_receiver = message_bus.subscribe(EventType::UtxoSelected);
    
    // Start the message bus
    let mut message_bus_mut = MessageBus::new();
    message_bus_mut.start();
    
    // Publish to the domain-specific bus
    utxo_bus.publish(event.clone());
    
    // Verify the event was received on the domain-specific bus
    match utxo_receiver.recv_timeout(Duration::from_millis(100)) {
        Ok(received_event) => {
            match received_event {
                UtxoEvent::Selected { utxos, strategy, target_amount, fee_amount, change_amount } => {
                    assert_eq!(utxos.len(), 1);
                    assert_eq!(utxos[0].txid, outpoint_info.txid);
                    assert_eq!(utxos[0].vout, outpoint_info.vout);
                    assert_eq!(strategy, "MinimizeFee");
                    assert_eq!(target_amount, 10_000);
                    assert_eq!(fee_amount, 1_000);
                    assert_eq!(change_amount, Some(2_000));
                },
                _ => panic!("Received wrong event type"),
            }
        },
        Err(_) => panic!("Did not receive event on domain-specific bus"),
    }
    
    // Verify an event was also forwarded to the general bus
    // (We can't check the full message as it's serialized to JSON)
    match general_receiver.recv_timeout(Duration::from_millis(100)) {
        Ok(_) => {
            // Successfully received a message on the general bus
        },
        Err(_) => panic!("Did not receive event on general bus"),
    }
}

#[test]
fn test_utxo_selector_with_event_bus() {
    // Create a general message bus and a domain-specific event bus
    let message_bus = MessageBus::new();
    let utxo_bus = Arc::new(UtxoEventBus::new());
    
    // Create UTXOs for selection
    let utxos = create_test_utxos();
    
    // Create a selector
    let selector = UtxoSelector::new();
    
    // Subscribe to domain-specific events
    let selected_receiver = utxo_bus.subscribe("selected");
    let failed_receiver = utxo_bus.subscribe("selection_failed");
    
    // Perform UTXO selection with event buses
    let target = Amount::from_sat(25_000);
    let result = selector.select_utxos(
        &utxos,
        target,
        SelectionStrategy::MinimizeFee,
        Some(&message_bus),
        Some(&utxo_bus),
    );
    
    // Verify selection was successful
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            // Check the basic selection result
            assert!(!selected.is_empty());
            assert!(selected.iter().map(|u| u.amount).sum::<Amount>() >= target + fee_amount);
            
            // Verify we received a Selected event
            match selected_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    match event {
                        UtxoEvent::Selected { utxos, strategy, target_amount, fee_amount: event_fee, change_amount: event_change } => {
                            // Verify the event contains the correct data
                            assert_eq!(utxos.len(), selected.len());
                            assert_eq!(strategy, "MinimizeFee");
                            assert_eq!(target_amount, target.to_sat());
                            assert_eq!(event_fee, fee_amount.to_sat());
                            assert_eq!(event_change, Some(change_amount.to_sat()));
                        },
                        _ => panic!("Received wrong event type"),
                    }
                },
                Err(_) => panic!("Did not receive Selected event"),
            }
        },
        SelectionResult::InsufficientFunds { .. } => {
            panic!("Selection should have succeeded");
        }
    }
}

#[test]
fn test_utxo_manager_with_event_bus() {
    // Create a domain-specific event bus
    let utxo_bus = Arc::new(UtxoEventBus::new());
    
    // Create a UTXO manager with the event bus
    let mut manager = UtxoManager::with_event_bus(Arc::clone(&utxo_bus));
    
    // Add UTXOs to the manager
    for utxo in create_test_utxos() {
        manager.add_utxo(utxo);
    }
    
    // Subscribe to domain-specific events
    let selected_receiver = utxo_bus.subscribe("selected");
    
    // Create a general message bus for the selection
    let message_bus = MessageBus::new();
    
    // Perform UTXO selection
    let target = Amount::from_sat(25_000);
    let result = manager.select_utxos(target, SelectionStrategy::MinimizeFee, Some(&message_bus), None);
    
    // Verify selection was successful
    assert!(matches!(result, SelectionResult::Success { .. }));
    
    // Verify we received a Selected event
    match selected_receiver.recv_timeout(Duration::from_millis(100)) {
        Ok(event) => {
            match event {
                UtxoEvent::Selected { strategy, target_amount, .. } => {
                    assert_eq!(strategy, "MinimizeFee");
                    assert_eq!(target_amount, target.to_sat());
                },
                _ => panic!("Received wrong event type"),
            }
        },
        Err(_) => panic!("Did not receive Selected event"),
    }
}

#[test]
fn test_coin_control_with_event_bus() {
    // Create a domain-specific event bus
    let utxo_bus = Arc::new(UtxoEventBus::new());
    
    // Create a UTXO manager with the event bus
    let mut manager = UtxoManager::with_event_bus(Arc::clone(&utxo_bus));
    
    // Add UTXOs to the manager
    let utxos = create_test_utxos();
    for utxo in &utxos {
        manager.add_utxo(utxo.clone());
    }
    
    // Subscribe to domain-specific events
    let selected_receiver = utxo_bus.subscribe("selected");
    
    // Create a general message bus for the selection
    let message_bus = MessageBus::new();
    
    // Perform coin control selection with specific outpoints
    let selected_outpoints = vec![utxos[0].outpoint, utxos[2].outpoint];
    let target = Amount::from_sat(15_000);
    let result = manager.select_coin_control(&selected_outpoints, target, Some(&message_bus), None);
    
    // Verify selection was successful
    assert!(matches!(result, SelectionResult::Success { .. }));
    
    // Verify we received a Selected event
    match selected_receiver.recv_timeout(Duration::from_millis(100)) {
        Ok(event) => {
            match event {
                UtxoEvent::Selected { utxos, strategy, target_amount, .. } => {
                    assert_eq!(utxos.len(), 2);
                    assert_eq!(strategy, "CoinControl");
                    assert_eq!(target_amount, target.to_sat());
                    
                    // Verify the selected outpoints match what we requested
                    let selected_txids: Vec<String> = utxos.iter().map(|u| u.txid.clone()).collect();
                    assert!(selected_txids.contains(&format!("{:064x}", 1)));
                    assert!(selected_txids.contains(&format!("{:064x}", 3)));
                },
                _ => panic!("Received wrong event type"),
            }
        },
        Err(_) => panic!("Did not receive Selected event"),
    }
}

#[test]
fn test_enhanced_utxo_event_integration() {
    // Create a general message bus
    let mut message_bus = MessageBus::new();
    message_bus.start();
    
    // Create a UTXO manager with a new event bus
    let (mut manager, utxo_bus) = UtxoManager::with_new_event_bus();
    
    // Subscribe to all UTXO events
    let event_receiver = utxo_bus.subscribe_all();
    
    // Add some test UTXOs to the manager
    let utxos = create_test_utxos();
    manager.add_utxos(utxos);
    
    // Verify we received StatusChanged events for each added UTXO
    for _ in 0..5 {
        match event_receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                match event {
                    UtxoEvent::StatusChanged { status, .. } => {
                        assert_eq!(status, "added");
                    },
                    _ => panic!("Unexpected event type: {:?}", event),
                }
            },
            Err(_) => panic!("Did not receive StatusChanged event"),
        }
    }
    
    // Freeze a UTXO and verify the event
    let txid_str = format!("{:064x}", 1);
    let outpoint = OutPoint::new(Txid::from_str(&txid_str).unwrap(), 0);
    manager.freeze_utxo(&outpoint);
    
    match event_receiver.recv_timeout(Duration::from_millis(100)) {
        Ok(event) => {
            match event {
                UtxoEvent::Frozen { outpoint } => {
                    assert_eq!(outpoint.txid, txid_str);
                },
                _ => panic!("Unexpected event type: {:?}", event),
            }
        },
        Err(_) => panic!("Did not receive Frozen event"),
    }
    
    // Use the enhanced selection method that manages event buses
    let target = Amount::from_sat(25_000);
    let (result, _) = manager.select_utxos_with_events(
        target,
        SelectionStrategy::MinimizeFee,
        Some(&message_bus),
        false, // Don't create a new event bus since we already have one
    );
    
    // Verify the selection was successful
    match result {
        SelectionResult::Success { selected, fee_amount, change_amount } => {
            assert!(!selected.is_empty());
            assert!(selected.iter().map(|u| u.amount).sum::<Amount>() >= target + fee_amount);
            
            // Verify we received a Selected event
            match event_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    match event {
                        UtxoEvent::Selected { strategy, target_amount, .. } => {
                            assert_eq!(strategy, "MinimizeFee");
                            assert_eq!(target_amount, target.to_sat());
                        },
                        _ => panic!("Unexpected event type: {:?}", event),
                    }
                },
                Err(_) => panic!("Did not receive Selected event"),
            }
        },
        SelectionResult::InsufficientFunds { .. } => {
            panic!("Selection should have succeeded");
        },
    }
    
    // Now try a selection that should fail (too large amount)
    let large_target = Amount::from_sat(1_000_000);
    let (result, _) = manager.select_utxos_with_events(
        large_target,
        SelectionStrategy::MinimizeFee,
        Some(&message_bus),
        false,
    );
    
    // Verify the selection failed
    match result {
        SelectionResult::Success { .. } => {
            panic!("Selection should have failed");
        },
        SelectionResult::InsufficientFunds { .. } => {
            // Verify we received a SelectionFailed event
            match event_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    match event {
                        UtxoEvent::SelectionFailed { reason, strategy, .. } => {
                            assert_eq!(reason, "insufficient_funds");
                            assert_eq!(strategy, "MinimizeFee");
                        },
                        _ => panic!("Unexpected event type: {:?}", event),
                    }
                },
                Err(_) => panic!("Did not receive SelectionFailed event"),
            }
        },
    }
} 