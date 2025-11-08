use crate::{
    SodapopState,
    error::FrontendError,
    systems::utils::{data_path, get_handle_to_music_folder, sanitize_string},
};

use common::{AlbumType, Albums, Artists, NewAlbum, NewArtist, NewTrack, Tracks, traits::Hashable};
use metadata_audio::Metadata;
use serde::Serialize;
use specta::Type;

use std::{
    collections::HashSet,
    fs::{self},
    path::Path,
};

use tauri::{Manager, ipc::Channel};

#[derive(Clone, Serialize, Type)]
#[serde(tag = "event", content = "data")]
// rust-analyzer expected Expr error: https://github.com/specta-rs/specta/issues/387
pub enum MetadataEvent {
    Started { id: usize, total: usize },
    Progress { id: usize, current: usize },
    Finished { id: usize },
}

#[tauri::command]
#[specta::specta]
pub async fn select_music_folder(
    app: tauri::AppHandle,
    on_event: Channel<MetadataEvent>,
) -> Result<String, FrontendError> {
    let state = app.state::<SodapopState>();

    if let Some(handle) = get_handle_to_music_folder(&state).await? {
        let path = handle.path();
        let all_track_files = Metadata::collect_album_files_for_smart(path)?;

        let total_files = all_track_files.iter().map(|album| album.len()).sum();

        let event_id = 1;
        on_event.send(MetadataEvent::Started {
            id: event_id,
            total: total_files,
        })?;

        // If I were to attempt zero-copy reading, then this function would need
        // to own the data passed into `from_files_smart`.
        let all_metadata = Metadata::from_files_smart(
            &all_track_files,
            total_files,
            Some(|current| {
                let _ = on_event.send(MetadataEvent::Progress {
                    id: event_id,
                    current,
                });
            }),
        )?;

        on_event.send(MetadataEvent::Finished { id: event_id })?;

        // New flow could be to parse & insert in same step instead of waiting
        // for whole folder to be parsed we insert into database the moment it's
        // done.
        // I don't believe this new method could be multi-threaded though as we're
        // constantly relying on the previous tracks inserted / checking if they exist
        // so batching could work instead technically.

        /*
            INIT REUSABLE FILE BUFFER ( 128 KB? )
            FOR file IN ALL_TRACK_FILES
                OPEN file
                WRAP file IN BufReader
                PARSE metadata FROM BufReader
                INSERT parsed metadata INTO database
                SEND event TO frontend
        */

        // Store a HashMap of already found artist IDs & album IDs.
        let mut existing_hashes: HashSet<String> = HashSet::new();
        for metadata in all_metadata.into_iter() {
            let artist_exists = state.db.exists::<Artists>("name", &metadata.artist)?;

            let artist_id = if artist_exists {
                state.db.artist_by_name(&metadata.artist)?.id
            } else {
                state.db.insert::<NewArtist>(NewArtist {
                    name: &metadata.artist,
                })?;

                state.db.latest::<Artists>()?.id
            };

            let album_path = get_album_path(path.to_str().unwrap(), &metadata.file_path);
            let cover_path = cover_path(&metadata.artist, &metadata.album);

            let (album_id, cover_path) =
                if let Some(a) = state.db.album_exists(&metadata.album, metadata.year)? {
                    (a.id, a.cover_path)
                } else {
                    let mut cover_path = cover_path;

                    if !Path::new(&cover_path).exists() {
                        if let Some(picture_data) = &metadata.picture_data {
                            fs::write(&cover_path, &**picture_data)?;
                        } else {
                            let album_path = Path::new(&album_path);

                            // If there is no picture data, check if there exists
                            // either cover.jpg or cover.png, and then copy that
                            // over. If not, just point cover_path to placeholder.png
                            if album_path.join("cover.jpg").exists() {
                                fs::copy(album_path.join("cover.jpg"), &cover_path)?;
                            } else if album_path.join("cover.png").exists() {
                                fs::copy(album_path.join("cover.png"), &cover_path)?;
                            } else {
                                cover_path = data_path()
                                    .join("covers")
                                    .join("placeholder.png")
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                            }
                        }
                    }

                    state.db.insert_album::<NewAlbum>(NewAlbum {
                        artist_id,
                        artist_name: &metadata.artist,
                        name: &metadata.album,
                        cover_path: &cover_path,
                        year: metadata.year,
                        album_type: &metadata.album_type.as_str().into(),
                        track_count: 0,
                        duration: 0,
                        path: &album_path,
                    })?;

                    let a_id = state.db.latest::<Albums>()?.id;

                    (a_id, cover_path)
                };

            let new_track = NewTrack {
                duration: metadata.duration.round() as u32,
                album_name: &metadata.album,
                album_id,
                artist_name: &metadata.artist,
                artist_id,
                name: &metadata.name,
                number: metadata.track_number,
                path: &metadata.file_path,
                cover_path: &cover_path,
            };

            let hash = new_track.make_hash();

            let track_exists = state.db.exists::<Tracks>("hash", &hash)?;

            if !track_exists {
                state.db.insert::<NewTrack>(new_track)?;
            };

            existing_hashes.insert(hash);
        }

        // Remove tracks that are no longer in the music folder
        let all_tracks = &state.db.all::<Tracks>()?;

        // Stored album covers aren't cleaned up upon album deletion.
        for track in all_tracks.iter() {
            let track_in_db = existing_hashes.contains(&track.hash);

            if !track_in_db {
                state.db.delete::<Tracks>(track.id)?;
                let album_tracks = state.db.count::<Tracks>(track.album_id, "album_id", None)?;
                if album_tracks == 0 {
                    state.db.delete::<Albums>(track.album_id)?;

                    let artist_albums = state.db.count::<Albums>(
                        track.artist_id,
                        "artist_id",
                        Some("album_artists"),
                    )?;
                    if artist_albums == 0 {
                        state.db.delete::<Artists>(track.artist_id)?;
                    }
                }
            } else {
                let (total_duration, track_count) = state.db.get_album_duration(track.album_id)?;
                let album_type = get_album_type(track_count, total_duration);
                state.db.update_album_type(
                    track.album_id,
                    album_type,
                    total_duration,
                    track_count,
                )?;
            }
        }

        Ok(String::from(path.to_str().unwrap()))
    } else {
        Ok(String::from(""))
    }
}

/// Singles are less than 3 tracks and 30 minutes,
/// EPs are up to 6 tracks and 30 minutes,
/// LPs/Albums are more than 6 tracks and 30 minutes.
fn get_album_type(track_count: u32, duration: u32) -> AlbumType {
    if duration == 0 || track_count == 0 {
        AlbumType::Unknown
    } else if track_count < 3 && duration < 1800 {
        AlbumType::Single
    } else if track_count <= 6 && duration < 1800 {
        AlbumType::EP
    } else {
        AlbumType::Album
    }
}

fn cover_path(artist: &str, album: &str) -> String {
    // have to sanitize the artist and album names to avoid issues with file paths
    let p = data_path().to_str().unwrap().to_owned();
    p + "/covers/" + &sanitize_string(artist) + " - " + &sanitize_string(album) + ".jpg"
}

fn get_album_path(music_folder: &str, full_path: &str) -> String {
    let path = full_path.replace(music_folder, "");
    let path = path.split('/').collect::<Vec<&str>>()[1..3].join("/");
    music_folder.to_string() + "/" + &path
}
