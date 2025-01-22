use std::{fs::File, io::Write};

use audio_metadata::Metadata;

use crate::{
    db::{data_path, Database},
    models::{Albums, Artists, Tracks},
};

/*
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
}*/

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

pub fn write_cover(file: &str) {
    let path = file.to_string();
    let ext = file.split('.').last().unwrap();
    match ext {
        "flac" | "mp3" => {
            let file = Metadata::from_file(std::path::Path::new(&path)).unwrap();
            let cover_path = cover_path(&file.artist, &file.album);
            if !std::path::Path::new(&cover_path).exists() {
                let mut cover = file.picture_data;
                if cover.is_empty() {
                    cover = std::fs::read("../../../public/placeholder.png").unwrap();
                }
                let mut file = File::create(cover_path).unwrap();
                file.write_all(&cover).unwrap();
            }
        }
        _ => (),
    }
}

pub fn first_time_metadata(files: &[String], music_folder: &str, db: &Database) {
    files.iter().for_each(|file| {
        let metadata = Metadata::from_file(std::path::Path::new(file)).unwrap();
        let artist = &db.artist_by_name(&metadata.artist);

        let artist_id = if let Some(artist) = artist {
            artist.id
        } else {
            let artist_path = get_artist_path(music_folder, &metadata.file_path);
            db.insert::<Artists>(Artists {
                id: 0,
                name: metadata.artist.clone(),
                path: artist_path.clone(),
            })
        };

        let album = &db.spec_album_by_artist_id(&metadata.album, &artist_id);
        let cover_path = cover_path(&metadata.artist, &metadata.album);

        let album_id = if let Some(album) = album {
            album.id
        } else {
            write_cover(file);
            db.insert::<Albums>(Albums {
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

        let track = &db.track_by_album_id(&metadata.name, &album_id);

        if track.is_none() {
            db.insert::<Tracks>(Tracks {
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
