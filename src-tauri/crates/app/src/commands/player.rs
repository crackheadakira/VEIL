use crate::{SodapopState, TauriState, discord, error::FrontendError};
use common::Tracks;
use lastfm::{LastFMError, TrackData};
use logging::lock_or_log;
use media_controls::PlayerState;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, ipc::Channel};

#[tauri::command]
#[specta::specta]
pub async fn play_track(handle: AppHandle, track_id: u32) -> Result<(), FrontendError> {
    let state = handle.state::<SodapopState>();
    let track = state.db.by_id::<Tracks>(&track_id)?;

    let (last_fm_enabled, discord_enabled) = {
        let config = lock_or_log(state.config.read(), "Config Read")?;
        (config.last_fm_enabled, config.discord_enabled)
    };

    let should_scrobble = {
        let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;
        player.should_scrobble()
    };

    // if player has track that's been playing, scrobble condition will pass
    if let Some((track_id, track_timestamp)) = should_scrobble
        && last_fm_enabled
    {
        try_scrobble_track_to_lastfm(handle.clone(), track_id, track_timestamp).await?;
    }

    // Temporary scope to avoid MutexGuard + async issues
    let (progress, duration) = {
        let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;

        if player.track.is_some() {
            player.stop()?;
        };

        let duration = if track.duration == 0 {
            state
                .db
                .update_duration(&track_id, &track.album_id, &(player.duration as u32))?;

            player.duration
        } else {
            track.duration as f32
        };

        player.play(&track)?;

        (player.progress, duration)
    };

    // Get album_cover to be used in Discord RPC if both Discord && LastFM are enabled.
    let handle = handle.clone();
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

    if last_fm_enabled {
        try_update_now_playing_to_lastfm(handle.clone(), track).await?;
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn pause_track(handle: AppHandle) -> Result<(), FrontendError> {
    let state = handle.state::<SodapopState>();

    {
        let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
        discord.update_activity("paused", "Paused", false, None);
    };

    let should_scrobble = {
        let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;
        player.pause()?;

        player.should_scrobble()
    };

    // if player has track that's been playing, scrobble condition will pass
    if let Some((track_id, track_timestamp)) = should_scrobble {
        try_scrobble_track_to_lastfm(handle.clone(), track_id, track_timestamp).await?;
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn resume_track(state: TauriState) -> Result<(), FrontendError> {
    let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;

    let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;

    player.resume()?;
    discord.update_activity("playing", "Playing", true, Some(player.get_progress()));
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn seek_track(position: f64, resume: bool, state: TauriState) -> Result<(), FrontendError> {
    let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;
    let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
    player.seek(position, resume)?;

    let text_display = if resume { "Playing" } else { "Paused" };

    discord.update_activity(
        &text_display.to_lowercase(),
        text_display,
        resume,
        Some(position),
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_volume(volume: f32, state: TauriState) -> Result<(), FrontendError> {
    let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;

    player.set_volume(volume)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_player_state(state: TauriState) -> PlayerState {
    let player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();
    player.state
}

#[tauri::command]
#[specta::specta]
pub fn get_player_progress(state: TauriState) -> f64 {
    let player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();
    player.progress
}

#[tauri::command]
#[specta::specta]
pub fn player_has_ended(state: TauriState) -> bool {
    let player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();
    player.has_ended()
}

#[tauri::command]
#[specta::specta]
pub fn player_has_track(state: TauriState) -> bool {
    let player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();
    player.track.is_some()
}

#[tauri::command]
#[specta::specta]
pub fn get_player_duration(state: TauriState) -> f32 {
    let player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();
    player.duration
}

#[tauri::command]
#[specta::specta]
pub fn stop_player(state: TauriState) -> Result<(), FrontendError> {
    let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;
    player.stop()?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn initialize_player(
    track_id: u32,
    progress: f64,
    state: TauriState,
) -> Result<(), FrontendError> {
    let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;
    let track = state.db.by_id::<Tracks>(&track_id)?;
    player.initialize_player(track, progress)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_player_progress(progress: f64, state: TauriState) {
    let mut player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();
    player.set_progress(progress);
}

#[derive(Clone, Serialize, Type)]
#[serde(tag = "event", content = "data")]
// rust-analyzer expected Expr error: https://github.com/specta-rs/specta/issues/387
pub enum PlayerProgressEvent {
    Progress { progress: f64 },
    TrackEnd,
}

#[tauri::command]
#[specta::specta]
pub fn player_progress_channel(
    handle: AppHandle,
    on_event: Channel<PlayerProgressEvent>,
) -> Result<(), FrontendError> {
    std::thread::spawn(move || {
        let state = handle.state::<SodapopState>();

        let track_end_interval = std::time::Duration::from_millis(25);
        let mut last_track_end_check = std::time::Instant::now();

        let progress_interval = std::time::Duration::from_millis(400);
        let mut last_progress_sent = std::time::Instant::now();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();

            if last_track_end_check.elapsed() >= track_end_interval {
                if let Some(player_state) = player.get_player_state() {
                    if player_state == media_controls::PlaybackState::Stopped
                    // || player_state == media_controls::PlaybackState::Stopping
                    {
                        if on_event.send(PlayerProgressEvent::TrackEnd).is_err() {
                            logging::error!("Track-end channel closed");
                            break;
                        }
                    }
                }
                last_track_end_check = std::time::Instant::now();
            }

            if last_progress_sent.elapsed() >= progress_interval {
                if let media_controls::PlayerState::Playing = player.state {
                    let progress = player.get_progress();
                    if on_event
                        .send(PlayerProgressEvent::Progress { progress })
                        .is_err()
                    {
                        logging::error!("Progress channel closed");
                        break;
                    }
                }
                last_progress_sent = std::time::Instant::now();
            }
        }
    });

    Ok(())
}

/// Try to scrobble the track to LastFM.
///
/// Spawns an async task, and upon an error logs it.
pub async fn try_scrobble_track_to_lastfm(
    handle: AppHandle,
    track_id: u32,
    track_timestamp: i64,
) -> Result<(), FrontendError> {
    let state = handle.state::<SodapopState>();
    let track = state.db.by_id::<Tracks>(&track_id)?;

    let lastfm = state.lastfm.lock().await;
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
        Err(LastFMError::RequestWhenDisabled) => {}
        Err(e) => return Err(e.into()),
        Ok(_) => {}
    }

    Ok(())
}

/// Tries to set now playing to given track on LastFM.
///
/// Spawns an async task, and upon an error logs it.
pub async fn try_update_now_playing_to_lastfm(
    handle: AppHandle,
    track: Tracks,
) -> Result<(), FrontendError> {
    let state = handle.state::<SodapopState>();

    let lastfm = state.lastfm.lock().await;
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
        Err(LastFMError::RequestWhenDisabled) => {}
        Err(e) => return Err(e.into()),
        Ok(_) => {}
    }

    Ok(())
}
