use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    data_path,
    error::FrontendError,
    events::SodapopConfigEvent,
    queue::{QueueOrigin, RepeatMode},
};

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct SodapopConfig {
    #[serde(default)]
    pub theme: ThemeMode,

    #[serde(default)]
    pub music_dir: Option<String>,

    #[serde(default)]
    pub discord_enabled: bool,

    #[serde(default)]
    pub last_fm_enabled: bool,

    #[serde(default)]
    pub last_fm_key: Option<String>,

    #[serde(default)]
    pub queue_origin: Option<QueueOrigin>,

    #[serde(default)]
    pub queue_idx: usize,

    #[serde(default)]
    pub repeat_mode: RepeatMode,
}

#[derive(Serialize, Deserialize, Type, Clone, Copy, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
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
                queue_origin: None,
                queue_idx: 0,
                repeat_mode: RepeatMode::None,
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
        self.queue_origin = new_config.queue_origin.or(self.queue_origin.take());
        self.queue_idx = new_config.queue_idx.unwrap_or(self.queue_idx);
        self.repeat_mode = new_config.repeat_mode.unwrap_or(self.repeat_mode);

        self.write_config()?;

        Ok(())
    }

    pub fn write_config(&self) -> Result<(), FrontendError> {
        fs::write(config_path(), serde_json::to_string(&self)?)?;
        Ok(())
    }
}

pub fn config_path() -> PathBuf {
    data_path().join("config.json")
}
