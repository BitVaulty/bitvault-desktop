//! End-to-End UI Testing for BitVault Desktop
//!
//! Tests complete user workflows by:
//! 1. Testing navigation flows
//! 2. Testing state management
//! 3. Testing UI component rendering
//! 4. Verifying UI state changes

#[path = "../src/state/navigation.rs"]
mod navigation;
use navigation::{Navigation, View};

#[test]
fn test_navigation_workflow() {
    // Test: Navigation between views works correctly
    let mut navigation = Navigation::new();

    // Start at vault selection
    navigation.navigate_to(View::VaultSelection);
    assert!(matches!(navigation.current_view, View::VaultSelection));

    // Navigate to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 0 }
    ));

    // Navigate to send transaction
    navigation.navigate_to(View::SendTransaction);
    assert!(matches!(navigation.current_view, View::SendTransaction));

    // Test back navigation
    navigation.go_back();
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
}

#[test]
fn test_dashboard_tab_switching() {
    // Test: Dashboard tabs can be switched
    let mut navigation = Navigation::new();

    // Set dashboard view
    navigation.set_view(View::Dashboard { tab: 0 });
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 0 }
    ));

    // Switch to history tab
    navigation.set_dashboard_tab(1);
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 1 }
    ));

    // Switch to settings tab
    navigation.set_dashboard_tab(2);
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 2 }
    ));
}

#[test]
fn test_navigation_history() {
    // Test: Navigation history is maintained correctly
    let mut navigation = Navigation::new();

    // Navigate through multiple views
    navigation.navigate_to(View::Dashboard { tab: 0 });
    navigation.navigate_to(View::SendTransaction);
    navigation.navigate_to(View::Receive);

    // Verify history
    assert_eq!(navigation.history.len(), 3);
    assert_eq!(navigation.history[0], View::VaultSelection);
    assert_eq!(navigation.history[1], View::Dashboard { tab: 0 });
    assert_eq!(navigation.history[2], View::SendTransaction);

    // Go back multiple times
    navigation.go_back();
    assert!(matches!(navigation.current_view, View::SendTransaction));

    navigation.go_back();
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
}

#[test]
fn test_navigation_with_data() {
    // Test: Navigation with data works correctly
    let mut navigation = Navigation::new();

    // Navigate with data
    navigation.navigate_to_with_data(
        View::TransactionDetail {
            txid: "test123".to_string(),
        },
        Some("test_data".to_string()),
    );

    assert!(matches!(
        navigation.current_view,
        View::TransactionDetail { txid: _ }
    ));
    assert_eq!(navigation.navigation_data, Some("test_data".to_string()));

    // Take navigation data
    let data = navigation.take_navigation_data();
    assert_eq!(data, Some("test_data".to_string()));
    assert!(navigation.navigation_data.is_none());
}

// Note: UI component rendering tests require full app context
// These are better tested through integration tests or manual testing
// Navigation and state management tests are covered here

#[test]
fn test_view_equality() {
    // Test: View enum equality works correctly
    let view1 = View::Dashboard { tab: 0 };
    let view2 = View::Dashboard { tab: 0 };
    let view3 = View::Dashboard { tab: 1 };

    assert_eq!(view1, view2, "Same views should be equal");
    assert_ne!(view1, view3, "Different tabs should not be equal");

    let view4 = View::VaultSelection;
    let view5 = View::VaultSelection;
    assert_eq!(view4, view5, "Same view types should be equal");
}

#[test]
fn test_navigation_initial_state() {
    // Test: Navigation starts in correct initial state
    let navigation = Navigation::new();

    assert_eq!(navigation.current_view, View::VaultSelection);
    assert!(navigation.history.is_empty());
    assert!(navigation.navigation_data.is_none());
}
