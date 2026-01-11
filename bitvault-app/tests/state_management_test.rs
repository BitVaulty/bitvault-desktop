//! State Management Tests
//!
//! Tests application state, vault data, and state transitions

#[path = "../src/state/navigation.rs"]
mod navigation;
use navigation::{Navigation, View};

#[path = "../src/state/vault_data.rs"]
mod vault_data;
use vault_data::VaultData;

#[test]
fn test_vault_data_initialization() {
    // Test: Vault data initializes correctly
    let vault_data = VaultData::new();

    assert!(vault_data.confirmed_balance.is_none());
    assert!(vault_data.available_balance.is_none());
    assert!(vault_data.receive_address.is_none());
    assert!(!vault_data.is_loading);
}

#[test]
fn test_vault_data_updates() {
    // Test: Vault data can be updated
    let mut vault_data = VaultData::new();

    // Update balance
    vault_data.confirmed_balance = Some(100000);
    vault_data.available_balance = Some(100000);
    assert_eq!(vault_data.confirmed_balance, Some(100000));
    assert_eq!(vault_data.available_balance, Some(100000));

    // Update address
    vault_data.receive_address = Some("bc1test123".to_string());
    assert_eq!(vault_data.receive_address, Some("bc1test123".to_string()));
}

#[test]
fn test_vault_data_address_updates() {
    // Test: Vault data address updates work correctly
    let mut vault_data = VaultData::new();

    // Initially no address
    assert!(vault_data.receive_address.is_none());

    // Set address
    vault_data.receive_address = Some("bc1test123".to_string());
    assert_eq!(vault_data.receive_address, Some("bc1test123".to_string()));

    // Update address
    vault_data.receive_address = Some("bc1new456".to_string());
    assert_eq!(vault_data.receive_address, Some("bc1new456".to_string()));
}

#[test]
fn test_navigation_state_consistency() {
    // Test: Navigation state remains consistent
    let mut navigation = Navigation::new();

    // Initial state
    assert_eq!(navigation.current_view, View::VaultSelection);
    assert!(navigation.history.is_empty());
    assert!(navigation.navigation_data.is_none());

    // Navigate multiple times
    for i in 0..10 {
        navigation.navigate_to(View::Dashboard { tab: i % 3 });
        assert!(matches!(
            navigation.current_view,
            View::Dashboard { tab: _ }
        ));
        assert_eq!(navigation.history.len(), i + 1);
    }

    // Go back multiple times (but stop before reaching root to avoid the edge case)
    for i in (1..10).rev() {
        let went_back = navigation.go_back();
        assert!(went_back, "Should be able to go back when history exists");
        assert_eq!(navigation.history.len(), i, "History should be reduced");
    }

    // Final go back should return to root
    let went_back = navigation.go_back();
    assert!(went_back, "Should go back to root");
    assert_eq!(navigation.current_view, View::VaultSelection);
    assert!(navigation.history.is_empty());

    // Can't go back from root
    let went_back = navigation.go_back();
    assert!(!went_back, "Should not be able to go back from root");
}

#[test]
fn test_navigation_data_clearing() {
    // Test: Navigation data is cleared appropriately
    let mut navigation = Navigation::new();

    // Set data
    navigation.navigate_to_with_data(View::SendTransaction, Some("test_data".to_string()));
    assert_eq!(navigation.navigation_data, Some("test_data".to_string()));

    // Navigate without data clears it
    navigation.navigate_to(View::Receive);
    assert!(navigation.navigation_data.is_none());

    // Set data again
    navigation.navigate_to_with_data(
        View::TransactionDetail {
            txid: "tx123".to_string(),
        },
        Some("new_data".to_string()),
    );
    assert_eq!(navigation.navigation_data, Some("new_data".to_string()));

    // Take data clears it
    let data = navigation.take_navigation_data();
    assert_eq!(data, Some("new_data".to_string()));
    assert!(navigation.navigation_data.is_none());
}

#[test]
fn test_view_cloning() {
    // Test: Views can be cloned for history
    let view1 = View::Dashboard { tab: 0 };
    let view2 = view1.clone();

    assert_eq!(view1, view2);

    let view3 = View::SendTransaction;
    let view4 = view3.clone();

    assert_eq!(view3, view4);
    assert_ne!(view1, view3);
}

#[test]
fn test_navigation_history_management() {
    // Test: Navigation history is managed correctly
    let mut navigation = Navigation::new();

    // Build up history
    navigation.navigate_to(View::Dashboard { tab: 0 });
    navigation.navigate_to(View::SendTransaction);
    navigation.navigate_to(View::Receive);

    assert_eq!(navigation.history.len(), 3);
    assert_eq!(navigation.history[0], View::VaultSelection);
    assert_eq!(navigation.history[1], View::Dashboard { tab: 0 });
    assert_eq!(navigation.history[2], View::SendTransaction);

    // Go back reduces history
    navigation.go_back();
    assert_eq!(navigation.history.len(), 2);
    assert_eq!(navigation.current_view, View::SendTransaction);
}

#[test]
fn test_view_debug_formatting() {
    // Test: Views can be formatted for debugging
    let view = View::Dashboard { tab: 1 };
    let debug_str = format!("{:?}", view);
    assert!(debug_str.contains("Dashboard") || debug_str.contains("1"));
}

#[test]
fn test_navigation_can_go_back() {
    // Test: can_go_back() works correctly
    let mut navigation = Navigation::new();

    // Initially can't go back
    assert!(!navigation.can_go_back());

    // After navigation, can go back
    navigation.navigate_to(View::Dashboard { tab: 0 });
    assert!(navigation.can_go_back());

    // After going back to root, can't go back
    navigation.go_back();
    assert!(!navigation.can_go_back());
}

#[test]
fn test_vault_data_needs_refresh() {
    // Test: Vault data refresh logic works
    let mut vault_data = VaultData::new();

    // Initially needs refresh (no data)
    assert!(vault_data.needs_refresh());

    // Set last update to now
    vault_data.last_update = Some(std::time::Instant::now());
    assert!(!vault_data.needs_refresh());
}
