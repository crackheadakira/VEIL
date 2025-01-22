use crate::{
    first_time_metadata,
    models::{Albums, Artists, Tracks},
    SodapopState,
};
use std::{fs, sync::Mutex};
use tauri::Manager;
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

    let state = app.state::<Mutex<SodapopState>>();
    let state_guard = state.lock().unwrap();

    if let Some(path) = file_path {
        let path = path.try_into().unwrap();
        let mut all_paths = recursive_dir_to_strings(&path);
        all_paths.sort();

        let start = std::time::Instant::now();
        first_time_metadata(&all_paths, path.to_str().unwrap(), &state_guard.db);
        println!("First time metadata read time: {:?}", start.elapsed());

        let all_tracks = &state_guard.db.all::<Tracks>();

        for track in all_tracks {
            if !all_paths.contains(&track.path) {
                state_guard.db.delete::<Tracks>(track.id);

                if state_guard.db.count::<Tracks>(track.albums_id, "albums_id") == 0 {
                    state_guard.db.delete::<Albums>(track.albums_id);

                    if state_guard
                        .db
                        .count::<Albums>(track.artists_id, "artists_id")
                        == 0
                    {
                        state_guard.db.delete::<Artists>(track.artists_id);
                    }
                }
            } else {
                let duration = &state_guard.db.get_album_duration(&track.albums_id);
                let album_type = get_album_type(duration.1, duration.0);
                state_guard
                    .db
                    .update_album_type(&track.albums_id, &album_type, duration);
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
            if extension != "mp3" && extension != "flac" {
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
        String::from("Single")
    } else if tracks <= 6 && duration < 1800 {
        String::from("EP")
    } else {
        String::from("Album")
    }
}
