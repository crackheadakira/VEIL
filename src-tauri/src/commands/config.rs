use crate::{config::SodapopConfig, StateMutex};

#[tauri::command]
#[specta::specta]
pub fn get_config(state: StateMutex) -> SodapopConfig {
    let state_guard = state.lock().unwrap();
    state_guard.config.clone()
}
