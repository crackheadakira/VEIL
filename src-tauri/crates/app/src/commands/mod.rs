use std::path::PathBuf;

use crate::{
    config::{SodapopConfig, config_path},
    data_path,
    error::FrontendError,
};

pub(crate) mod db;
pub(crate) mod lastfm;
pub(crate) mod music_folder;
pub(crate) mod player;
pub(crate) mod plugins;

fn custom_style_path() -> PathBuf {
    data_path().join("custom.css")
}

#[tauri::command]
#[specta::specta]
pub fn read_custom_style() -> Result<String, FrontendError> {
    let path = custom_style_path();
    Ok(std::fs::read_to_string(&path)?)
}

#[tauri::command]
#[specta::specta]
pub fn read_config() -> Result<SodapopConfig, FrontendError> {
    let path = config_path();
    let json_reader = std::fs::File::open(path)?;
    Ok(serde_json::from_reader(json_reader)?)
}
