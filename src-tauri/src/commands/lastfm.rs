use crate::{config::SodapopConfigEvent, error::FrontendError, SodapopState};
use lastfm::{Auth, LastFMData};

use std::sync::Mutex;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn get_token(state: State<'_, Mutex<SodapopState>>) -> Result<(String, String), FrontendError> {
    let state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    let a = state_guard.lastfm.auth().token().send()?;

    let mut url = String::new();
    url.push_str("http://www.last.fm/api/auth/?api_key=");
    url.push_str(&state_guard.lastfm.api_key());
    url.push_str("&token=");
    url.push_str(&a.token);

    Ok((url, a.token))
}

#[tauri::command]
#[specta::specta]
pub fn get_session(
    token: String,
    state: State<'_, Mutex<SodapopState>>,
) -> Result<(), FrontendError> {
    let mut state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    let a = state_guard.lastfm.auth().session(token).send()?;

    state_guard.config.update_config(SodapopConfigEvent {
        theme: None,
        music_dir: None,
        last_fm_key: Some(a.session.key),
        discord_enabled: None,
    })?;

    Ok(())
}
