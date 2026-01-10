# End-to-End Testing Guide for BitVault Desktop

## Overview

This guide explains how to run and write end-to-end (E2E) tests for the BitVault desktop application. E2E tests verify complete user workflows by testing UI components, state management, and user interactions.

## Running E2E Tests

### Run All E2E Tests
```bash
cd bitvault-desktop/bitvault-app
cargo test --test e2e_test
```

### Run Specific Test
```bash
cargo test --test e2e_test test_vault_selection_empty_state
```

### Run with Output
```bash
cargo test --test e2e_test -- --nocapture
```

### Run All Tests (Including E2E)
```bash
cargo test
```

## Test Structure

E2E tests are located in `bitvault-desktop/bitvault-app/tests/e2e_test.rs`.

### Test Categories

1. **UI Component Tests**: Verify components render without errors
2. **Navigation Tests**: Verify navigation between screens works
3. **Form Validation Tests**: Verify input validation and error handling
4. **State Management Tests**: Verify state changes correctly
5. **Workflow Tests**: Verify complete user workflows

## Writing E2E Tests

### Basic Test Structure

```rust
#[test]
fn test_example() {
    // 1. Create test context
    let ctx = create_test_context();
    let mut app_state = create_test_app_state();
    let mut navigation = crate::state::Navigation::new();
    
    // 2. Render UI component
    egui::CentralPanel::default().show(&ctx, |ui| {
        // Render your component here
    });
    
    // 3. Verify expected behavior
    assert!(condition, "Error message");
}
```

### Testing UI Components

```rust
#[test]
fn test_component_renders() {
    let ctx = create_test_context();
    
    egui::CentralPanel::default().show(&ctx, |ui| {
        crate::ui::components::card(ui, |ui| {
            ui.label("Test content");
        });
    });
    
    // Component should render without panicking
    assert!(true, "Component should render");
}
```

### Testing Navigation

```rust
#[test]
fn test_navigation() {
    let mut navigation = crate::state::Navigation::new();
    
    navigation.navigate_to(crate::state::View::Dashboard { tab: 0 });
    assert!(matches!(
        navigation.current_view,
        crate::state::View::Dashboard { tab: 0 }
    ));
}
```

### Testing Form Validation

```rust
#[test]
fn test_form_validation() {
    let ctx = create_test_context();
    let mut state = SomeState::default();
    
    // Set invalid input
    state.input = String::new();
    
    // Render form
    egui::CentralPanel::default().show(&ctx, |ui| {
        render_form(ui, &mut state);
    });
    
    // Verify validation
    assert!(state.error.is_some(), "Should have validation error");
}
```

## Test Helpers

Test helpers are located in `tests/test_helpers.rs`:

- `create_test_context()`: Create a test egui context
- `create_test_context_light()`: Create a light mode context
- `create_test_app_state()`: Create a test app state
- `simulate_click()`: Simulate a click interaction
- `simulate_text_input()`: Simulate text input

## Best Practices

### 1. Test Critical User Journeys

Focus on testing the most important workflows:
- Vault creation
- Transaction sending
- Transaction receiving
- Settings changes

### 2. Test Error States

Verify error handling works correctly:
- Invalid inputs
- Network errors
- Missing data

### 3. Test Loading States

Verify loading indicators display correctly:
- Initial loading
- Refresh loading
- Async operation loading

### 4. Keep Tests Independent

Each test should be independent and not rely on other tests:
- Use fresh state for each test
- Don't share mutable state between tests
- Clean up after tests if needed

### 5. Test Both Light and Dark Modes

Verify UI works in both themes:
```rust
#[test]
fn test_dark_mode() {
    let ctx = create_test_context(); // Dark mode by default
    // Test...
}

#[test]
fn test_light_mode() {
    let ctx = create_test_context_light();
    // Test...
}
```

## Limitations

### egui Immediate Mode

Since egui uses immediate mode rendering:
- We can't easily simulate mouse/keyboard events
- We test state changes and rendering, not actual user interactions
- Tests verify components render without errors

### Async Operations

For async operations:
- Tests verify the UI handles loading states
- Tests verify error states are displayed
- Actual async operations are tested in integration tests

## CI/CD Integration

E2E tests run automatically in CI/CD:
- All tests must pass before merging
- Tests run on every pull request
- Test results are reported in CI logs

## Debugging Failed Tests

### Run with Output
```bash
cargo test --test e2e_test -- --nocapture
```

### Run Single Test
```bash
cargo test --test e2e_test test_name -- --nocapture
```

### Check Test Coverage
```bash
cargo test --test e2e_test -- --test-threads=1
```

## Future Improvements

Potential enhancements to the E2E testing framework:

1. **Screenshot Testing**: Compare rendered outputs
2. **Visual Regression Testing**: Detect UI changes
3. **Interaction Simulation**: Better simulation of user interactions
4. **Performance Testing**: Measure render times
5. **Accessibility Testing**: Verify accessibility features

## Related Documentation

- [Testing Strategy](../design/testing-strategy.md)
- [Development Guide](./development.md)
- [UI Components](../design/ui-components.md)
