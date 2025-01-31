use std::sync::Mutex;

use tauri::State;

use crate::{error::FrontendError, models::*, SodapopState};

#[tauri::command]
#[specta::specta]
pub fn get_album_with_tracks(
    id: u32,
    state: State<'_, Mutex<SodapopState>>,
) -> Result<AlbumWithTracks, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.album_with_tracks(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn track_by_id(
    id: u32,
    state: State<'_, Mutex<SodapopState>>,
) -> Result<Tracks, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.by_id::<Tracks>(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_all_albums(state: State<'_, Mutex<SodapopState>>) -> Result<Vec<Albums>, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.all::<Albums>()?)
}

#[tauri::command]
#[specta::specta]
pub fn get_artist_with_albums(
    id: u32,
    state: State<'_, Mutex<SodapopState>>,
) -> Result<ArtistWithAlbums, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.artist_with_albums(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn new_playlist(
    name: String,
    state: State<'_, Mutex<SodapopState>>,
) -> Result<(), FrontendError> {
    let state_guard = state.lock().unwrap();
    state_guard.db.insert::<Playlists>(Playlists {
        id: 0,
        name,
        description: String::from(""),
        cover_path: String::from("../../../public/placeholder.png"),
    })?;

    Ok(())
}
