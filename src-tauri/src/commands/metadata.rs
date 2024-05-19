use std::{fs::File, io::Write};

use audiotags::{traits::*, FlacTag, Id3v2Tag, Mp4Tag};
use serde::Serialize;
use specta::Type;

use crate::{
    db::data_path,
    interface::album::*,
    interface::artist::*,
    interface::track::*,
    models::{Albums, Tracks},
};

#[derive(Debug, Serialize, Clone, Type)]
pub struct Metadata {
    album: String,
    artist: String,
    name: String,
    path: String,
    album_type: String,
    year: i32,
}

#[tauri::command]
#[specta::specta]
pub fn read_metadata(file: String) -> Metadata {
    let path = file.to_string();
    let ext = path.split('.').last().unwrap();
    match ext {
        "mp3" => {
            let tag = Id3v2Tag::read_from_path(&path).unwrap();
            let total_tracks = tag.total_tracks().unwrap_or(0);
            let duration = tag.duration().unwrap_or(0.0);
            Metadata {
                path,
                artist: tag.artist().unwrap().to_string(),
                name: tag.title().unwrap().to_string(),
                album: tag.album().unwrap().title.to_string(),
                album_type: get_album_type(total_tracks, duration),
                year: tag.year().unwrap(),
            }
        }
        "flac" => {
            let tag = FlacTag::read_from_path(&path).unwrap();
            let total_tracks = tag.total_tracks().unwrap_or(0);
            let duration = tag.duration().unwrap_or(0.0);
            Metadata {
                path,
                artist: tag.artist().unwrap().to_string(),
                name: tag.title().unwrap().to_string(),
                album: tag.album().unwrap().title.to_string(),
                album_type: get_album_type(total_tracks, duration),
                year: tag.year().unwrap(),
            }
        }
        "m4a" => {
            let tag = Mp4Tag::read_from_path(&path).unwrap();
            let total_tracks = tag.total_tracks().unwrap_or(0);
            let duration = tag.duration().unwrap_or(0.0);
            Metadata {
                path,
                artist: tag.artist().unwrap().to_string(),
                name: tag.title().unwrap().to_string(),
                album: tag.album().unwrap().title.to_string(),
                album_type: get_album_type(total_tracks, duration),
                year: tag.year().unwrap(),
            }
        }
        _ => Metadata {
            path,
            artist: "Unknown".to_string(),
            name: "Unknown".to_string(),
            album: "Unknown".to_string(),
            album_type: "Unknown".to_string(),
            year: 0,
        },
    }
}

fn cover_path(artist: &str, album: &str) -> String {
    data_path().to_string() + "/covers/" + artist + " - " + album + ".jpg"
}

// Singles are less than 3 tracks and 30 minutes,
// EPs are up to 6 tracks and 30 minutes,
// LPs/Albums are more than 6 tracks and 30 minutes.
fn get_album_type(tracks: u16, duration: f64) -> String {
    if tracks < 3 && duration < 1800.0 {
        "Single".to_string()
    } else if tracks <= 6 && duration < 1800.0 {
        "EP".to_string()
    } else {
        "Album".to_string()
    }
}

pub fn write_cover(file: &str) {
    let path = file.to_string();
    let ext = file.split('.').last().unwrap();
    match ext {
        "mp3" => {
            let tag = Id3v2Tag::read_from_path(&path).unwrap();
            let album = tag.album_title().unwrap();
            let artist = tag.artist().unwrap();
            let cover_path = cover_path(&artist, &album);
            if !std::path::Path::new(&cover_path).exists() {
                let cover = tag.album_cover().unwrap();
                let mut file = File::create(cover_path).unwrap();
                file.write_all(&cover.data).unwrap();
            }
        }
        "flac" => {
            let tag = FlacTag::read_from_path(&path).unwrap();
            let album = tag.album_title().unwrap();
            let artist = tag.artist().unwrap();
            let cover_path = cover_path(&artist, &album);
            if !std::path::Path::new(&cover_path).exists() {
                let cover = tag.album_cover().unwrap();
                let mut file = File::create(cover_path).unwrap();
                file.write_all(&cover.data).unwrap();
            }
        }
        "m4a" => {
            let tag = Mp4Tag::read_from_path(&path).unwrap();
            let album = tag.album_title().unwrap();
            let artist = tag.artist().unwrap();
            let cover_path = cover_path(&artist, &album);
            if !std::path::Path::new(&cover_path).exists() {
                let cover = tag.album_cover().unwrap();
                let mut file = File::create(cover_path).unwrap();
                file.write_all(&cover.data).unwrap();
            }
        }
        _ => (),
    }
}

pub fn first_time_metadata(files: &Vec<String>, music_folder: &str) -> Vec<Metadata> {
    let metadata: Vec<Metadata> = files
        .iter()
        .map(|file| {
            let album_id;
            let artist_id;
            let metadata = read_metadata(file.to_string());
            let artist = artist_by_name(&metadata.artist);

            if artist.is_none() {
                let artist_path = get_artist_path(music_folder, &metadata.path);
                artist_id = new_artist(&metadata.artist, &artist_path);
            } else {
                artist_id = artist.unwrap().id
            }

            let album = spec_album_by_artist_id(&metadata.album, &artist_id);

            if album.is_none() {
                album_id = new_album(Albums {
                    id: 0,
                    artists_id: artist_id,
                    name: metadata.album.clone(),
                    cover_path: cover_path(&metadata.artist, &metadata.album),
                    year: metadata.year,
                    album_type: metadata.album_type.clone(),
                    path: get_album_path(music_folder, &metadata.path),
                });
                write_cover(&file);
            } else {
                album_id = album.unwrap().id
            }

            let track = track_by_album_id(&metadata.name, &album_id);

            if track.is_none() {
                new_track(Tracks {
                    id: 0,
                    album: metadata.album.clone(),
                    albums_id: album_id,
                    artist: metadata.artist.clone(),
                    name: metadata.name.clone(),
                    path: metadata.path.clone(),
                });
            }

            metadata
        })
        .collect();

    metadata
}

pub fn get_artist_path(music_folder: &str, full_path: &str) -> String {
    let path = full_path.replace(music_folder, "");
    let path = path.split('/').collect::<Vec<&str>>()[1];
    music_folder.to_string() + "/" + path
}

pub fn get_album_path(music_folder: &str, full_path: &str) -> String {
    let path = full_path.replace(music_folder, "");
    let path = path.split('/').collect::<Vec<&str>>()[1..3].join("/");
    music_folder.to_string() + "/" + &path
}
