use crate::first_time_metadata;
use crate::Metadata;
use std::fs;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn select_music_folder(app: tauri::AppHandle) -> Vec<Metadata> {
    let file_path = app.dialog().file().blocking_pick_folder();

    match file_path {
        Some(path) => {
            let mut path = recursive_dir(&path.to_str().unwrap());
            path.sort();
            first_time_metadata(&path)
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
