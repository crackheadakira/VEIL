use rusqlite::{Error, Result, Row};

use super::track::stmt_to_track;
use crate::db::db_connect;
use crate::models::{AlbumWithTracks, Albums, Tracks};

pub fn all_albums() -> Vec<Albums> {
    let conn = db_connect();

    let mut stmt = conn.prepare("SELECT * FROM albums").unwrap();
    let result = stmt
        .query_map([], |row| stmt_to_album(row))
        .unwrap()
        .collect();

    match result {
        Ok(albums) => albums,
        Err(e) => panic!("Error fetching all albums: {}", e),
    }
}

/// This returns an option due to it's usage in metadata.rs
pub fn spec_album_by_artist_id(album_name: &str, artist_id: &i32) -> Option<Albums> {
    let conn = db_connect();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM albums WHERE (name, artists_id) = (?1, ?2)")
        .unwrap();
    let result = stmt.query_row((album_name, artist_id), |row| stmt_to_album(row));

    match result {
        Ok(album) => Some(album),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => panic!("Error fetching album: {}", e),
    }
}

pub fn album_by_artist_id(artist_id: &i32) -> Vec<Albums> {
    let conn = db_connect();

    let mut stmt = conn
        .prepare("SELECT * FROM albums WHERE artists_id = ?1")
        .unwrap();
    let result = stmt
        .query_map([artist_id], |row| stmt_to_album(row))
        .unwrap()
        .collect::<Result<Vec<Albums>>>()
        .unwrap();

    result
}

pub fn album_by_id(album_id: &i32) -> Albums {
    let conn = db_connect();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM albums WHERE ID = ?1")
        .unwrap();
    let result = stmt.query_row([album_id], |row| stmt_to_album(row));

    result.unwrap()
}

pub fn album_by_path(album_path: &str) -> Albums {
    let conn = db_connect();

    let mut stmt = conn
        .prepare("SELECT * FROM albums WHERE path = ?1")
        .unwrap();
    let result = stmt.query_row([album_path], |row| stmt_to_album(row));

    result.unwrap()
}

pub fn album_with_tracks(album_id: &i32) -> AlbumWithTracks {
    let conn = db_connect();

    let mut stmt = conn
        .prepare("SELECT * FROM tracks WHERE albums_id = ?1")
        .unwrap();
    let tracks = stmt
        .query_map([album_id], |row| stmt_to_track(row))
        .unwrap()
        .collect::<Result<Vec<Tracks>>>()
        .unwrap();

    let album = album_by_id(album_id);

    AlbumWithTracks { album, tracks }
}

pub fn album_tracks_length(album_id: &i32) -> i32 {
    let conn = db_connect();

    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM tracks WHERE albums_id = ?1")
        .unwrap();
    let result = stmt.query_row([album_id], |row| row.get(0));

    match result {
        Ok(length) => length,
        Err(e) => panic!("Error fetching album tracks length: {}", e),
    }
}

pub fn new_album(album: Albums) -> i32 {
    let conn = db_connect();
    let stmt = conn.prepare_cached(
        "INSERT INTO albums (artists_id, name, cover_path, type, year, path) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    );
    let result = stmt.unwrap().execute((
        album.artists_id,
        album.name,
        album.cover_path,
        album.album_type,
        album.year,
        album.path,
    ));

    match result {
        Ok(_) => conn.last_insert_rowid() as i32,
        Err(e) => panic!("Error inserting album: {}", e),
    }
}

pub fn delete_album(album_id: &i32) {
    let conn = db_connect();
    let mut stmt = conn
        .prepare("DELETE FROM albums WHERE ID = ?1 OR path = ?1")
        .unwrap();
    let result = stmt.execute([album_id]);

    match result {
        Ok(_) => (),
        Err(e) => panic!("Error deleting artist: {}", e),
    }
}

pub fn stmt_to_album(row: &Row) -> Result<Albums, Error> {
    Ok(Albums {
        id: row.get(0)?,
        artists_id: row.get(1)?,
        name: row.get(2)?,
        cover_path: row.get(3)?,
        album_type: row.get(4)?,
        year: row.get(5)?,
        path: row.get(6)?,
    })
}
