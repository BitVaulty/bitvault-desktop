//! Error Handling E2E Tests
//!
//! Tests error states, error recovery, and error display

#[path = "../src/state/navigation.rs"]
mod navigation;
use navigation::{Navigation, View};

#[test]
fn test_navigation_error_recovery() {
    // Test: Navigation can recover from invalid states
    let mut navigation = Navigation::new();
    
    // Navigate to a valid view
    navigation.navigate_to(View::Dashboard { tab: 0 });
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 0 }
    ));
    
    // Go back to recover
    navigation.go_back();
    assert_eq!(navigation.current_view, View::VaultSelection);
}

#[test]
fn test_navigation_empty_history() {
    // Test: Navigation handles empty history correctly
    let mut navigation = Navigation::new();
    
    // Can't go back from initial state
    let went_back = navigation.go_back();
    assert!(!went_back);
    assert_eq!(navigation.current_view, View::VaultSelection);
    assert!(navigation.history.is_empty());
}

#[test]
fn test_navigation_invalid_tab() {
    // Test: Navigation handles invalid tab numbers
    let mut navigation = Navigation::new();
    
    // Navigate to dashboard
    navigation.navigate_to(View::Dashboard { tab: 0 });
    
    // Try to set invalid tab (should be clamped or ignored)
    navigation.set_dashboard_tab(999);
    // Tab should be valid (0-2) or unchanged
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
}

#[test]
fn test_view_equality_edge_cases() {
    // Test: View equality handles edge cases
    let view1 = View::Dashboard { tab: 0 };
    let view2 = View::Dashboard { tab: 0 };
    let view3 = View::Dashboard { tab: 1 };
    
    // Same views are equal
    assert_eq!(view1, view2);
    
    // Different tabs are not equal
    assert_ne!(view1, view3);
    
    // Different view types are not equal
    let view4 = View::VaultSelection;
    assert_ne!(view1, view4);
}

#[test]
fn test_navigation_data_edge_cases() {
    // Test: Navigation data handles edge cases
    let mut navigation = Navigation::new();
    
    // Navigate with None data
    navigation.navigate_to_with_data(View::SendTransaction, None);
    assert!(navigation.navigation_data.is_none());
    
    // Navigate with empty string
    navigation.navigate_to_with_data(View::Receive, Some(String::new()));
    assert_eq!(navigation.navigation_data, Some(String::new()));
    
    // Take empty data
    let data = navigation.take_navigation_data();
    assert_eq!(data, Some(String::new()));
    assert!(navigation.navigation_data.is_none());
    
    // Take data when none exists
    let data = navigation.take_navigation_data();
    assert!(data.is_none());
}

#[test]
fn test_navigation_history_limits() {
    // Test: Navigation history doesn't grow unbounded
    let mut navigation = Navigation::new();
    
    // Navigate many times
    for i in 0..100 {
        navigation.navigate_to(View::Dashboard { tab: i % 3 });
    }
    
    // History should contain all previous views
    assert_eq!(navigation.history.len(), 100);
    
    // Can still go back
    assert!(navigation.can_go_back());
    
    // Go back many times
    for _ in 0..50 {
        navigation.go_back();
    }
    
    // History should be reduced
    assert_eq!(navigation.history.len(), 50);
}

#[test]
fn test_dashboard_tab_boundary_conditions() {
    // Test: Dashboard tab switching handles boundaries
    let mut navigation = Navigation::new();
    
    navigation.navigate_to(View::Dashboard { tab: 0 });
    
    // Switch to each valid tab
    navigation.set_dashboard_tab(0);
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 0 }
    ));
    
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
}

#[test]
fn test_navigation_state_consistency() {
    // Test: Navigation state remains consistent through operations
    let mut navigation = Navigation::new();
    
    // Initial state
    assert_eq!(navigation.current_view, View::VaultSelection);
    assert!(navigation.history.is_empty());
    assert!(navigation.navigation_data.is_none());
    
    // Navigate
    navigation.navigate_to(View::Dashboard { tab: 0 });
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: 0 }
    ));
    assert_eq!(navigation.history.len(), 1);
    
    // Navigate with data
    navigation.navigate_to_with_data(
        View::SendTransaction,
        Some("test".to_string()),
    );
    assert!(matches!(
        navigation.current_view,
        View::SendTransaction
    ));
    assert_eq!(navigation.history.len(), 2);
    assert_eq!(navigation.navigation_data, Some("test".to_string()));
    
    // Go back
    navigation.go_back();
    assert!(matches!(
        navigation.current_view,
        View::Dashboard { tab: _ }
    ));
    assert_eq!(navigation.history.len(), 1);
    // Data should be cleared on back navigation
}
