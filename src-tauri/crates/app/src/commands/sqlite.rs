use crate::{TauriState, error::FrontendError};
use common::*;

#[tauri::command]
#[specta::specta]
pub fn get_album_with_tracks(id: u32, state: TauriState) -> Result<AlbumWithTracks, FrontendError> {
    Ok(state.db.album_with_tracks(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn track_by_id(id: u32, state: TauriState) -> Result<Tracks, FrontendError> {
    Ok(state.db.by_id::<Tracks>(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_all_albums(state: TauriState) -> Result<Vec<Albums>, FrontendError> {
    Ok(state.db.all::<Albums>()?)
}

#[tauri::command]
#[specta::specta]
pub fn get_total_albums(state: TauriState) -> Result<u32, FrontendError> {
    Ok(state.db.rows::<Albums>()?)
}

#[tauri::command]
#[specta::specta]
pub fn get_albums_offset(
    state: TauriState,
    limit: u32,
    offset: u32,
) -> Result<Vec<Albums>, FrontendError> {
    Ok(state.db.album_pagination(limit, offset)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_batch_track(
    state: TauriState,
    ids: Vec<u32>,
) -> Result<Vec<Option<Tracks>>, FrontendError> {
    Ok(state.db.batch_id::<Tracks>(&ids)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_artist_with_albums(
    id: u32,
    state: TauriState,
) -> Result<ArtistWithAlbums, FrontendError> {
    Ok(state.db.artist_with_albums(&id)?)
}

#[tauri::command]
#[specta::specta]
pub fn search_db(search_str: &str, state: TauriState) -> Result<Vec<Search>, FrontendError> {
    Ok(state.db.search(search_str)?)
}

#[tauri::command]
#[specta::specta]
pub fn new_playlist(name: String, state: TauriState) -> Result<u32, FrontendError> {
    state.db.insert::<Playlists>(Playlists {
        id: 0,
        name,
        description: String::from(""),
        cover_path: String::from("/placeholder.png"),
    })?;

    let latest = state.db.latest::<Playlists>()?;

    Ok(latest.id)
}

#[tauri::command]
#[specta::specta]
pub fn get_all_playlists(state: TauriState) -> Result<Vec<Playlists>, FrontendError> {
    Ok(state.db.all::<Playlists>()?)
}

#[tauri::command]
#[specta::specta]
pub fn add_to_playlist(
    playlist_id: u32,
    track_id: u32,
    state: TauriState,
) -> Result<(), FrontendError> {
    state.db.insert_track_to_playlist(&playlist_id, &track_id)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_playlist_tracks(
    playlist_id: u32,
    state: TauriState,
) -> Result<PlaylistWithTracks, FrontendError> {
    let playlist = state.db.get_playlist_with_tracks(&playlist_id)?;

    Ok(playlist)
}

#[tauri::command]
#[specta::specta]
pub fn remove_from_playlist(
    playlist_id: u32,
    track_id: u32,
    state: TauriState,
) -> Result<(), FrontendError> {
    state
        .db
        .delete_track_from_playlist(&playlist_id, &track_id)?;

    Ok(())
}
