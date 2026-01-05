//! Settings management for the desktop app
//!
//! Handles persistent storage of user preferences

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CNY,
    CAD,
    AUD,
    BTC,
}

impl Currency {
    pub fn code(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
            Currency::CNY => "CNY",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
            Currency::BTC => "BTC",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Currency::USD => "US Dollar",
            Currency::EUR => "Euro",
            Currency::GBP => "British Pound",
            Currency::JPY => "Japanese Yen",
            Currency::CNY => "Chinese Yuan",
            Currency::CAD => "Canadian Dollar",
            Currency::AUD => "Australian Dollar",
            Currency::BTC => "Bitcoin",
        }
    }

    pub fn all() -> Vec<Currency> {
        vec![
            Currency::USD,
            Currency::EUR,
            Currency::GBP,
            Currency::JPY,
            Currency::CNY,
            Currency::CAD,
            Currency::AUD,
            Currency::BTC,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppTheme {
    Light,
    Dark,
    System,
}

impl AppTheme {
    pub fn display_name(&self) -> &'static str {
        match self {
            AppTheme::Light => "Light",
            AppTheme::Dark => "Dark",
            AppTheme::System => "System",
        }
    }

    pub fn annotation(&self) -> &'static str {
        match self {
            AppTheme::Light => "Always use light theme",
            AppTheme::Dark => "Always use dark theme",
            AppTheme::System => "Follow system preference",
        }
    }

    pub fn all() -> Vec<AppTheme> {
        vec![AppTheme::Light, AppTheme::Dark, AppTheme::System]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub currency: Currency,
    pub theme: AppTheme,
    pub network: Option<String>, // Stored as string for persistence
    #[serde(default)]
    pub biometrics_enabled: Option<bool>, // None = not set, Some(true/false) = user preference
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            currency: Currency::USD,
            theme: AppTheme::System,
            network: None,
            biometrics_enabled: None, // Default to None (not configured)
        }
    }
}

pub struct SettingsManager {
    settings_path: PathBuf,
}

impl SettingsManager {
    pub fn new() -> Result<Self, String> {
        let settings_dir = Self::get_settings_directory()?;
        let settings_path = settings_dir.join("settings.json");

        Ok(Self { settings_path })
    }

    fn get_settings_directory() -> Result<PathBuf, String> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Failed to get config directory".to_string())?;
        let app_dir = config_dir.join("bitvault");

        // Create directory if it doesn't exist
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;

        Ok(app_dir)
    }

    pub fn load(&self) -> Result<AppSettings, String> {
        if !self.settings_path.exists() {
            return Ok(AppSettings::default());
        }

        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;

        let settings: AppSettings = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings: {}", e))?;

        Ok(settings)
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), String> {
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&self.settings_path, content)
            .map_err(|e| format!("Failed to write settings: {}", e))?;

        Ok(())
    }

    pub fn get_currency(&self) -> Result<Currency, String> {
        let settings = self.load()?;
        Ok(settings.currency)
    }

    pub fn set_currency(&self, currency: Currency) -> Result<(), String> {
        let mut settings = self.load()?;
        settings.currency = currency;
        self.save(&settings)
    }

    pub fn get_theme(&self) -> Result<AppTheme, String> {
        let settings = self.load()?;
        Ok(settings.theme)
    }

    pub fn set_theme(&self, theme: AppTheme) -> Result<(), String> {
        let mut settings = self.load()?;
        settings.theme = theme;
        self.save(&settings)
    }

    pub fn get_network(&self) -> Result<Option<String>, String> {
        let settings = self.load()?;
        Ok(settings.network)
    }

    pub fn set_network(&self, network: String) -> Result<(), String> {
        let mut settings = self.load()?;
        settings.network = Some(network);
        self.save(&settings)
    }

    pub fn get_biometrics_enabled(&self) -> Result<bool, String> {
        let settings = self.load()?;
        Ok(settings.biometrics_enabled.unwrap_or(false))
    }

    pub fn set_biometrics_enabled(&self, enabled: bool) -> Result<(), String> {
        let mut settings = self.load()?;
        settings.biometrics_enabled = Some(enabled);
        self.save(&settings)
    }
}
