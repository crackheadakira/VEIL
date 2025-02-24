use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{data_path, error::FrontendError};

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct SodapopConfig {
    pub theme: ThemeMode,
    pub music_dir: Option<String>,
    pub discord_enabled: bool,
    pub last_fm_enabled: bool,
    pub last_fm_key: Option<String>,
}

#[derive(Serialize, Deserialize, Type, Clone, Copy)]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

#[derive(Serialize, Deserialize, Type, tauri_specta::Event, Clone)]
pub struct SodapopConfigEvent {
    pub theme: Option<ThemeMode>,
    pub music_dir: Option<String>,
    pub last_fm_key: Option<String>,
    pub discord_enabled: Option<bool>,
    pub last_fm_enabled: Option<bool>,
}

impl SodapopConfig {
    pub fn new() -> Result<Self, FrontendError> {
        let path = config_path();
        if path.exists() {
            let json_reader = fs::File::open(path).expect("error opening config.json reader");
            Ok(serde_json::from_reader(json_reader)?)
        } else {
            let config = Self {
                theme: ThemeMode::Dark,
                music_dir: None,
                last_fm_key: None,
                discord_enabled: false,
                last_fm_enabled: false,
            };
            config.write_config()?;
            Ok(config)
        }
    }

    /// Update config field values
    pub fn update_config(&mut self, new_config: SodapopConfigEvent) -> Result<(), FrontendError> {
        self.theme = new_config.theme.unwrap_or(self.theme);
        self.music_dir = new_config.music_dir.or(self.music_dir.take());
        self.last_fm_key = new_config.last_fm_key.or(self.last_fm_key.take());
        self.discord_enabled = new_config.discord_enabled.unwrap_or(self.discord_enabled);
        self.last_fm_enabled = new_config.last_fm_enabled.unwrap_or(self.last_fm_enabled);

        self.write_config()?;

        Ok(())
    }

    pub fn write_config(&self) -> Result<(), FrontendError> {
        fs::write(config_path(), serde_json::to_string(&self)?)?;
        Ok(())
    }
}

fn config_path() -> PathBuf {
    data_path().join("config.json")
}
