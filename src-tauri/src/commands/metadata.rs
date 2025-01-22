use std::{fs::File, io::Write, path::Path};

use audio_metadata::Metadata;

use crate::{
    db::{data_path, Database},
    models::{Albums, Artists, Tracks},
};

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

pub fn write_cover(metadata: &Metadata, cover_path: &str) {
    if !Path::new(&cover_path).exists() {
        let cover = if metadata.picture_data.is_empty() {
            &include_bytes!("../../../public/placeholder.png").to_vec()
        } else {
            &metadata.picture_data
        };
        let mut file = File::create(cover_path).unwrap();
        file.write_all(&cover).unwrap();
    }
}

pub fn first_time_metadata(files: &[String], music_folder: &str, db: &Database) {
    files.iter().for_each(|file| {
        let metadata = Metadata::from_file(Path::new(file)).unwrap();
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
            write_cover(&metadata, &cover_path);
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
