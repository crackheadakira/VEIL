use std::sync::Mutex;

use tauri::State;

use crate::{models::*, SodapopState};

#[tauri::command]
#[specta::specta]
pub fn get_album_with_tracks(id: u32, state: State<'_, Mutex<SodapopState>>) -> AlbumWithTracks {
    let state_guard = state.lock().unwrap();
    state_guard.db.album_with_tracks(&id)
}

#[tauri::command]
#[specta::specta]
pub fn track_by_id(id: u32, state: State<'_, Mutex<SodapopState>>) -> Tracks {
    let state_guard = state.lock().unwrap();
    state_guard.db.get_track_by_id(&id)
}

#[tauri::command]
#[specta::specta]
pub fn get_all_albums(state: State<'_, Mutex<SodapopState>>) -> Vec<Albums> {
    let state_guard = state.lock().unwrap();
    state_guard.db.all_albums()
}

#[tauri::command]
#[specta::specta]
pub fn get_artist_with_albums(id: u32, state: State<'_, Mutex<SodapopState>>) -> ArtistWithAlbums {
    let state_guard = state.lock().unwrap();
    state_guard.db.artist_with_albums(&id)
}
