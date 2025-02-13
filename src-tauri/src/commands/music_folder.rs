use crate::{
    db::data_path,
    error::FrontendError,
    models::{Albums, Artists, Tracks},
    SodapopState,
};

use audio_metadata::Metadata;

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[tauri::command(async)] // summon on async thread du to `.blocking_pick_folder()``
#[specta::specta]
pub fn select_music_folder(app: tauri::AppHandle) -> Result<(), FrontendError> {
    let folder_path = app
        .dialog()
        .file()
        .set_title("Select your music folder")
        .blocking_pick_folder();

    let state = app.state::<Mutex<SodapopState>>();
    let state_guard = state.lock().unwrap();

    if let Some(path) = folder_path {
        let path = path.try_into().unwrap();
        let mut all_paths = recursive_dir(&path);
        all_paths.sort();

        let start = std::time::Instant::now();
        let all_metadata = Metadata::from_files(&all_paths)?;

        for metadata in all_metadata {
            let artist_exists = state_guard.db.exists::<Artists>("name", &metadata.artist)?;

            let artist_id = if artist_exists {
                state_guard.db.artist_by_name(&metadata.artist)?.id
            } else {
                state_guard.db.insert::<Artists>(Artists {
                    id: 0,
                    name: metadata.artist.clone(),
                })?
            };

            let album_exists = state_guard.db.exists::<Albums>("name", &metadata.album)?;
            let album_path = get_album_path(path.to_str().unwrap(), &metadata.file_path);
            let cover_path = cover_path(&metadata.artist, &metadata.album);

            let album_id = if album_exists {
                state_guard
                    .db
                    .album_by_name(&metadata.album, &artist_id)?
                    .id
            } else {
                write_cover(&metadata, &cover_path, &album_path)?;
                state_guard.db.insert::<Albums>(Albums {
                    id: 0,
                    artist_id,
                    artist_name: metadata.artist.clone(),
                    name: metadata.album.clone(),
                    cover_path: cover_path.clone(),
                    year: metadata.year,
                    album_type: metadata.album_type,
                    track_count: 0,
                    duration: 0,
                    path: album_path,
                })?
            };

            let track_exists = state_guard.db.exists::<Tracks>("name", &metadata.name)?;

            if !track_exists {
                state_guard.db.insert::<Tracks>(Tracks {
                    id: 0,
                    duration: metadata.duration.round() as u32,
                    album_name: metadata.album.clone(),
                    album_id,
                    artist_name: metadata.artist.clone(),
                    artist_id,
                    name: metadata.name.clone(),
                    path: metadata.file_path.clone(),
                    cover_path,
                })?;
            };
        }

        // Remove tracks that are no longer in the music folder
        let all_tracks = &state_guard.db.all::<Tracks>()?;

        for track in all_tracks {
            if !all_paths.contains(&PathBuf::from(&track.path)) {
                state_guard.db.delete::<Tracks>(track.id)?;

                let album_tracks = state_guard.db.count::<Tracks>(track.album_id, "album_id")?;
                if album_tracks == 0 {
                    state_guard.db.delete::<Albums>(track.album_id)?;

                    let artist_albums = state_guard
                        .db
                        .count::<Albums>(track.artist_id, "artist_id")?;
                    if artist_albums == 0 {
                        state_guard.db.delete::<Artists>(track.artist_id)?;
                    }
                }
            } else {
                let duration = &state_guard.db.get_album_duration(&track.album_id)?;
                let album_type = get_album_type(duration.1, duration.0);
                state_guard
                    .db
                    .update_album_type(&track.album_id, &album_type, duration)?;
            }
        }

        println!("Finished indexing: {:?}", start.elapsed());
    }

    Ok(())
}

fn recursive_dir(path: &PathBuf) -> Vec<PathBuf> {
    let paths = fs::read_dir(path).unwrap();
    let mut tracks = Vec::new();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            tracks.extend(recursive_dir(&path));
        } else {
            let extension = path.extension().unwrap();
            if extension != "mp3" && extension != "flac" {
                continue;
            }

            tracks.push(path); // Return PathBuf directly
        }
    }

    tracks
}

// Singles are less than 3 tracks and 30 minutes,
// EPs are up to 6 tracks and 30 minutes,
// LPs/Albums are more than 6 tracks and 30 minutes.
pub fn get_album_type(tracks: u32, duration: u32) -> String {
    if tracks < 3 && duration < 1800 {
        String::from("Single")
    } else if tracks <= 6 && duration < 1800 {
        String::from("EP")
    } else {
        String::from("Album")
    }
}

fn sanitize_string(string: &str) -> String {
    string.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "")
}

fn cover_path(artist: &str, album: &str) -> String {
    // have to sanitize the artist and album names to avoid issues with file paths
    data_path().to_string()
        + "/covers/"
        + &sanitize_string(artist)
        + " - "
        + &sanitize_string(album)
        + ".jpg"
}

fn write_cover(
    metadata: &Metadata,
    cover_path: &str,
    album_path: &str,
) -> Result<(), FrontendError> {
    if !Path::new(&cover_path).exists() {
        let cover = if metadata.picture_data.is_empty() {
            if Path::new(&(album_path.to_string() + "/cover.jpg")).exists() {
                fs::copy(album_path.to_string() + "/cover.jpg", cover_path)?;
            } else if Path::new(&(album_path.to_string() + "/cover.png")).exists() {
                fs::copy(album_path.to_string() + "/cover.png", cover_path)?;
            }

            &include_bytes!("../../../public/placeholder.png").to_vec()
        } else {
            &metadata.picture_data
        };
        let mut file = File::create(cover_path)?;
        file.write_all(cover)?;
    }

    Ok(())
}

fn get_album_path(music_folder: &str, full_path: &str) -> String {
    let path = full_path.replace(music_folder, "");
    let path = path.split('/').collect::<Vec<&str>>()[1..3].join("/");
    music_folder.to_string() + "/" + &path
}
