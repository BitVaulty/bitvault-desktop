//! Tests for navigation system

#[path = "../src/state/navigation.rs"]
mod navigation;
use navigation::{Navigation, View};

#[test]
fn test_navigation_initial_state() {
    let nav = Navigation::new();
    assert_eq!(nav.current_view, View::VaultSelection);
    assert!(nav.history.is_empty());
    assert!(nav.navigation_data.is_none());
}

#[test]
fn test_navigation_navigate_to() {
    let mut nav = Navigation::new();

    nav.navigate_to(View::Dashboard { tab: 0 });
    assert_eq!(nav.current_view, View::Dashboard { tab: 0 });
    assert_eq!(nav.history.len(), 1);
    assert_eq!(nav.history[0], View::VaultSelection);
}

#[test]
fn test_navigation_go_back() {
    let mut nav = Navigation::new();

    nav.navigate_to(View::Dashboard { tab: 0 });
    nav.navigate_to(View::SendTransaction);

    assert_eq!(nav.current_view, View::SendTransaction);
    assert_eq!(nav.history.len(), 2);

    let went_back = nav.go_back();
    assert!(went_back);
    assert_eq!(nav.current_view, View::Dashboard { tab: 0 });
    assert_eq!(nav.history.len(), 1);
}

#[test]
fn test_navigation_go_back_from_root() {
    let mut nav = Navigation::new();

    let went_back = nav.go_back();
    assert!(!went_back);
    assert_eq!(nav.current_view, View::VaultSelection);
}

#[test]
fn test_navigation_set_dashboard_tab() {
    let mut nav = Navigation::new();

    nav.navigate_to(View::Dashboard { tab: 0 });
    nav.set_dashboard_tab(1);

    assert_eq!(nav.current_view, View::Dashboard { tab: 1 });
}

#[test]
fn test_navigation_with_data() {
    let mut nav = Navigation::new();

    nav.navigate_to_with_data(View::SendTransaction, Some("bc1test".to_string()));

    assert_eq!(nav.current_view, View::SendTransaction);
    assert_eq!(nav.navigation_data, Some("bc1test".to_string()));

    let data = nav.take_navigation_data();
    assert_eq!(data, Some("bc1test".to_string()));
    assert!(nav.navigation_data.is_none());
}
