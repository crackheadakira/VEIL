use crate::db::db_connect;
use crate::interface::album::*;
use crate::models::Artists;
use crate::models::*;
use rusqlite::{Error, Result, Row};

pub fn all_artists() -> Vec<Artists> {
    let conn = db_connect();

    let mut stmt = conn.prepare("SELECT * FROM artists").unwrap();
    let result: Result<Vec<Artists>> = stmt
        .query_map([], |row| stmt_to_artist(row))
        .unwrap()
        .collect();

    match result {
        Ok(artists) => artists,
        Err(e) => panic!("Error fetching all artists: {}", e),
    }
}

pub fn artist_by_name(name: &str) -> Option<Artists> {
    let conn = db_connect();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM artists WHERE name = ?1")
        .unwrap();
    let result: Result<Artists> = stmt.query_row([name], |row| stmt_to_artist(row));

    match result {
        Ok(artist) => Some(artist),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => panic!("Error fetching artist: {}", e),
    }
}

pub fn artist_by_id(id: &i32) -> Artists {
    let conn = db_connect();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM artists WHERE ID = ?1")
        .unwrap();
    let result = stmt.query_row([id], |row| stmt_to_artist(row));

    result.unwrap()
}

pub fn artist_with_albums(id: &i32) -> ArtistWithAlbums {
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

pub fn new_artist(artist: &str) -> i32 {
    let conn = db_connect();
    let stmt = conn.prepare_cached("INSERT INTO artists (name) VALUES (?1)");
    let result = stmt.unwrap().execute([artist]);

    match result {
        Ok(_) => conn.last_insert_rowid() as i32,
        Err(e) => panic!("Error inserting artist: {}", e),
    }
}

fn stmt_to_artist(row: &Row) -> Result<Artists, Error> {
    Ok(Artists {
        id: row.get(0)?,
        name: row.get(1)?,
    })
}
