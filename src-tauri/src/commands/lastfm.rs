use crate::{error::FrontendError, StateMutex};
use lastfm::{Auth, LastFMData};

#[tauri::command]
#[specta::specta]
pub fn get_token(state: StateMutex) -> Result<(String, String), FrontendError> {
    let state_guard = state.lock().unwrap();
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
pub fn get_session(token: String, state: StateMutex) -> Result<(), FrontendError> {
    let mut state_guard = state.lock().unwrap();
    let a = state_guard.lastfm.auth().session(token).send()?;

    state_guard.config.last_fm_key = Some(a.session.key);
    state_guard.config.write_config()?;

    Ok(())
}
