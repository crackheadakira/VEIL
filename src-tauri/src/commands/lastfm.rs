use crate::{error::FrontendError, TauriState};
use lastfm::{Auth, LastFMData};

#[tauri::command]
#[specta::specta]
pub async fn get_token(state: TauriState<'_>) -> Result<(String, String), FrontendError> {
    let lastfm = state.lastfm.lock().await;
    let a = lastfm.auth().token().send().await?;

    let mut url = String::new();
    url.push_str("http://www.last.fm/api/auth/?api_key=");
    url.push_str(&lastfm.api_key());
    url.push_str("&token=");
    url.push_str(&a.token);

    Ok((url, a.token))
}

#[tauri::command]
#[specta::specta]
pub async fn get_session(state: TauriState<'_>, token: String) -> Result<(), FrontendError> {
    let mut lastfm = state.lastfm.lock().await;
    let a = lastfm.auth().session(token).send().await?;
    lastfm.set_session_key(a.session.key.clone());

    let mut config = state.config.write().unwrap();
    config.last_fm_key = Some(a.session.key);
    config.write_config()?;

    Ok(())
}
