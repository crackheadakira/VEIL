use common::Tracks;
use logging::lock_or_log;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
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
#[serde(tag = "type", content = "data")]
pub enum PlayerEvent {
    NewTrack { track: Tracks },
    Pause,
    Resume,
    Stop,
}

struct OnlineFeatures {
    discord_enabled: bool,
    last_fm_enabled: bool,
}

impl PlayerEvent {
    pub async fn handle(
        event: TypedEvent<PlayerEvent>,
        handle: &AppHandle,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();

        let online = {
            let config = lock_or_log(state.config.read(), "Config Read")?;
            OnlineFeatures {
                last_fm_enabled: config.last_fm_enabled,
                discord_enabled: config.discord_enabled,
            }
        };

        match event.payload {
            PlayerEvent::NewTrack { track } => Self::set_new_track(track, &handle, online).await?,
            PlayerEvent::Pause => Self::pause_current_track(&handle, online).await?,
            PlayerEvent::Resume => todo!(),
            PlayerEvent::Stop => todo!(),
        };

        Ok(())
    }

    /// Resets the player states to as if no track had been playing, then loads in the new track.
    ///
    /// Also handles Discord RPC & Last.FM scrobbling.
    async fn set_new_track(
        track: Tracks,
        handle: &AppHandle,
        online: OnlineFeatures,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();

        let should_scrobble = {
            let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;
            player.should_scrobble()
        };

        // Scrobble the previous track to Last.FM
        if let Some((track_id, track_timestamp)) = should_scrobble
            && online.last_fm_enabled
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

        if online.discord_enabled {
            // Get album cover URL from Last.FM if Discord & Last.FM are enabled.
            let album_cover = if online.last_fm_enabled {
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
        if online.last_fm_enabled {
            try_update_now_playing_to_lastfm(handle.clone(), track).await?;
        }

        Ok(())
    }

    /// Pauses the currently playing track.
    ///
    /// Update Discord activity to also say paused, and will check if the current playing track
    /// should be scrobbled on Last.FM
    async fn pause_current_track(
        handle: &AppHandle,
        online: OnlineFeatures,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();

        if online.discord_enabled {
            let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
            discord.update_activity("paused", "Paused", false, None);
        };

        let should_scrobble = {
            let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;
            player.pause()?;

            player.should_scrobble()
        };

        if online.last_fm_enabled {
            if let Some((track_id, track_timestamp)) = should_scrobble {
                try_scrobble_track_to_lastfm(handle.clone(), track_id, track_timestamp).await?;
            }
        }

        Ok(())
    }
}
