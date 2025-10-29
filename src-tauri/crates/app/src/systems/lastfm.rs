use crate::{config::SodapopConfig, error::FrontendError};
use lastfm::LastFMData;
use std::sync::RwLockWriteGuard;
use tokio::sync::MutexGuard;

pub async fn get_token(
    lastfm: MutexGuard<'_, lastfm::LastFM>,
) -> Result<(String, String), FrontendError> {
    let a = lastfm.auth().token().send().await?;

    let mut url = String::new();
    url.push_str("http://www.last.fm/api/auth/?api_key=");
    url.push_str(&lastfm.api_key());
    url.push_str("&token=");
    url.push_str(&a.token);

    Ok((url, a.token))
}

pub async fn get_session(
    mut lastfm: MutexGuard<'_, lastfm::LastFM>,
    token: String,
) -> Result<String, FrontendError> {
    let a = lastfm.auth().session(&token).send().await?;
    lastfm.set_session_key(a.session.key.clone());

    Ok(a.session.key)
}

pub fn write_session_to_config(
    mut config: RwLockWriteGuard<'_, SodapopConfig>,
    session_key: String,
) -> Result<(), FrontendError> {
    config.last_fm_key = Some(session_key);
    config.write_config()?;

    Ok(())
}
