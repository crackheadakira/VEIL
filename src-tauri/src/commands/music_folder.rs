use crate::interface::album::*;
use crate::interface::artist::*;
use crate::interface::track::*;
use crate::{first_time_metadata, get_album_path};
use std::fs;
use tauri_plugin_dialog::DialogExt;

use std::path::PathBuf;

#[tauri::command]
#[specta::specta]
pub async fn select_music_folder(app: tauri::AppHandle) {
    let file_path = app
        .dialog()
        .file()
        .set_title("Select your music folder")
        .blocking_pick_folder();

    if let Some(path) = file_path {
        let mut all_paths = recursive_dir_to_strings(&path);
        all_paths.sort();

        let start = std::time::Instant::now();
        first_time_metadata(&all_paths, path.to_str().unwrap());
        println!("First time metadata read time: {:?}", start.elapsed());

        let db_paths = get_all_tracks_path();

        for db_path in db_paths {
            let album_path = get_album_path(path.to_str().unwrap(), &db_path);
            let album = album_by_path(&album_path);

            if !all_paths.contains(&db_path) {
                delete_track(&db_path);
                if album_tracks_length(&album.id) == 0 {
                    delete_album(&album.id);
                    if artist_albums_length(&album.artists_id) == 0 {
                        delete_artist(&album.artists_id);
                    }
                }
            } else {
                let duration = get_album_duration(&album.id);
                let album_type = get_album_type(duration.1, duration.0);
                update_album_type(&album.id, &album_type, &duration);
            }
        }

        println!("Finished indexing: {:?}", start.elapsed());
    }
}

pub fn recursive_dir(path: &PathBuf) -> Vec<PathBuf> {
    let paths = fs::read_dir(path).unwrap();
    let mut tracks = Vec::new();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            tracks.extend(recursive_dir(&path));
        } else {
            let extension = path.extension().unwrap();
            if extension != "mp3" && extension != "flac" && extension != "m4a" {
                continue;
            }

            tracks.push(path); // Return PathBuf directly
        }
    }

    tracks
}

pub fn recursive_dir_to_strings(path: &PathBuf) -> Vec<String> {
    let paths = recursive_dir(path);
    paths
        .into_iter()
        .map(|path| path.display().to_string())
        .collect()
}

// Singles are less than 3 tracks and 30 minutes,
// EPs are up to 6 tracks and 30 minutes,
// LPs/Albums are more than 6 tracks and 30 minutes.
fn get_album_type(tracks: u32, duration: u32) -> String {
    if tracks < 3 && duration < 1800 {
        "Single".to_string()
    } else if tracks <= 6 && duration < 1800 {
        "EP".to_string()
    } else {
        "Album".to_string()
    }
}
