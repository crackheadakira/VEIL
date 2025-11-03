use crate::{
    SodapopState,
    error::FrontendError,
    systems::utils::{data_path, get_handle_to_music_folder, sanitize_string},
};

use common::{AlbumType, Albums, Artists, NewAlbum, NewArtist, NewTrack, Tracks};
use metadata_audio::Metadata;
use serde::Serialize;
use specta::Type;

use std::{
    fs::{self},
    path::{Path, PathBuf},
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
        let all_track_files = collect_album_tracks(path);
        let total_files = all_track_files.iter().map(|album| album.len()).sum();

        let event_id = 1;
        on_event.send(MetadataEvent::Started {
            id: event_id,
            total: total_files,
        })?;

        let mut all_metadata = Vec::with_capacity(total_files);
        for album_tracks in all_track_files.iter() {
            let first_track = &album_tracks[0];
            let first_metadata = Metadata::from_file(first_track, false)?;

            for (idx, track) in album_tracks[1..].iter().enumerate() {
                let metadata = Metadata::from_file(track, true);

                match metadata {
                    Ok(mut metadata) => {
                        if metadata.picture_data.is_none() {
                            metadata.picture_data = first_metadata.picture_data.clone();
                        }

                        all_metadata.push(metadata);

                        on_event.send(MetadataEvent::Progress {
                            id: event_id,
                            current: idx,
                        })?;
                    }

                    Err(_) => continue,
                }
            }

            all_metadata.push(first_metadata);
        }

        on_event.send(MetadataEvent::Finished { id: event_id })?;

        for metadata in &all_metadata {
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

            let track_exists = state.db.exists::<Tracks>("name", &metadata.name)?;

            if !track_exists {
                state.db.insert::<NewTrack>(NewTrack {
                    duration: metadata.duration.round() as u32,
                    album_name: &metadata.album,
                    album_id,
                    artist_name: &metadata.artist,
                    artist_id,
                    name: &metadata.name,
                    number: metadata.track_number,
                    path: &metadata.file_path,
                    cover_path: &cover_path,
                })?;
            };
        }

        // Remove tracks that are no longer in the music folder
        let all_tracks = &state.db.all::<Tracks>()?;

        for track in all_tracks.iter() {
            let exists_on_disk = all_track_files
                .iter()
                .flatten()
                .any(|p| p == &PathBuf::from(&track.path));

            if !exists_on_disk {
                state.db.delete::<Tracks>(track.id)?;

                let album_tracks = state.db.count::<Tracks>(track.album_id, "album_id")?;
                if album_tracks == 0 {
                    state.db.delete::<Albums>(track.album_id)?;

                    let artist_albums = state.db.count::<Albums>(track.artist_id, "artist_id")?;
                    if artist_albums == 0 {
                        state.db.delete::<Artists>(track.artist_id)?;
                    }
                }
            } else {
                let duration = &state.db.get_album_duration(&track.album_id)?;
                let album_type = get_album_type(duration.1, duration.0);
                state
                    .db
                    .update_album_type(&track.album_id, album_type, duration)?;
            }
        }

        Ok(String::from(path.to_str().unwrap()))
    } else {
        Ok(String::from(""))
    }
}

fn collect_album_tracks(path: &Path) -> Vec<Vec<PathBuf>> {
    let mut albums = Vec::new();

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            albums.extend(collect_album_tracks(&path));
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("mp3") || ext.eq_ignore_ascii_case("flac") {
                // Find album by parent
                if let Some(album_idx) = albums.iter().position(|a| a[0].parent() == path.parent())
                {
                    albums[album_idx].push(path);
                } else {
                    albums.push(vec![path]);
                }
            }
        }
    }

    albums
}

/// Singles are less than 3 tracks and 30 minutes,
/// EPs are up to 6 tracks and 30 minutes,
/// LPs/Albums are more than 6 tracks and 30 minutes.
fn get_album_type(tracks: u32, duration: u32) -> AlbumType {
    if duration == 0 || tracks == 0 {
        AlbumType::Unknown
    } else if tracks < 3 && duration < 1800 {
        AlbumType::Single
    } else if tracks <= 6 && duration < 1800 {
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
