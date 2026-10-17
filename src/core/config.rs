use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use log::info;

/// Persistent user configuration for HyperShot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Default directory where screenshots are saved
    pub save_directory: String,
    /// Default image format: "png", "webp", or "jpeg"
    pub default_format: String,
    /// Whether to automatically copy the screenshot to clipboard when saving to file
    pub copy_to_clipboard_on_save: bool,
    /// Primary global capture hotkey name
    pub primary_hotkey: String,
    /// Fallback global hotkey name
    pub fallback_hotkey: String,
    /// Default active drawing color in RGBA
    pub default_color: [u8; 4],
    /// Default brush stroke thickness in pixels
    pub default_stroke_size: i32,
    /// Enable 8x magnifier loupe before selection
    pub enable_loupe: bool,
    /// Darkness opacity of the fullscreen backdrop (0 to 255)
    pub dark_backdrop_level: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_save_dir = dirs_fallback_pictures();

        Self {
            save_directory: default_save_dir,
            default_format: "png".to_string(),
            copy_to_clipboard_on_save: true,
            primary_hotkey: "PrintScreen".to_string(),
            fallback_hotkey: "Ctrl+Shift+X".to_string(),
            default_color: [235, 47, 47, 255], // Vibrant Red
            default_stroke_size: 4,
            enable_loupe: true,
            dark_backdrop_level: 150, // 40% darkness
        }
    }
}

impl AppConfig {
    const CONFIG_FILE_NAME: &'static str = "config.json";

    /// Returns the configuration path inside the user's AppData (`%APPDATA%\HyperShot\config.json`).
    ///
    /// The current working directory is never consulted, so a `config.json` planted in the
    /// directory HyperShot is launched from cannot override the user's settings. Returns `None`
    /// when `APPDATA` is unset or not an absolute path.
    pub fn config_path() -> Option<PathBuf> {
        let app_data = std::env::var("APPDATA").ok()?;
        let app_data = Path::new(&app_data);
        if !app_data.is_absolute() {
            return None;
        }

        let app_dir = app_data.join("HyperShot");
        let _ = fs::create_dir_all(&app_dir);
        Some(app_dir.join(Self::CONFIG_FILE_NAME))
    }

    /// Loads the configuration from disk, creating a default one if it does not exist.
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                        info!("Loaded user configuration from: {}", path.display());
                        return config;
                    }
                }
            }
        }

        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    /// Persists configuration to disk as formatted JSON.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()
            .ok_or_else(|| "Cannot determine config location: APPDATA is not set to an absolute path".to_string())?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {e}"))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write config file {}: {e}", path.display()))?;

        info!("Configuration saved to: {}", path.display());
        Ok(())
    }
}

fn dirs_fallback_pictures() -> String {
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let pictures = Path::new(&userprofile).join("Pictures").join("HyperShot");
        return pictures.to_string_lossy().to_string();
    }
    "./captures".to_string()
}
