use crate::{discord, error::FrontendError, player::PlayerState, SodapopState};
use db::models::Tracks;
use std::sync::Mutex;
use tauri::State;

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
            .update_duration(&track_id, &track.album_id, &(duration as u32))?;
    }

    let _ = state_guard.player.play(&track);

    let progress = state_guard.player.progress;

    let payload = discord::PayloadData {
        state: track.artist_name + " - " + &track.album_name,
        details: track.name,
        small_image: String::from("playing"),
        small_text: String::from("Playing"),
        show_timestamps: true,
        duration,
        progress,
    };

    state_guard.discord.make_activity(payload)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn pause_track(state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.pause();

    let mut payload = state_guard.discord.payload.clone();
    payload = discord::PayloadData {
        small_image: String::from("paused"),
        small_text: String::from("Paused"),
        show_timestamps: false,
        ..payload
    };

    state_guard.discord.make_activity(payload).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn resume_track(state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.resume();

    let mut payload = state_guard.discord.payload.clone();
    payload = discord::PayloadData {
        small_image: String::from("playing"),
        small_text: String::from("Playing"),
        show_timestamps: true,
        ..payload
    };

    state_guard.discord.make_activity(payload).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn seek_track(position: f64, resume: bool, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.seek(position, resume);

    let text_display = if resume {
        String::from("Playing")
    } else {
        String::from("Paused")
    };

    let mut payload = state_guard.discord.payload.clone();
    payload = discord::PayloadData {
        small_image: text_display.to_lowercase(),
        small_text: text_display,
        show_timestamps: true,
        ..payload
    };

    state_guard.discord.make_activity(payload).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn set_volume(volume: f32, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();

    state_guard.player.set_volume(volume);
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
    state_guard.player.initialize_player(track, progress)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_player_progress(progress: f64, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    state_guard.player.set_progress(progress);
}
