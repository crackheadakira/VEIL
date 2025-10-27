use crate::{SodapopState, TauriState, error::FrontendError};
use common::Tracks;
use lastfm::{LastFMError, TrackData};
use logging::lock_or_log;
use media_controls::PlayerState;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, ipc::Channel};

#[tauri::command]
#[specta::specta]
pub fn get_player_state(state: TauriState) -> PlayerState {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.state
}

#[tauri::command]
#[specta::specta]
pub fn get_player_progress(state: TauriState) -> f64 {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.progress
}

#[tauri::command]
#[specta::specta]
pub fn player_has_ended(state: TauriState) -> bool {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.has_ended()
}

#[tauri::command]
#[specta::specta]
pub fn player_has_track(state: TauriState) -> bool {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.track.is_some()
}

#[tauri::command]
#[specta::specta]
pub fn get_player_duration(state: TauriState) -> f32 {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.duration
}

#[tauri::command]
#[specta::specta]
pub fn set_player_progress(progress: f64, state: TauriState) {
    let mut player = lock_or_log(state.player.write(), "Player Write Lock").unwrap();
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
            let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();

            if last_track_end_check.elapsed() >= track_end_interval {
                // Track is about to end within 300ms
                if let Some(player_state) = player.get_player_state() {
                    if player_state == media_controls::PlaybackState::Stopped
                        && on_event.send(PlayerProgressEvent::TrackEnd).is_err()
                    {
                        logging::error!("Track-end channel closed");
                        break;
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

            if !player.has_next_sound_handle()
                && (player.duration as f64 - player.get_progress()) <= 0.3
            {
                drop(player);
                let mut player = lock_or_log(state.player.write(), "Player Write Lock").unwrap();
                let mut queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();

                if let Some(next) = queue.peek_next() {
                    match state.db.by_id::<Tracks>(&next) {
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
