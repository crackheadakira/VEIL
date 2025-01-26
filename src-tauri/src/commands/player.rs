use souvlaki::{MediaMetadata, MediaPlayback};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{error::FrontendError, models::Tracks, player::PlayerState, SodapopState};

#[tauri::command]
#[specta::specta]
pub fn play_track(
    track_id: u32,
    state: State<'_, Mutex<SodapopState>>,
) -> Result<(), FrontendError> {
    let mut state_guard = state.lock().unwrap();

    if state_guard.player.track.is_some() {
        state_guard.player.stop();
    };

    let track = state_guard.db.by_id::<Tracks>(&track_id)?;
    let duration = state_guard.player.duration;

    if track.duration == 0 {
        state_guard
            .db
            .update_duration(&track_id, &track.albums_id, &(duration as u32))?;
    }

    let _ = state_guard.player.play(track.clone());

    state_guard.controls.set_metadata(MediaMetadata {
        title: Some(&track.name),
        album: Some(&track.album),
        artist: Some(&track.artist),
        cover_url: Some(&track.cover_path),
        duration: Some(std::time::Duration::from_secs(duration as u64)),
    })?;

    let progress = state_guard.player.progress;
    state_guard.controls.set_playback(MediaPlayback::Playing {
        progress: progress_as_position(progress),
    })?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn pause_track(state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.pause();

    let progress = state_guard.player.progress;
    state_guard
        .controls
        .set_playback(MediaPlayback::Paused {
            progress: progress_as_position(progress),
        })
        .unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn resume_track(state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.resume();

    let progress = state_guard.player.progress;
    state_guard
        .controls
        .set_playback(MediaPlayback::Playing {
            progress: progress_as_position(progress),
        })
        .unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn seek_track(position: f64, resume: bool, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.seek(position, resume);

    let progress = state_guard.player.progress;
    let playback = if resume {
        MediaPlayback::Playing {
            progress: progress_as_position(progress),
        }
    } else {
        MediaPlayback::Paused {
            progress: progress_as_position(progress),
        }
    };
    state_guard.controls.set_playback(playback).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn set_volume(volume: f32, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();

    state_guard.player.set_volume(volume);

    // Convert volume to a 0.0 - 1.0 scale (from -30 to 1.2)
    let converted_volume = (volume + 30.0) / 31.2;
    state_guard
        .controls
        .set_volume(converted_volume as f64)
        .unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn get_player_state(state: State<'_, Mutex<SodapopState>>) -> PlayerState {
    let state_guard = state.lock().unwrap();
    state_guard.player.state
}

#[tauri::command]
#[specta::specta]
pub fn get_player_progress(state: State<'_, Mutex<SodapopState>>) -> f64 {
    let state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    state_guard.player.progress
}

#[tauri::command]
#[specta::specta]
pub fn player_has_ended(state: State<'_, Mutex<SodapopState>>) -> bool {
    let state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    state_guard.player.has_ended()
}

#[tauri::command]
#[specta::specta]
pub fn player_has_track(state: State<'_, Mutex<SodapopState>>) -> bool {
    let state_guard = state.lock().unwrap();
    state_guard.player.track.is_some()
}

#[tauri::command]
#[specta::specta]
pub fn get_player_duration(state: State<'_, Mutex<SodapopState>>) -> f32 {
    let state_guard = state.lock().unwrap();
    state_guard.player.duration
}

#[tauri::command]
#[specta::specta]
pub fn stop_player(state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.stop();

    state_guard
        .controls
        .set_playback(MediaPlayback::Stopped)
        .unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn update_progress(app: AppHandle) {
    let state = app.state::<Mutex<SodapopState>>();
    let mut state_guard = state.lock().unwrap();
    let progress = state_guard.player.progress;

    if let PlayerState::Playing = state_guard.player.state {
        state_guard.player.update();
        app.emit("player-progress", progress).unwrap();

        state_guard
            .controls
            .set_playback(MediaPlayback::Playing {
                progress: progress_as_position(progress),
            })
            .unwrap();

        if progress >= (state_guard.player.duration - 0.2) as f64 {
            app.emit("track-end", 0.0).unwrap();
        };
    };
}

#[tauri::command]
#[specta::specta]
pub fn initialize_player(
    track_id: u32,
    progress: f64,
    state: State<'_, Mutex<SodapopState>>,
) -> Result<(), FrontendError> {
    let mut state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    let track = state_guard.db.by_id::<Tracks>(&track_id)?;

    state_guard.controls.set_metadata(MediaMetadata {
        title: Some(&track.name),
        album: Some(&track.album),
        artist: Some(&track.artist),
        cover_url: Some(&track.cover_path),
        duration: Some(std::time::Duration::from_secs(track.duration as u64)),
    })?;

    state_guard.player.initialize_player(track, progress)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_player_progress(progress: f64, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    state_guard.player.set_progress(progress);

    state_guard
        .controls
        .set_playback(MediaPlayback::Playing {
            progress: progress_as_position(progress),
        })
        .unwrap();
}

fn progress_as_position(progress: f64) -> Option<souvlaki::MediaPosition> {
    Some(souvlaki::MediaPosition(std::time::Duration::from_secs_f64(
        progress,
    )))
}
