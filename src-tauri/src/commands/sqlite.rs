use crate::interface::album::*;
use crate::interface::artist::*;
use crate::interface::track::*;
use crate::models::*;

#[tauri::command]
#[specta::specta]
pub fn get_sqlite() -> String {
    /*
    let artists = get_all_artists();

    let mut data = Vec::new();
    for artist in artists {
        let artist_albums = get_artist_albums(artist.id);
        for album in &artist_albums {
            let album_tracks = get_album_tracks(album.id);
            data.push((artist.clone(), (vec![album.clone()], album_tracks)));
        }
    }

    data
     */
    "Not implemented yet".to_string()
}

#[tauri::command]
#[specta::specta]
pub fn get_album_with_tracks(id: u32) -> AlbumWithTracks {
    album_with_tracks(&id)
}

#[tauri::command]
#[specta::specta]
pub fn track_by_id(id: u32) -> Tracks {
    get_track_by_id(&id)
}

#[tauri::command]
#[specta::specta]
pub fn get_all_albums() -> Vec<Albums> {
    all_albums()
}

#[tauri::command]
#[specta::specta]
pub fn get_artist_with_albums(id: u32) -> ArtistWithAlbums {
    artist_with_albums(&id)
}
