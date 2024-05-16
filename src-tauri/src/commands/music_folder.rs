use crate::interface::album::{album_by_path, album_tracks_length, delete_album};
use crate::interface::artist::{artist_albums_length, delete_artist};
use crate::interface::track::{delete_track, get_all_tracks_path};
use crate::{first_time_metadata, get_album_path, get_artist_path, Metadata};
use std::fs;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn select_music_folder(app: tauri::AppHandle) -> Vec<Metadata> {
    let file_path = app.dialog().file().blocking_pick_folder();

    match file_path {
        Some(path) => {
            let mut all_paths = recursive_dir(&path.to_str().unwrap());
            all_paths.sort();
            let metadata = first_time_metadata(&all_paths, &path.to_str().unwrap());
            let db_paths = get_all_tracks_path();

            for db_path in db_paths {
                if !all_paths.contains(&db_path) {
                    let unwrapped = path.to_str().unwrap();
                    let artist_path = get_artist_path(unwrapped, &db_path);
                    let album_path = get_album_path(unwrapped, &db_path);
                    let album = album_by_path(&album_path);

                    delete_track(&db_path);
                    if album_tracks_length(&album.id) == 0 {
                        delete_album(&album_path);
                        if artist_albums_length(&album.artists_id) == 0 {
                            delete_artist(&artist_path);
                        }
                    }
                }
            }

            metadata
        }
        None => Vec::new(),
    }
}

fn recursive(path: &str) -> Vec<String> {
    let paths = fs::read_dir(path).unwrap();
    let mut tracks = Vec::new();
    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            tracks.append(&mut recursive(path.to_str().unwrap()));
        } else {
            let extension = path.extension().unwrap();
            if extension != "mp3" && extension != "flac" && extension != "m4a" {
                continue;
            }
            tracks.push(path.display().to_string());
        }
    }
    tracks
}

pub fn recursive_dir(path: &str) -> Vec<String> {
    let paths = fs::read_dir(path).unwrap();
    let mut tracks = Vec::new();
    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            tracks.append(&mut recursive(path.to_str().unwrap()));
        } else {
            let extension = path.extension().unwrap();
            if extension != "mp3" && extension != "flac" && extension != "m4a" {
                continue;
            }
            tracks.push(path.display().to_string());
        }
    }
    tracks
}
