use std::sync::Mutex;

use tauri::State;

use crate::{config::SodapopConfig, SodapopState};

#[tauri::command]
#[specta::specta]
pub fn get_config(state: State<'_, Mutex<SodapopState>>) -> SodapopConfig {
    let state_guard = state.lock().unwrap();
    state_guard.config.clone()
}
