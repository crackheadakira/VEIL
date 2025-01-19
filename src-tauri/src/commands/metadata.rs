use std::{fs::File, io::Write};

use audio_metadata::Metadata;
use audiotags::{traits::*, Id3v2Tag, Mp4Tag};

use crate::{
    db::data_path,
    interface::{album::*, artist::*, track::*},
    models::{Albums, Tracks},
    recursive_dir,
};

#[tauri::command]
#[specta::specta]
pub fn read_metadata(file: String) -> Metadata {
    let path = file.to_string();
    let ext = path.split('.').last().unwrap();
    match ext {
        "mp3" => {
            let tag = Id3v2Tag::read_from_path(&path).unwrap();
            let duration = tag.duration().unwrap_or(0.0) as f32;
            Metadata {
                duration,
                file_path: path,
                artist: tag.artist().unwrap().to_string(),
                name: tag.title().unwrap().to_string(),
                album: tag.album().unwrap().title.to_string(),
                album_type: "Unknown".to_string(),
                year: tag.year().unwrap() as u16,
                track_number: tag.track_number().unwrap_or(0),
                picture_data: tag.album_cover().unwrap().data.to_vec(),
            }
        }
        "flac" => Metadata::from_file(std::path::Path::new(&path)).unwrap(),
        "m4a" => {
            let tag = Mp4Tag::read_from_path(&path).unwrap();
            let duration = tag.duration().unwrap_or(0.0) as f32;
            Metadata {
                duration,
                file_path: path,
                artist: tag.artist().unwrap().to_string(),
                name: tag.title().unwrap().to_string(),
                album: tag.album().unwrap().title.to_string(),
                album_type: "Unknown".to_string(),
                year: tag.year().unwrap() as u16,
                track_number: tag.track_number().unwrap_or(0),
                picture_data: tag.album_cover().unwrap().data.to_vec(),
            }
        }
        _ => Metadata {
            duration: 0.0,
            file_path: path,
            artist: "Unknown".to_string(),
            name: "Unknown".to_string(),
            album: "Unknown".to_string(),
            album_type: "Unknown".to_string(),
            year: 0,
            track_number: 0,
            picture_data: Vec::new(),
        },
    }
}

#[tauri::command]
#[specta::specta]
pub async fn async_metadata(music_folder: String) {
    println!("Async metadata start");
    let music_folder_path = std::path::PathBuf::from(&music_folder);
    let files = recursive_dir(&music_folder_path);
    let start = std::time::Instant::now();
    let metadatas = audio_metadata::Metadata::from_files(&files, "flac").await;
    println!("Async metadata read time: {:?}", start.elapsed());

    for metadata in metadatas.unwrap() {
        let artist = artist_by_name(&metadata.artist);

        let artist_id = if let Some(artist) = artist {
            artist.id
        } else {
            let artist_path = get_artist_path(&music_folder, &metadata.file_path);
            new_artist(&metadata.artist, &artist_path)
        };

        let album = spec_album_by_artist_id(&metadata.album, &artist_id);
        let cover_path = cover_path(&metadata.artist, &metadata.album);

        let album_id = if let Some(album) = album {
            album.id
        } else {
            write_cover(&metadata.file_path, &music_folder);
            new_album(Albums {
                id: 0,
                path: get_album_path(&music_folder, &metadata.file_path),
                artists_id: artist_id,
                artist: metadata.artist.clone(),
                name: metadata.album.clone(),
                cover_path: cover_path.clone(),
                year: metadata.year,
                album_type: metadata.album_type.clone(),
                track_count: 0,
                duration: 0,
            })
        };

        let track = track_by_album_id(&metadata.name, &album_id);

        if track.is_none() {
            new_track(Tracks {
                id: 0,
                duration: metadata.duration.round() as u32,
                album: metadata.album.clone(),
                albums_id: album_id,
                artist: metadata.artist.clone(),
                artists_id: artist_id,
                name: metadata.name.clone(),
                path: metadata.file_path.clone(),
                cover_path: cover_path,
            });
        }
    }

    println!("Async metadata time: {:?}", start.elapsed());
}

