use crate::{error::FrontendError, StateMutex};
use db::models::*;

#[tauri::command]
#[specta::specta]
pub fn get_album_with_tracks(id: u32, state: StateMutex) -> Result<AlbumWithTracks, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.album_with_tracks(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn track_by_id(id: u32, state: StateMutex) -> Result<Tracks, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.by_id::<Tracks>(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_all_albums(state: StateMutex) -> Result<Vec<Albums>, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.all::<Albums>()?)
}

#[tauri::command]
#[specta::specta]
pub fn get_artist_with_albums(
    id: u32,
    state: StateMutex,
) -> Result<ArtistWithAlbums, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.artist_with_albums(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn search_db(search_str: &str, state: StateMutex) -> Result<Vec<Search>, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.search(search_str)?)
}

#[tauri::command]
#[specta::specta]
pub fn new_playlist(name: String, state: StateMutex) -> Result<(), FrontendError> {
    let state_guard = state.lock().unwrap();
    state_guard.db.insert::<Playlists>(Playlists {
        id: 0,
        name,
        description: String::from(""),
        cover_path: String::from("/placeholder.png"),
    })?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_all_playlists(state: StateMutex) -> Result<Vec<Playlists>, FrontendError> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.db.all::<Playlists>()?)
}

#[tauri::command]
#[specta::specta]
pub fn add_to_playlist(
    playlist_id: u32,
    track_id: u32,
    state: StateMutex,
) -> Result<(), FrontendError> {
    let state_guard = state.lock().unwrap();
    state_guard
        .db
        .insert_track_to_playlist(&playlist_id, &track_id)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_playlist_tracks(
    playlist_id: u32,
    state: StateMutex,
) -> Result<PlaylistWithTracks, FrontendError> {
    let state_guard = state.lock().unwrap();
    let playlist = state_guard.db.get_playlist_with_tracks(&playlist_id)?;

    Ok(playlist)
}

#[tauri::command]
#[specta::specta]
pub fn remove_from_playlist(
    playlist_id: u32,
    track_id: u32,
    state: StateMutex,
) -> Result<(), FrontendError> {
    let state_guard = state.lock().unwrap();
    state_guard
        .db
        .delete_track_from_playlist(&playlist_id, &track_id)?;

    Ok(())
}
