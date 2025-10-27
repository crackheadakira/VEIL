use common::Tracks;
use logging::lock_or_log;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::Manager;
use tauri_specta::{Event, TypedEvent};

use crate::{
    SodapopState,
    commands::player::{try_scrobble_track_to_lastfm, try_update_now_playing_to_lastfm},
    config::ThemeMode,
    discord,
    error::FrontendError,
};

#[derive(Serialize, Deserialize, Type, Event, Clone)]
pub struct SodapopConfigEvent {
    pub theme: Option<ThemeMode>,
    pub discord_enabled: Option<bool>,
    pub last_fm_enabled: Option<bool>,
    pub music_dir: Option<String>,
    pub last_fm_key: Option<String>,
}

#[derive(Serialize, Deserialize, Type, Event)]
pub struct NewTrackEvent {
    pub track: Tracks,
}

impl NewTrackEvent {
    /// Resets the player states to as if no track had been playing, then loads in the new track.
    ///
    /// Also handles Discord RPC & Last.FM scrobbling.
    pub async fn set_new_track(
        event: TypedEvent<NewTrackEvent>,
        handle: &tauri::AppHandle,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();

        let (last_fm_enabled, discord_enabled) = {
            let config = lock_or_log(state.config.read(), "Config Read")?;
            (config.last_fm_enabled, config.discord_enabled)
        };

        let should_scrobble = {
            let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;
            player.should_scrobble()
        };

        let track = event.payload.track;

        // Scrobble the previous track to Last.FM
        if let Some((track_id, track_timestamp)) = should_scrobble
            && last_fm_enabled
        {
            try_scrobble_track_to_lastfm(handle.clone(), track_id, track_timestamp).await?;
        }

        let (duration, progress) = {
            let mut player = lock_or_log(state.player.write(), "Player Write Lock").unwrap();

            // Reset the player's internal progress to 0
            player.set_progress(0.0);

            // Reset the player state to as if no track had been playing.
            if player.track.is_some() {
                player.stop()?;
            };

            player.play(&track)?;

            (player.duration, player.progress)
        };

        // Get album cover URL from Last.FM if Discord & Last.FM are enabled.
        let album_cover = if last_fm_enabled && discord_enabled {
            let state = handle.state::<SodapopState>();
            let lastfm = state.lastfm.lock().await;
            match lastfm
                .album()
                .info(&track.album_name, &track.artist_name)
                .send()
                .await
            {
                Ok(response) => response
                    .image
                    .iter()
                    .rev()
                    .find(|img| !img.url.is_empty())
                    .map(|img| img.url.clone()),
                Err(e) => {
                    logging::error!("LastFM album fetch error: {e}");
                    None
                }
            }
        } else {
            None
        };

        if discord_enabled {
            let payload = discord::PayloadData {
                state: format!("{} - {}", &track.artist_name, &track.album_name),
                details: track.name.clone(),
                small_image: String::from("playing"),
                small_text: String::from("Playing"),
                album_cover,
                show_timestamps: true,
                duration,
                progress,
            };

            let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
            discord.make_activity(&payload)?;
        }

        // Update Last.FM now playing to current track
        if last_fm_enabled {
            try_update_now_playing_to_lastfm(handle.clone(), track).await?;
        }

        Ok(())
    }
}
