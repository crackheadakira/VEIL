use crate::db::db_connect;
use crate::interface::album::*;
use crate::models::Artists;
use crate::models::*;
use rusqlite::{Error, Result, Row};

pub fn all_artists() -> Vec<Artists> {
    let conn = db_connect();

    let mut stmt = conn.prepare("SELECT * FROM artists").unwrap();
    let result: Result<Vec<Artists>> = stmt.query_map([], stmt_to_artist).unwrap().collect();

    match result {
        Ok(artists) => artists,
        Err(e) => panic!("Error fetching all artists: {}", e),
    }
}

pub fn artist_albums_length(artist_id: &u32) -> u32 {
    let conn = db_connect();

    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM albums WHERE artists_id = ?1")
        .unwrap();
    let result = stmt.query_row([artist_id], |row| row.get(0));

    match result {
        Ok(count) => count,
        Err(e) => panic!("Error fetching artist albums length: {}", e),
    }
}

pub fn artist_by_name(name: &str) -> Option<Artists> {
    let conn = db_connect();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM artists WHERE name = ?1")
        .unwrap();
    let result: Result<Artists> = stmt.query_row([name], stmt_to_artist);

    match result {
        Ok(artist) => Some(artist),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => panic!("Error fetching artist: {}", e),
    }
}

pub fn artist_by_id(id: &u32) -> Artists {
    let conn = db_connect();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM artists WHERE ID = ?1")
        .unwrap();
    let result = stmt.query_row([id], stmt_to_artist);

    result.unwrap()
}

pub fn artist_with_albums(id: &u32) -> ArtistWithAlbums {
    let artist = artist_by_id(id);
    let albums = album_by_artist_id(id);

    let albums_with_tracks = albums
        .iter()
        .map(|album| album_with_tracks(&album.id))
        .collect();

    ArtistWithAlbums {
        artist,
        albums: albums_with_tracks,
    }
}

pub fn new_artist(artist: &str, path: &str) -> u32 {
    let conn = db_connect();
    let stmt = conn.prepare_cached("INSERT INTO artists (name, path) VALUES (?1, ?2)");
    let result = stmt.unwrap().execute((artist, path));

    match result {
        Ok(_) => conn.last_insert_rowid() as u32,
        Err(e) => panic!("Error inserting artist: {}", e),
    }
}

pub fn delete_artist(artist_id: &u32) {
    let conn = db_connect();
    let mut stmt = conn
        .prepare("DELETE FROM artists WHERE ID = ?1 OR name = ?1 OR path = ?1")
        .unwrap();
    let result = stmt.execute([artist_id]);

    match result {
        Ok(_) => (),
        Err(e) => panic!("Error deleting artist: {}", e),
    }
}

pub fn stmt_to_artist(row: &Row) -> Result<Artists, Error> {
    Ok(Artists {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
    })
}