fn cover_path(artist: &str, album: &str) -> String {
    data_path().to_string() + "/covers/" + artist + " - " + album + ".jpg"
}

pub fn write_cover(file: &str, music_folder: &str) {
    let path = file.to_string();
    let ext = file.split('.').last().unwrap();
    match ext {
        "mp3" => {
            let tag = Id3v2Tag::read_from_path(&path).unwrap();
            let album = tag.album_title().unwrap();
            let artist = tag.artist().unwrap();
            let cover_path = cover_path(artist, album);
            if !std::path::Path::new(&cover_path).exists() {
                let cover = tag.album_cover();
                match cover {
                    Some(cover) => {
                        let mut file = File::create(cover_path).unwrap();
                        file.write_all(cover.data).unwrap();
                    }
                    None => find_folder_cover(&cover_path, &path, music_folder),
                }
            }
        }
        "flac" => {
            let file = Metadata::from_file(std::path::Path::new(&path)).unwrap();
            let cover_path = cover_path(&file.artist, &file.album);
            if !std::path::Path::new(&cover_path).exists() {
                let cover = file.picture_data;
                let mut file = File::create(cover_path).unwrap();
                file.write_all(&cover).unwrap();
            }
        }
        "m4a" => {
            let tag = Mp4Tag::read_from_path(&path).unwrap();
            let album = tag.album_title().unwrap();
            let artist = tag.artist().unwrap();
            let cover_path = cover_path(artist, album);
            if !std::path::Path::new(&cover_path).exists() {
                let cover = tag.album_cover();
                match cover {
                    Some(cover) => {
                        let mut file = File::create(cover_path).unwrap();
                        file.write_all(cover.data).unwrap();
                    }
                    None => find_folder_cover(&cover_path, &path, music_folder),
                }
            }
        }
        _ => (),
    }
}

fn find_folder_cover(cover_path: &str, path: &str, music_folder: &str) {
    let folder_cover = get_album_path(music_folder, path) + "/cover.jpg";
    if std::path::Path::new(&folder_cover).exists() {
        let cover = std::fs::read(folder_cover).unwrap();
        let mut file = File::create(cover_path).unwrap();
        file.write_all(&cover).unwrap();
    } else {
        let cover = std::fs::read("../../../public/placeholder.png").unwrap();
        let mut file = File::create(cover_path).unwrap();
        file.write_all(&cover).unwrap();
    }
}

pub fn first_time_metadata(files: &[String], music_folder: &str) {
    files.iter().for_each(|file| {
        let metadata = read_metadata(file.to_string());
        let artist = artist_by_name(&metadata.artist);

        let artist_id = if let Some(artist) = artist {
            artist.id
        } else {
            let artist_path = get_artist_path(music_folder, &metadata.file_path);
            new_artist(&metadata.artist, &artist_path)
        };

        let album = spec_album_by_artist_id(&metadata.album, &artist_id);
        let cover_path = cover_path(&metadata.artist, &metadata.album);

        let album_id = if let Some(album) = album {
            album.id
        } else {
            write_cover(file, music_folder);
            new_album(Albums {
                id: 0,
                artists_id: artist_id,
                artist: metadata.artist.clone(),
                name: metadata.album.clone(),
                cover_path: cover_path.clone(),
                year: metadata.year,
                album_type: metadata.album_type.clone(),
                track_count: 0,
                duration: 0,
                path: get_album_path(music_folder, &metadata.file_path),
            })
        };

        let track = track_by_album_id(&metadata.name, &album_id);

        if track.is_none() {
            new_track(Tracks {
                id: 0,
                duration: metadata.duration.round() as u32,
                album: metadata.album.clone(),
                albums_id: album_id,
                artist: metadata.artist.clone(),
                artists_id: artist_id,
                name: metadata.name.clone(),
                path: metadata.file_path.clone(),
                cover_path,
            });
        }
    });
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
