use std::sync::RwLockWriteGuard;

use common::Tracks;
use lastfm::TrackData;
use logging::lock_or_log;
use media_controls::{DefaultPlayer, PlaybackState};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, ipc::Channel};
use tauri_specta::{Event, TypedEvent};
use tokio::sync::MutexGuard;

use crate::{
    SodapopState, TauriState,
    commands::player::PlayerProgressEvent,
    discord::PayloadData,
    error::FrontendError,
    events::EventSystemHandler,
    systems::ui::{PlayButtonState, UIUpdateEvent},
};

/// Try to scrobble the track to `LastFM`.
///
/// Spawns an async task, and upon an error logs it.
pub async fn try_scrobble_track_to_lastfm(
    lastfm: MutexGuard<'_, lastfm::LastFM>,
    track: Tracks,
    track_timestamp: i64,
) -> Result<(), FrontendError> {
    let res = lastfm
        .track()
        .scrobble_one(&TrackData {
            artist: track.artist_name,
            name: track.name,
            album: Some(track.album_name),
            timestamp: Some(track_timestamp),
        })
        .send()
        .await;

    match res {
        Err(lastfm::Error::RequestWhenDisabled) | Ok(_) => {}
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

/// Tries to set now playing to given track on `LastFM`.
///
/// Spawns an async task, and upon an error logs it.
pub async fn try_update_now_playing_to_lastfm(
    lastfm: MutexGuard<'_, lastfm::LastFM>,
    track: Tracks,
) -> Result<(), FrontendError> {
    let res = lastfm
        .track()
        .update_now_playing(&TrackData {
            artist: track.artist_name,
            name: track.name,
            album: Some(track.album_name),
            timestamp: None,
        })
        .send()
        .await;

    match res {
        Err(lastfm::Error::RequestWhenDisabled) | Ok(_) => {}
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

pub fn next_track_status(
    state: &TauriState,
    player: &RwLockWriteGuard<'_, DefaultPlayer>,
) -> Option<Tracks> {
    player
        .get_player_state()
        .filter(|&s| s == media_controls::PlaybackState::Stopped)
        .and_then(|_| {
            let mut queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
            queue.next()
        })
        .and_then(|id| match state.db.by_id::<Tracks>(&id) {
            Ok(track) => Some(track),
            Err(e) => {
                logging::error!("Error fetching track from database in queue: {e}");
                None
            }
        })
}

pub fn send_player_progress_via_channel(
    state: &TauriState,
    on_event: &Channel<PlayerProgressEvent>,
) {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();

    if let media_controls::PlayerState::Playing = player.state {
        let progress = player.get_progress();
        if on_event
            .send(PlayerProgressEvent::Progress { progress })
            .is_err()
        {
            logging::error!("Progress channel closed");
        }
    }
}

pub fn try_preloading_next_sound_handle(
    state: &TauriState,
    player: &mut RwLockWriteGuard<'_, DefaultPlayer>,
) {
    // We already check if the track ends soon outside the call
    if !player.has_preloaded_track() {
        let next_track_id = {
            let mut queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
            queue.peek_next()
        };

        if let Some(next_track_id) = next_track_id {
            match state.db.by_id::<Tracks>(&next_track_id) {
                Ok(track) => {
                    if let Err(e) = player.maybe_queue_next(&track) {
                        logging::error!("Failed to preload next track: {e}");
                    }
                }
                Err(e) => {
                    logging::error!(
                        "Tried preloading next track but got error fetching from database: {e}"
                    );
                }
            };
        }
    }
}

#[derive(Serialize, Deserialize, Type, Event, Clone)]
#[serde(tag = "type", content = "data")]
pub enum PlayerEvent {
    /// Initialize the player to load in this track, seeked to the specified position.
    Initialize { track: Tracks, progress: f64 },

    /// If a new track is to be played.
    NewTrack { track: Tracks },

    /// Plays the track at the current index in the queue.
    CurrentTrackInQueue,

    /// Play the previous track in the queue.
    PreviousTrackInQueue,

    /// Play the next track in the queue.
    NextTrackInQueue,

    /// If the current track is to be paused.
    Pause,

    /// If the current track is to be resumed.
    Resume,

    /// Depdenent on the state of the player either pause or resumme the track.
    UpdatePlayerState,

    /// If the current track is to be stopped.
    Stop,

    /// Where to set the progress of the currently playing track.
    Seek { position: f64, resume: bool },

    /// Set the volume of the player.
    SetVolume { volume: f32 },
}

struct OnlineFeatures {
    /// Whether or not Discord is enabled, and any corresponding features should be used.
    discord_enabled: bool,

    /// Whether or not Last.FM is enabled, and any corresponding features should be used.
    last_fm_enabled: bool,
}

impl EventSystemHandler for PlayerEvent {
    async fn handle(
        event: TypedEvent<PlayerEvent>,
        handle: &AppHandle,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();

        let online = {
            let config: std::sync::RwLockReadGuard<'_, crate::config::SodapopConfig> =
                lock_or_log(state.config.read(), "Config Read")?;
            OnlineFeatures {
                last_fm_enabled: config.integrations.last_fm_enabled,
                discord_enabled: config.integrations.discord_enabled,
            }
        };

        // TODO: Implement frontend emits.
        // These emits would then sync the frontend to the backend, with the backend being the source of truth.
        match event.payload {
            PlayerEvent::Initialize { track, progress } => {
                Self::initialize_player_with_track(handle, track, progress)?;
            }
            PlayerEvent::NewTrack { track } => Self::set_new_track(handle, track, online).await?,
            PlayerEvent::Pause => Self::pause_current_track(handle, online).await?,
            PlayerEvent::Resume => Self::resume_current_track(handle, online)?,
            PlayerEvent::UpdatePlayerState => {
                let playback_state = {
                    let player = lock_or_log(state.player.read(), "Player Read Lock")?;
                    player.get_player_state()
                };

                if let Some(playback_state) = playback_state {
                    if playback_state == PlaybackState::Playing {
                        Self::pause_current_track(handle, online).await?;
                    } else if playback_state == PlaybackState::Paused {
                        Self::resume_current_track(handle, online)?;
                    }
                } else {
                    // We have to load in the track as there is no playback state yet
                    logging::debug!(
                        "Player does not have a track, but attempted to update it's state"
                    );
                }
            }
            PlayerEvent::Stop => Self::stop_current_track(handle)?,
            PlayerEvent::Seek { position, resume } => {
                Self::seek_current_track(handle, position, resume, online)?;
            }
            PlayerEvent::SetVolume { volume } => Self::set_player_volume(handle, volume)?,
            PlayerEvent::PreviousTrackInQueue => {
                Self::play_previous_track_from_queue(handle, online).await?;
            }
            PlayerEvent::NextTrackInQueue => {
                Self::play_next_track_from_queue(handle, online).await?;
            }
            PlayerEvent::CurrentTrackInQueue => {
                Self::play_current_track_from_queue(handle, online).await?;
            }
        };

        Ok(())
    }
}

impl PlayerEvent {
    /// Loads the track into the player at the specified position.
    fn initialize_player_with_track(
        handle: &AppHandle,
        track: Tracks,
        progress: f64,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();
        let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;
        player.initialize_player(track, progress)?;

        Ok(())
    }

    /// Resets the player states to as if no track had been playing, then loads in the new track.
    ///
    /// Also handles Discord RPC & Last.FM scrobbling.
    async fn set_new_track(
        handle: &AppHandle,
        track: Tracks,
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
            let track = state.db.by_id::<Tracks>(&track_id)?;
            let lastfm = state.lastfm.lock().await;
            try_scrobble_track_to_lastfm(lastfm, track, track_timestamp).await?;
        }

        let (progress, duration) = {
            let mut player = lock_or_log(state.player.write(), "Player Write Lock").unwrap();

            // Reset the player state to as if no track had been playing.
            if player.track.is_some() {
                player.stop()?;
            };

            player.play(&track, None)?
        };

        state.resume_notify.notify_waiters();

        UIUpdateEvent::emit(
            &UIUpdateEvent::PlayButton {
                state: PlayButtonState::Playing,
            },
            handle,
        )?;

        UIUpdateEvent::emit(
            &UIUpdateEvent::TrackChange {
                track: track.clone(),
            },
            handle,
        )?;

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

            let payload = PayloadData {
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
            let lastfm = state.lastfm.lock().await;
            try_update_now_playing_to_lastfm(lastfm, track).await?;
        }

        Ok(())
    }

    /// Plays the current track in queue by passing it to [`PlayerEvent::set_new_track`]
    ///
    async fn play_current_track_from_queue(
        handle: &AppHandle,
        online: OnlineFeatures,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();
        let track_id = {
            let queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
            queue.current()
        };

        if let Some(track_id) = track_id {
            let track = state.db.by_id::<Tracks>(&track_id)?;

            Self::set_new_track(handle, track, online).await?;
        }

        Ok(())
    }

    /// Plays the next track in queue by passing it to [`PlayerEvent::set_new_track`]
    ///
    async fn play_next_track_from_queue(
        handle: &AppHandle,
        online: OnlineFeatures,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();
        let track_id = {
            let mut queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
            queue.next()
        };

        if let Some(track_id) = track_id {
            let track = state.db.by_id::<Tracks>(&track_id)?;

            Self::set_new_track(handle, track, online).await?;
        }

        Ok(())
    }

    /// Plays the previous track in queue by passing it to [`PlayerEvent::set_new_track`]
    ///
    async fn play_previous_track_from_queue(
        handle: &AppHandle,
        online: OnlineFeatures,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();
        let track_id = {
            let mut queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
            queue.previous()
        };

        if let Some(track_id) = track_id {
            let track = state.db.by_id::<Tracks>(&track_id)?;

            Self::set_new_track(handle, track, online).await?;
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

        UIUpdateEvent::emit(
            &UIUpdateEvent::PlayButton {
                state: PlayButtonState::Paused,
            },
            handle,
        )?;

        if online.last_fm_enabled
            && let Some((track_id, track_timestamp)) = should_scrobble
        {
            let track = state.db.by_id::<Tracks>(&track_id)?;
            let lastfm = state.lastfm.lock().await;
            try_scrobble_track_to_lastfm(lastfm, track, track_timestamp).await?;
        }

        Ok(())
    }

    /// Resumes the current track.
    ///
    /// Updates Discord activity.
    fn resume_current_track(
        handle: &AppHandle,
        online: OnlineFeatures,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();

        let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;
        player.resume()?;

        UIUpdateEvent::emit(
            &UIUpdateEvent::PlayButton {
                state: PlayButtonState::Playing,
            },
            handle,
        )?;

        if online.discord_enabled {
            let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
            discord.update_activity("playing", "Playing", true, Some(player.get_progress()));
        }

        Ok(())
    }

    /// Stops the current track.
    fn stop_current_track(handle: &AppHandle) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();
        let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;

        player.stop()?;

        UIUpdateEvent::emit(
            &UIUpdateEvent::PlayButton {
                state: PlayButtonState::Paused,
            },
            handle,
        )?;

        Ok(())
    }

    /// Seek the current track to the given position.
    ///
    /// Updates Discord activity.
    fn seek_current_track(
        handle: &AppHandle,
        position: f64,
        resume: bool,
        online: OnlineFeatures,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();

        let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;
        player.seek(position, resume)?;

        if online.discord_enabled {
            let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
            let text_display = if resume { "Playing" } else { "Paused" };
            discord.update_activity(
                &text_display.to_lowercase(),
                text_display,
                resume,
                Some(position),
            );
        }

        Ok(())
    }

    fn set_player_volume(handle: &AppHandle, volume: f32) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();
        let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;

        player.set_volume(volume)?;
        Ok(())
    }
}
