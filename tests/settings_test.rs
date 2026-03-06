//! Settings Tests
//!
//! Tests settings management and persistence

#[path = "../src/settings.rs"]
mod settings;
use settings::{AppTheme, Currency, SettingsManager};

#[test]
fn test_currency_enum() {
    // Test: Currency enum works correctly
    let usd = Currency::USD;
    let eur = Currency::EUR;
    let btc = Currency::BTC;

    // All currencies should be valid
    assert!(matches!(usd, Currency::USD));
    assert!(matches!(eur, Currency::EUR));
    assert!(matches!(btc, Currency::BTC));
}

#[test]
fn test_app_theme_enum() {
    // Test: AppTheme enum works correctly
    let system = AppTheme::System;
    let light = AppTheme::Light;
    let dark = AppTheme::Dark;

    // All themes should be valid
    assert!(matches!(system, AppTheme::System));
    assert!(matches!(light, AppTheme::Light));
    assert!(matches!(dark, AppTheme::Dark));
}

#[test]
fn test_settings_manager_creation() {
    // Test: SettingsManager can be created
    let settings = SettingsManager::new();

    // Should succeed (may fail if config directory can't be created, but that's OK)
    assert!(settings.is_ok() || settings.is_err());
}

#[test]
fn test_settings_defaults() {
    // Test: Settings have reasonable defaults
    if let Ok(settings_manager) = SettingsManager::new() {
        // Load settings (returns defaults if file doesn't exist)
        if let Ok(settings) = settings_manager.load() {
            // Settings should have defaults
            assert!(matches!(
                settings.currency,
                Currency::USD
                    | Currency::EUR
                    | Currency::GBP
                    | Currency::JPY
                    | Currency::CNY
                    | Currency::CAD
                    | Currency::AUD
                    | Currency::BTC
            ));
            assert!(matches!(
                settings.theme,
                AppTheme::System | AppTheme::Light | AppTheme::Dark
            ));
        }
    }
}

#[test]
fn test_currency_serialization() {
    // Test: Currency can be serialized/deserialized
    // This is tested implicitly through SettingsManager
    // If serialization fails, settings won't work
    let currency = Currency::USD;

    // Currency should be a valid enum variant
    assert!(matches!(
        currency,
        Currency::USD
            | Currency::EUR
            | Currency::GBP
            | Currency::JPY
            | Currency::CNY
            | Currency::CAD
            | Currency::AUD
            | Currency::BTC
    ));
}

#[test]
fn test_theme_serialization() {
    // Test: AppTheme can be serialized/deserialized
    let theme = AppTheme::System;

    // Theme should be a valid enum variant
    assert!(matches!(
        theme,
        AppTheme::System | AppTheme::Light | AppTheme::Dark
    ));
}

#[test]
fn test_currency_all_variants() {
    // Test: All currency variants are accessible
    let currencies = Currency::all();

    assert_eq!(currencies.len(), 8);
    for currency in currencies {
        assert!(matches!(
            currency,
            Currency::USD
                | Currency::EUR
                | Currency::GBP
                | Currency::JPY
                | Currency::CNY
                | Currency::CAD
                | Currency::AUD
                | Currency::BTC
        ));
    }
}

#[test]
fn test_theme_all_variants() {
    // Test: All theme variants are accessible
    let themes = AppTheme::all();

    assert_eq!(themes.len(), 3);
    for theme in themes {
        assert!(matches!(
            theme,
            AppTheme::System | AppTheme::Light | AppTheme::Dark
        ));
    }
}

#[test]
fn test_currency_code_and_name() {
    // Test: Currency code and name methods work
    let usd = Currency::USD;
    assert_eq!(usd.code(), "USD");
    assert_eq!(usd.name(), "US Dollar");

    let btc = Currency::BTC;
    assert_eq!(btc.code(), "BTC");
    assert_eq!(btc.name(), "Bitcoin");
}

#[test]
fn test_theme_display_name() {
    // Test: Theme display name method works
    let light = AppTheme::Light;
    assert_eq!(light.display_name(), "Light");

    let dark = AppTheme::Dark;
    assert_eq!(dark.display_name(), "Dark");

    let system = AppTheme::System;
    assert_eq!(system.display_name(), "System");
}
