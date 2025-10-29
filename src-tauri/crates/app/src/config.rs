use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use crate::{
    error::FrontendError,
    queue::{QueueOrigin, RepeatMode},
    systems::utils::data_path,
};

#[derive(Serialize, Deserialize, Type, Clone, Default)]
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

#[derive(Serialize, Deserialize, PartialEq, Type, Clone, Copy, Debug, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Serialize, Deserialize, Type, Event, Clone, Default)]
pub struct SodapopConfigEvent {
    pub theme: Option<ThemeMode>,
    pub discord_enabled: Option<bool>,
    pub last_fm_enabled: Option<bool>,
    pub music_dir: Option<String>,
    pub last_fm_key: Option<String>,
    pub queue_origin: Option<QueueOrigin>,
    pub queue_idx: Option<usize>,
    pub repeat_mode: Option<RepeatMode>,
}

impl SodapopConfig {
    pub fn new() -> Result<Self, FrontendError> {
        let path = Self::config_file_path();
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
    fn update_config(&mut self, config: SodapopConfigEvent) {
        self.theme = config.theme.unwrap_or(self.theme);
        self.music_dir = config.music_dir.or(self.music_dir.take());
        self.last_fm_key = config.last_fm_key.or(self.last_fm_key.take());
        self.discord_enabled = config.discord_enabled.unwrap_or(self.discord_enabled);
        self.last_fm_enabled = config.last_fm_enabled.unwrap_or(self.last_fm_enabled);
        self.queue_origin = config.queue_origin.or(self.queue_origin.take());
        self.queue_idx = config.queue_idx.unwrap_or(self.queue_idx);
        self.repeat_mode = config.repeat_mode.unwrap_or(self.repeat_mode);
    }

    /// Update config field values and writes it to disk
    pub fn update_config_and_write(
        &mut self,
        config: SodapopConfigEvent,
    ) -> Result<(), FrontendError> {
        self.update_config(config);
        self.write_config()?;

        Ok(())
    }

    pub fn write_config(&self) -> Result<(), FrontendError> {
        fs::write(Self::config_file_path(), serde_json::to_string(&self)?)?;
        Ok(())
    }

    pub fn config_file_path() -> PathBuf {
        data_path().join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_theme() {
        let mut config = SodapopConfig::default();

        assert_eq!(config.theme, ThemeMode::Dark);

        config.update_config({
            SodapopConfigEvent {
                theme: Some(ThemeMode::Light),
                ..SodapopConfigEvent::default()
            }
        });

        assert_eq!(config.theme, ThemeMode::Light);
    }

    #[test]
    fn update_music_dir() {
        let mut config = SodapopConfig::default();

        assert_eq!(config.music_dir, None);

        config.update_config({
            SodapopConfigEvent {
                music_dir: Some("hello".to_owned()),
                ..SodapopConfigEvent::default()
            }
        });

        assert_eq!(config.music_dir, Some("hello".to_owned()));
    }

    #[test]
    fn update_discord_enabled() {
        let mut config = SodapopConfig::default();

        assert_eq!(config.discord_enabled, false);

        config.update_config({
            SodapopConfigEvent {
                discord_enabled: Some(true),
                ..SodapopConfigEvent::default()
            }
        });

        assert_eq!(config.discord_enabled, true);
    }

    #[test]
    fn update_last_fm_enabled() {
        let mut config = SodapopConfig::default();

        assert_eq!(config.last_fm_enabled, false);

        config.update_config({
            SodapopConfigEvent {
                last_fm_enabled: Some(true),
                ..SodapopConfigEvent::default()
            }
        });

        assert_eq!(config.last_fm_enabled, true);
    }

    #[test]
    fn update_last_fm_key() {
        let mut config = SodapopConfig::default();

        assert_eq!(config.last_fm_key, None);

        config.update_config({
            SodapopConfigEvent {
                last_fm_key: Some("hello".to_owned()),
                ..SodapopConfigEvent::default()
            }
        });

        assert_eq!(config.last_fm_key, Some("hello".to_owned()));
    }

    #[test]
    fn update_queue_origin() {
        let mut config = SodapopConfig::default();
        let origin = QueueOrigin::Album { id: 0 };

        assert_eq!(config.queue_origin, None);

        config.update_config({
            SodapopConfigEvent {
                queue_origin: Some(origin.clone()),
                ..SodapopConfigEvent::default()
            }
        });

        assert_eq!(config.queue_origin, Some(origin));
    }

    #[test]
    fn update_queue_idx() {
        let mut config = SodapopConfig::default();

        assert_eq!(config.queue_idx, usize::MIN);

        config.update_config({
            SodapopConfigEvent {
                queue_idx: Some(usize::MAX),
                ..SodapopConfigEvent::default()
            }
        });

        assert_eq!(config.queue_idx, usize::MAX);
    }

    #[test]
    fn update_repeat_mode() {
        let mut config = SodapopConfig::default();

        assert_eq!(config.repeat_mode, RepeatMode::None);

        config.update_config({
            SodapopConfigEvent {
                repeat_mode: Some(RepeatMode::Track),
                ..SodapopConfigEvent::default()
            }
        });

        assert_eq!(config.repeat_mode, RepeatMode::Track);
    }
}
