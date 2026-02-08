//! Brukerinnstillinger for Bare
//!
//! Håndterer lagring og lasting av brukerpreferanser.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Feil som kan oppstå ved innstillingsoperasjoner
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Kunne ikke lese innstillinger: {0}")]
    Read(String),

    #[error("Kunne ikke lagre innstillinger: {0}")]
    Write(String),
}

/// Tema-valg
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

/// Skrifttype-valg
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FontFamily {
    #[default]
    System,
    Serif,
    SansSerif,
    Mono,
}

/// Konverteringsmodus for nettsider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ConversionMode {
    /// Kun vis native markdown-filer (.md)
    MarkdownOnly,
    /// Konverter HTML til markdown automatisk
    #[default]
    ConvertAll,
    /// Spør brukeren for hver side
    AskEverytime,
}

/// Brukerinnstillinger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Tema (lys/mørk/system)
    #[serde(default)]
    pub theme: Theme,

    /// Skriftstørrelse i prosent (100 = normal)
    #[serde(default = "default_font_size")]
    pub font_size: u32,

    /// Zoom-nivå i prosent (100 = normal)
    #[serde(default = "default_zoom")]
    pub zoom: u32,

    /// Skrifttype
    #[serde(default)]
    pub font_family: FontFamily,

    /// Maks innholdsbredde i piksler
    #[serde(default = "default_content_width")]
    pub content_width: u32,

    /// Vis linjenumre i kodeblokker
    #[serde(default)]
    pub show_line_numbers: bool,

    /// Konverteringsmodus for HTML-sider
    #[serde(default)]
    pub conversion_mode: ConversionMode,

    /// Aktiver readability-modus for å ekstrahere hovedinnhold
    #[serde(default = "default_readability")]
    pub readability_enabled: bool,

    /// Om brukeren har fullført onboarding
    #[serde(default)]
    pub onboarding_completed: bool,
}

fn default_font_size() -> u32 {
    100
}

fn default_zoom() -> u32 {
    100
}

fn default_content_width() -> u32 {
    800
}

fn default_readability() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            font_size: default_font_size(),
            zoom: default_zoom(),
            font_family: FontFamily::default(),
            content_width: default_content_width(),
            show_line_numbers: false,
            conversion_mode: ConversionMode::default(),
            readability_enabled: default_readability(),
            onboarding_completed: false,
        }
    }
}

impl Settings {
    /// Last innstillinger fra fil
    pub fn load(path: &PathBuf) -> Result<Self, SettingsError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).map_err(|e| SettingsError::Read(e.to_string()))?;

        serde_json::from_str(&content).map_err(|e| SettingsError::Read(e.to_string()))
    }

    /// Lagre innstillinger til fil
    pub fn save(&self, path: &PathBuf) -> Result<(), SettingsError> {
        // Opprett overordnede mapper hvis de ikke finnes
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| SettingsError::Write(e.to_string()))?;
        }

        let content =
            serde_json::to_string_pretty(self).map_err(|e| SettingsError::Write(e.to_string()))?;

        fs::write(path, content).map_err(|e| SettingsError::Write(e.to_string()))
    }

    /// Øk zoom-nivå
    pub fn zoom_in(&mut self) {
        if self.zoom < 200 {
            self.zoom = (self.zoom + 10).min(200);
        }
    }

    /// Senk zoom-nivå
    pub fn zoom_out(&mut self) {
        if self.zoom > 50 {
            self.zoom = (self.zoom - 10).max(50);
        }
    }

    /// Tilbakestill zoom til 100%
    pub fn zoom_reset(&mut self) {
        self.zoom = 100;
    }
}

/// Hent stien til innstillings-filen
pub fn get_settings_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("bare").join("settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.theme, Theme::Light);
        assert_eq!(settings.font_size, 100);
        assert_eq!(settings.zoom, 100);
    }

    #[test]
    fn test_zoom_in() {
        let mut settings = Settings::default();
        settings.zoom_in();
        assert_eq!(settings.zoom, 110);
    }

    #[test]
    fn test_zoom_out() {
        let mut settings = Settings::default();
        settings.zoom_out();
        assert_eq!(settings.zoom, 90);
    }

    #[test]
    fn test_zoom_limits() {
        let mut settings = Settings::default();
        settings.zoom = 200;
        settings.zoom_in();
        assert_eq!(settings.zoom, 200); // Skal ikke gå over 200

        settings.zoom = 50;
        settings.zoom_out();
        assert_eq!(settings.zoom, 50); // Skal ikke gå under 50
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let mut settings = Settings::default();
        settings.theme = Theme::Dark;
        settings.zoom = 120;

        settings.save(&path).unwrap();

        let loaded = Settings::load(&path).unwrap();
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.zoom, 120);
    }
}
