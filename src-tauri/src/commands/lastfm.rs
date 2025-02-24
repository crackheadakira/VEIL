use crate::{error::FrontendError, StateMutex};
use lastfm::{Auth, LastFMData};

#[tauri::command]
#[specta::specta]
pub async fn get_token(state: StateMutex<'_>) -> Result<(String, String), FrontendError> {
    let a = state.lastfm.auth().token().send().await?;

    let mut url = String::new();
    url.push_str("http://www.last.fm/api/auth/?api_key=");
    url.push_str(&state.lastfm.api_key());
    url.push_str("&token=");
    url.push_str(&a.token);

    Ok((url, a.token))
}

#[tauri::command]
#[specta::specta]
pub async fn get_session(state: StateMutex<'_>, token: String) -> Result<(), FrontendError> {
    let a = state.lastfm.auth().session(token).send().await?;

    let mut config = state.config.write().unwrap();
    config.last_fm_key = Some(a.session.key);
    config.write_config()?;

    Ok(())
}
