use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::data_path;

#[derive(Serialize, Deserialize)]
pub struct SodapopConfig {
    pub theme: ThemeMode,
    pub music_dir: Option<String>,
    pub last_fm_key: Option<String>,
}

#[derive(Serialize, Deserialize, Type)]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

#[derive(Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct SodapopConfigEvent {
    pub theme: Option<ThemeMode>,
    pub music_dir: Option<String>,
    pub last_fm_key: Option<String>,
}

impl SodapopConfig {
    pub fn new() -> Result<Self, serde_json::Error> {
        let path = config_path();
        if Path::new(&path).exists() {
            Ok(serde_json::from_str(&path)?)
        } else {
            Ok(Self {
                theme: ThemeMode::Dark,
                music_dir: None,
                last_fm_key: None,
            })
        }
    }

    /// Update config field values
    pub fn update_config(&mut self, new_config: SodapopConfigEvent) -> Result<(), std::io::Error> {
        if let Some(t) = new_config.theme {
            self.theme = t;
        }

        if let Some(m) = new_config.music_dir {
            self.music_dir = Some(m);
        }

        if let Some(l) = new_config.last_fm_key {
            self.last_fm_key = Some(l);
        }

        self.write_config()?;

        Ok(())
    }

    fn write_config(&self) -> Result<(), std::io::Error> {
        fs::write(config_path(), serde_json::to_string(&self)?)?;
        Ok(())
    }
}

fn config_path() -> String {
    data_path() + "/config.json"
}
