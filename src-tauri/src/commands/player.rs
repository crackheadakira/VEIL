use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{player::PlayerState, SodapopState};

#[tauri::command]
#[specta::specta]
pub fn play_track(track_id: u32, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();

    if state_guard.player.track.is_some() {
        state_guard.player.stop();
    };

    let _ = state_guard.player.play(track_id);
}

#[tauri::command]
#[specta::specta]
pub fn pause_track(state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.pause()
}

#[tauri::command]
#[specta::specta]
pub fn resume_track(state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.resume()
}

#[tauri::command]
#[specta::specta]
pub fn seek_track(position: f64, resume: bool, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard = state.lock().unwrap();
    state_guard.player.seek(position, resume)
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
pub fn update_progress(app: AppHandle) {
    let state = app.state::<Mutex<SodapopState>>();
    let mut state_guard = state.lock().unwrap();

    match state_guard.player.state {
        PlayerState::Playing => {
            state_guard.player.update();
            app.emit("player-progress", state_guard.player.progress)
                .unwrap();
        }
        _ => (),
    };

    if state_guard.player.progress >= (state_guard.player.duration - 0.2) as f64 {
        app.emit("track-end", 0.0).unwrap();
    };
}

#[tauri::command]
#[specta::specta]
pub fn initialize_player(track_id: u32, progress: f64, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    let _ = state_guard.player.initialize_player(track_id, progress);
}

#[tauri::command]
#[specta::specta]
pub fn set_player_progress(progress: f64, state: State<'_, Mutex<SodapopState>>) {
    let mut state_guard: std::sync::MutexGuard<'_, SodapopState> = state.lock().unwrap();
    state_guard.player.set_progress(progress);
}
