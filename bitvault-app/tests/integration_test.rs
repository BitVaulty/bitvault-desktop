//! Integration Tests for BitVault Desktop
//!
//! Tests complete workflows and component interactions
//! These tests verify that multiple components work together correctly

#[path = "../src/state/navigation.rs"]
mod navigation;
use navigation::{Navigation, View};

// Note: AppState tests require full app initialization
// These are better tested through unit tests in app_state module

#[test]
fn test_vault_selection_to_dashboard_flow() {
    // Test: Complete flow from vault selection to dashboard
    let mut navigation = Navigation::new();

    // Start at vault selection
    assert!(matches!(navigation.current_view, View::VaultSelection));

    // Simulate selecting a vault and navigating to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 0 }
    ));

    // Verify history is maintained
    assert_eq!(navigation.history.len(), 1);
    assert_eq!(navigation.history[0], View::VaultSelection);
}

#[test]
fn test_send_transaction_workflow() {
    // Test: Complete send transaction workflow navigation
    let mut navigation = Navigation::new();

    // Navigate to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });

    // Navigate to send transaction
    navigation.navigate_to(View::SendTransaction);
    assert!(matches!(navigation.current_view, View::SendTransaction));

    // Verify we can go back to dashboard
    navigation.go_back();
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
}

#[test]
fn test_receive_workflow() {
    // Test: Complete receive workflow navigation
    let mut navigation = Navigation::new();

    // Navigate to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });

    // Navigate to receive
    navigation.navigate_to(View::Receive);
    assert!(matches!(navigation.current_view, View::Receive));

    // Verify we can go back
    navigation.go_back();
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
}

#[test]
fn test_transaction_detail_workflow() {
    // Test: Transaction detail view workflow
    let mut navigation = Navigation::new();

    // Navigate to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });

    // Navigate to transaction detail with txid
    let txid = "test_tx_123".to_string();
    navigation.navigate_to(View::TransactionDetail { txid: txid.clone() });
    assert!(matches!(
        navigation.current_view,
        View::TransactionDetail { txid: _ }
    ));

    // Verify we can go back
    navigation.go_back();
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
}

#[test]
fn test_settings_workflow() {
    // Test: Settings navigation workflow
    let mut navigation = Navigation::new();

    // Navigate to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });

    // Navigate to settings
    navigation.navigate_to(View::Settings);
    assert!(matches!(navigation.current_view, View::Settings));

    // Verify we can go back
    navigation.go_back();
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
}

#[test]
fn test_subscription_workflow() {
    // Test: Subscription view workflow
    let mut navigation = Navigation::new();

    // Navigate to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });

    // Navigate to subscription
    navigation.navigate_to(View::Subscription);
    assert!(matches!(navigation.current_view, View::Subscription));

    // Verify we can go back
    navigation.go_back();
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
}

#[test]
fn test_complex_navigation_flow() {
    // Test: Complex multi-step navigation flow
    let mut navigation = Navigation::new();

    // Start at vault selection
    assert_eq!(navigation.current_view, View::VaultSelection);

    // Navigate through multiple views
    navigation.navigate_to(View::Dashboard { tab: 0 });
    navigation.navigate_to(View::SendTransaction);
    navigation.navigate_to(View::Receive);
    navigation.navigate_to(View::Settings);

    // Verify current view
    assert!(matches!(navigation.current_view, View::Settings));

    // Verify history
    assert_eq!(navigation.history.len(), 4);

    // Go back multiple times
    navigation.go_back(); // Settings -> Receive
    assert!(matches!(navigation.current_view, View::Receive));

    navigation.go_back(); // Receive -> SendTransaction
    assert!(matches!(navigation.current_view, View::SendTransaction));

    navigation.go_back(); // SendTransaction -> Dashboard
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));

    navigation.go_back(); // Dashboard -> VaultSelection
    assert!(matches!(navigation.current_view, View::VaultSelection));

    // Can't go back from root
    let went_back = navigation.go_back();
    assert!(!went_back);
    assert_eq!(navigation.current_view, View::VaultSelection);
}

#[test]
fn test_dashboard_tab_navigation() {
    // Test: Dashboard tab navigation within dashboard
    let mut navigation = Navigation::new();

    // Navigate to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });

    // Switch tabs
    navigation.set_dashboard_tab(1);
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 1 }
    ));

    navigation.set_dashboard_tab(2);
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 2 }
    ));

    // Switch back to first tab
    navigation.set_dashboard_tab(0);
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 0 }
    ));
}

// Note: AppState initialization tests are in unit tests
// Integration tests focus on workflow navigation

#[test]
fn test_navigation_data_persistence() {
    // Test: Navigation data handling works correctly
    let mut navigation = Navigation::new();

    // Navigate with data
    navigation.navigate_to_with_data(View::SendTransaction, Some("test_data_1".to_string()));

    assert_eq!(navigation.navigation_data, Some("test_data_1".to_string()));

    // Navigate to another view (data is cleared on new navigation)
    // This is expected behavior - each navigation can have its own data
    navigation.navigate_to(View::Receive);
    // Data is cleared when navigating without data
    assert!(navigation.navigation_data.is_none());

    // Navigate with new data
    navigation.navigate_to_with_data(
        View::TransactionDetail {
            txid: "tx123".to_string(),
        },
        Some("test_data_2".to_string()),
    );

    assert_eq!(navigation.navigation_data, Some("test_data_2".to_string()));

    // Take data
    let data = navigation.take_navigation_data();
    assert_eq!(data, Some("test_data_2".to_string()));
    assert!(navigation.navigation_data.is_none());
}
