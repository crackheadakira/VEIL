use crate::{config::VeilConfig, error::VeilError};
use lastfm::{LastFM, LastFMData};
use std::sync::RwLockWriteGuard;
use tokio::sync::MutexGuard;

pub async fn get_token(lastfm: MutexGuard<'_, LastFM>) -> Result<(String, String), VeilError> {
    let a = lastfm.auth().token().send().await?;

    let mut url = String::new();
    url.push_str("http://www.last.fm/api/auth/?api_key=");
    url.push_str(&lastfm.api_key());
    url.push_str("&token=");
    url.push_str(&a.token);

    Ok((url, a.token))
}

pub async fn get_session(
    mut lastfm: MutexGuard<'_, LastFM>,
    token: String,
) -> Result<String, VeilError> {
    let a = lastfm.auth().session(&token).send().await?;
    lastfm.set_session_key(a.session.key.clone());

    Ok(a.session.key)
}

pub fn write_session_to_config(
    mut config: RwLockWriteGuard<'_, VeilConfig>,
    session_key: String,
) -> Result<(), VeilError> {
    config.integrations.last_fm_session_key = Some(session_key);
    config.write_config()?;

    Ok(())
}
