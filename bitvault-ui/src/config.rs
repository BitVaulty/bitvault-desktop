use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// Settings struct to persist application settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub window_width: f32,
    pub window_height: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            window_width: 800.0,
            window_height: 600.0,
        }
    }
}

impl Settings {
    // Helper function to get the settings file path
    pub fn get_settings_file_path() -> Option<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("bitvault");

            // Create directory if it doesn't exist
            if !app_config_dir.exists() && fs::create_dir_all(&app_config_dir).is_err() {
                return None;
            }

            return Some(app_config_dir.join("settings.toml"));
        }
        None
    }

    // Load settings from TOML file
    pub fn load() -> Self {
        if let Some(file_path) = Self::get_settings_file_path() {
            if file_path.exists() {
                match fs::read_to_string(&file_path) {
                    Ok(toml_str) => match toml::from_str::<Settings>(&toml_str) {
                        Ok(settings) => {
                            log::info!("Settings loaded successfully");
                            return settings;
                        }
                        Err(e) => {
                            log::error!("Failed to parse settings file: {}", e);
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to read settings file: {}", e);
                    }
                }
            } else {
                log::info!("No settings file found, using defaults");
            }
        } else {
            log::error!("Could not determine settings file path");
        }

        // Return default settings if we couldn't load from file
        Settings::default()
    }

    // Save settings to TOML file
    pub fn save(&self) -> Result<(), String> {
        if let Some(file_path) = Self::get_settings_file_path() {
            match toml::to_string(self) {
                Ok(toml_str) => fs::write(file_path, toml_str)
                    .map_err(|e| format!("Failed to save settings: {}", e)),
                Err(e) => Err(format!("Failed to serialize settings: {}", e)),
            }
        } else {
            Err("Could not determine settings file path".to_string())
        }
    }

    // Update window size settings
    pub fn update_window_size(&mut self, width: f32, height: f32) -> bool {
        if self.window_width != width || self.window_height != height {
            self.window_width = width;
            self.window_height = height;

            // Save settings right away
            if let Err(e) = self.save() {
                log::error!("Failed to save window size: {}", e);
                return false;
            }

            log::debug!("Window size saved: {}x{}", width, height);
            return true;
        }
        false
    }
}
