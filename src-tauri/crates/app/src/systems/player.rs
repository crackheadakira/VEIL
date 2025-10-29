use std::sync::RwLockWriteGuard;

use common::Tracks;
use lastfm::TrackData;
use logging::lock_or_log;
use media_controls::Player;
use tauri::ipc::Channel;
use tokio::sync::MutexGuard;

use crate::{TauriState, commands::player::PlayerProgressEvent, error::FrontendError};

/// Try to scrobble the track to LastFM.
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
        Err(lastfm::LastFMError::RequestWhenDisabled) => {}
        Err(e) => return Err(e.into()),
        Ok(_) => {}
    }

    Ok(())
}

/// Tries to set now playing to given track on LastFM.
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
        Err(lastfm::LastFMError::RequestWhenDisabled) => {}
        Err(e) => return Err(e.into()),
        Ok(_) => {}
    }

    Ok(())
}

pub fn next_track_status(
    state: &TauriState,
    player: &RwLockWriteGuard<'_, Player>,
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
    player: &mut RwLockWriteGuard<'_, Player>,
) {
    // We already check if the track ends soon outside the call
    if !player.has_next_sound_handle() {
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
