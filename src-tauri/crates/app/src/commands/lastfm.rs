use crate::{TauriState, error::FrontendError, systems};

#[tauri::command]
#[specta::specta]
pub async fn get_token(state: TauriState<'_>) -> Result<(String, String), FrontendError> {
    let lastfm = state.lastfm.lock().await;
    systems::lastfm::get_token(lastfm).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_session(state: TauriState<'_>, token: String) -> Result<(), FrontendError> {
    let lastfm = state.lastfm.lock().await;
    let session_key = systems::lastfm::get_session(lastfm, token).await?;

    let config = logging::lock_or_log(state.config.write(), "Config Write")?;
    systems::lastfm::write_session_to_config(config, session_key)
}
