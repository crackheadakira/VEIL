use crate::{SodapopState, TauriState, discord, error::FrontendError};
use common::Tracks;
use lastfm::{LastFMError, TrackData};
use media_controls::PlayerState;
use tauri::{AppHandle, Manager};

#[tauri::command]
#[specta::specta]
pub async fn play_track(handle: AppHandle, track_id: u32) -> Result<(), FrontendError> {
    let state = handle.state::<SodapopState>();
    let mut player = state.player.lock().unwrap();

    // if player has track that's been playing, scrobble condition will pass
    if player.track.is_some() && player.scrobble() && !player.scrobbled {
        let player_track_id = player.track.unwrap();
        let track = state.db.by_id::<Tracks>(&player_track_id)?;
        let track_timestamp = player.timestamp;
        let handle = handle.clone();
        player.scrobbled = true;
        scrobble_helper(handle, track, track_timestamp);
    }

    let track = state.db.by_id::<Tracks>(&track_id)?;
    // temporary scope to drop player before await
    {
        if player.track.is_some() {
            player.stop()?;
        };

        let duration = player.duration;

        if track.duration == 0 {
            state
                .db
                .update_duration(&track_id, &track.album_id, &(duration as u32))?;
        }

        let _ = player.play(&track);

        let progress = player.progress;
        let payload = discord::PayloadData {
            state: track.artist_name.clone() + " - " + &track.album_name,
            details: track.name.clone(),
            small_image: String::from("playing"),
            small_text: String::from("Playing"),
            show_timestamps: true,
            duration,
            progress,
        };

        let mut discord = state.discord.lock().unwrap();
        discord.make_activity(&payload)?;
    }

    let handle = handle.clone();
    tokio::spawn(async move {
        let state = handle.state::<SodapopState>();
        let lastfm = state.lastfm.lock().await;
        let res = lastfm
            .track()
            .update_now_playing(TrackData {
                artist: track.artist_name,
                name: track.name,
                album: Some(track.album_name),
                timestamp: None,
            })
            .send()
            .await;

        if let Err(LastFMError::RequestWhenDisabled) = res {
        } else if let Err(e) = res {
            eprintln!("LastFM error from player: {e}");
        }
    });

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn pause_track(handle: AppHandle) -> Result<(), FrontendError> {
    let state = handle.state::<SodapopState>();
    let mut player = state.player.lock().unwrap();
    let mut discord = state.discord.lock().unwrap();
    player.pause()?;

    // if player has track that's been playing, scrobble condition will pass
    if player.track.is_some() && player.scrobble() && !player.scrobbled {
        let player_track_id = player.track.unwrap();
        let track = state.db.by_id::<Tracks>(&player_track_id)?;
        let track_timestamp = player.timestamp;
        let handle = handle.clone();
        player.scrobbled = true;
        scrobble_helper(handle, track, track_timestamp);
    }

    discord.update_activity("paused", "Paused", false);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn resume_track(state: TauriState) -> Result<(), FrontendError> {
    let mut player = state.player.lock().unwrap();
    let mut discord = state.discord.lock().unwrap();

    player.resume()?;
    discord.update_activity("playing", "Playing", true);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn seek_track(position: f64, resume: bool, state: TauriState) -> Result<(), FrontendError> {
    let mut player = state.player.lock().unwrap();
    let mut discord = state.discord.lock().unwrap();
    player.seek(position, resume)?;

    let text_display = if resume { "Playing" } else { "Paused" };

    discord.update_activity(&text_display.to_lowercase(), text_display, true);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_volume(volume: f32, state: TauriState) -> Result<(), FrontendError> {
    let mut player = state.player.lock().unwrap();

    player.set_volume(volume)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_player_state(state: TauriState) -> PlayerState {
    let player = state.player.lock().unwrap();
    player.state
}

#[tauri::command]
#[specta::specta]
pub fn get_player_progress(state: TauriState) -> f64 {
    let player = state.player.lock().unwrap();
    player.progress
}

#[tauri::command]
#[specta::specta]
pub fn player_has_ended(state: TauriState) -> bool {
    let player = state.player.lock().unwrap();
    player.has_ended()
}

#[tauri::command]
#[specta::specta]
pub fn player_has_track(state: TauriState) -> bool {
    let player = state.player.lock().unwrap();
    player.track.is_some()
}

#[tauri::command]
#[specta::specta]
pub fn get_player_duration(state: TauriState) -> f32 {
    let player = state.player.lock().unwrap();
    player.duration
}

#[tauri::command]
#[specta::specta]
pub fn stop_player(state: TauriState) -> Result<(), FrontendError> {
    let mut player = state.player.lock().unwrap();
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
    let mut player = state.player.lock().unwrap();
    let track = state.db.by_id::<Tracks>(&track_id)?;
    player.initialize_player(track, progress)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_player_progress(progress: f64, state: TauriState) {
    let mut player = state.player.lock().unwrap();
    player.set_progress(progress);
}

fn scrobble_helper(handle: AppHandle, track: Tracks, track_timestamp: i64) {
    tokio::spawn(async move {
        let state = handle.state::<SodapopState>();
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

        if let Err(LastFMError::RequestWhenDisabled) = res {
        } else if let Err(e) = res {
            eprintln!("LastFM error from player: {e}");
        }
    });
}
