use crate::{
    SodapopState,
    error::FrontendError,
    systems::utils::{data_path, get_handle_to_music_folder, sanitize_string},
};

use anyhow::Context;
use common::{AlbumType, Albums, Artists, NewAlbum, NewArtist, NewTrack, Tracks, traits::Hashable};
use metadata_audio::Metadata;
use serde::Serialize;
use specta::Type;

use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::{Path, PathBuf},
};

use tauri::{Manager, ipc::Channel};

#[derive(Clone, Serialize, Type)]
#[serde(tag = "event", content = "data")]
// rust-analyzer expected Expr error: https://github.com/specta-rs/specta/issues/387
pub enum MetadataEvent {
    Started { id: usize },
    Total { id: usize, total: usize },
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
        let event_id = 1;
        on_event.send(MetadataEvent::Started { id: event_id })?;

        let all_track_files = Metadata::recursive_dir(path);
        on_event.send(MetadataEvent::Total {
            id: event_id,
            total: all_track_files.len(),
        })?;

        let mut buffer = Vec::with_capacity(1024 * 64);

        let mut existing_hashes: HashSet<String> = HashSet::new();
        let mut existing_artists: HashMap<String, u32> = state
            .db
            .all::<Artists>()?
            .into_iter()
            .map(|a| (a.name, a.id))
            .collect();
        let mut existing_albums: HashMap<(u32, String), u32> = state
            .db
            .all::<Albums>()?
            .into_iter()
            .map(|a| ((a.artist_id, a.name), a.id))
            .collect();

        let mut album_covers: HashMap<u32, String> = state
            .db
            .all::<Albums>()?
            .into_iter()
            .map(|a| (a.id, a.cover_path))
            .collect();

        let mut albums_seen = HashSet::new();

        for (idx, track_path) in all_track_files.iter().enumerate() {
            buffer.clear();

            let skip_picture = if let Some(album_folder) = track_path.parent() {
                let contains = albums_seen.contains(&album_folder);

                if !contains {
                    albums_seen.insert(album_folder);
                }

                contains
            } else {
                false
            };

            let metadata = match Metadata::from_file(&mut buffer, track_path, skip_picture) {
                Ok(m) => m,
                Err(e) => {
                    logging::error!(
                        "Failed to read metadata for {}: {:?}",
                        track_path.display(),
                        e
                    );
                    continue;
                }
            };

            if let (Some(artist), Some(album), Some(name)) =
                (metadata.artist, metadata.album, metadata.name)
            {
                let year = metadata.year.unwrap_or(0);

                let artist_id = if let Some(&id) = existing_artists.get(artist) {
                    id
                } else {
                    state
                        .db
                        .insert::<NewArtist>(NewArtist { name: artist })
                        .with_context(|| {
                            format!("Failed to insert artist for {}", track_path.display())
                        })?;
                    let id = state.db.latest::<Artists>().unwrap().id;
                    existing_artists.insert(artist.to_owned(), id);
                    id
                };

                let album_path = get_album_path(path, track_path);

                let (album_id, cover_path) =
                    if let Some(&id) = existing_albums.get(&(artist_id, album.to_owned())) {
                        let cover_path = album_covers.get(&id).unwrap();
                        (id, cover_path.clone())
                    } else {
                        let mut cover_path = get_cover_path(artist, album);

                        if !Path::new(&cover_path).exists() {
                            if let Some(picture_data) = &metadata.picture_data {
                                fs::write(&cover_path, &**picture_data).with_context(|| {
                                    format!(
                                        "Failed to write the raw picture data to disk for {}",
                                        track_path.display()
                                    )
                                })?;
                            } else {
                                let album_path = Path::new(&album_path);
                                if album_path.join("cover.jpg").exists() {
                                    fs::copy(album_path.join("cover.jpg"), &cover_path)
                                        .with_context(|| {
                                            format!(
                                                "Failed to copy cover.jpg for {}",
                                                track_path.display()
                                            )
                                        })?;
                                } else if album_path.join("cover.png").exists() {
                                    fs::copy(album_path.join("cover.png"), &cover_path)
                                        .with_context(|| {
                                            format!(
                                                "Failed to copy cover.png for {}",
                                                track_path.display()
                                            )
                                        })?;
                                } else {
                                    cover_path = data_path()
                                        .join("covers")
                                        .join("placeholder.png")
                                        .to_str()
                                        .unwrap()
                                        .to_owned();
                                }
                            }
                        }

                        state
                            .db
                            .insert_album::<NewAlbum>(NewAlbum {
                                artist_id,
                                artist_name: artist,
                                name: album,
                                cover_path: &cover_path,
                                year,
                                album_type: &AlbumType::Unknown,
                                track_count: 0,
                                duration: 0,
                                path: &album_path.to_string_lossy(),
                            })
                            .with_context(|| {
                                format!("Failed to insert album for {}", track_path.display())
                            })?;

                        let id = state.db.latest::<Albums>()?.id;
                        existing_albums.insert((artist_id, album.to_owned()), id);
                        album_covers.insert(id, cover_path.clone());

                        (id, cover_path)
                    };

                let new_track = NewTrack {
                    duration: metadata.duration.round() as u32,
                    album_name: album,
                    album_id,
                    artist_name: artist,
                    artist_id,
                    name,
                    number: metadata.track_number.map_or(-1, |n| n as i32),
                    path: &track_path.to_string_lossy(),
                    cover_path: &cover_path,
                };

                let hash = new_track.make_hash();

                let track_exists = state.db.exists::<Tracks>("hash", &hash)?;

                if !track_exists {
                    state.db.insert::<NewTrack>(new_track).with_context(|| {
                        format!("Failed to insert track for {}", track_path.display())
                    })?;
                };

                existing_hashes.insert(hash);

                on_event.send(MetadataEvent::Progress {
                    id: event_id,
                    current: idx,
                })?;
            }
        }

        on_event.send(MetadataEvent::Finished { id: event_id })?;

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

fn get_cover_path(artist: &str, album: &str) -> String {
    // have to sanitize the artist and album names to avoid issues with file paths
    let p = data_path().to_str().unwrap().to_owned();
    p + "/covers/" + &sanitize_string(artist) + " - " + &sanitize_string(album) + ".jpg"
}

fn get_album_path(music_folder: &Path, full_path: &Path) -> PathBuf {
    let rel = full_path.strip_prefix(music_folder).unwrap();

    let comps: Vec<_> = rel.parent().unwrap().components().collect();

    match comps.len() {
        0 => music_folder.to_path_buf(),
        1 => music_folder.join(comps[0]), // only album
        _ => music_folder.join(comps[0]).join(comps[1]), // artist/album or album/subfolder
    }
}
