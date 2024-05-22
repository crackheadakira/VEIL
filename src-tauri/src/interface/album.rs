use rusqlite::{Error, Result, Row};

use super::track::stmt_to_track;
use crate::db::db_connect;
use crate::models::{AlbumWithTracks, Albums, Tracks};

pub fn all_albums() -> Vec<Albums> {
    let conn = db_connect();

    let mut stmt = conn.prepare("SELECT * FROM albums").unwrap();
    let result = stmt.query_map([], stmt_to_album).unwrap().collect();

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
    let result = stmt.query_row((album_name, artist_id), stmt_to_album);

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
        .query_map([artist_id], stmt_to_album)
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
    let result = stmt.query_row([album_id], stmt_to_album);

    result.unwrap()
}

pub fn get_album_duration(album_id: &i32) -> (i32, i32) {
    let conn = db_connect();

    let mut stmt = conn
        .prepare("SELECT SUM(duration), COUNT(*) FROM tracks WHERE albums_id = ?1")
        .unwrap();

    let result = stmt.query_row([album_id], |row| Ok((row.get(0)?, row.get(1)?)));

    match result {
        Ok((duration, tracks)) => (duration, tracks),
        Err(e) => panic!("Error fetching album duration: {}", e),
    }
}

pub fn album_by_path(album_path: &str) -> Albums {
    let conn = db_connect();

    let mut stmt = conn
        .prepare("SELECT * FROM albums WHERE path = ?1")
        .unwrap();
    let result = stmt.query_row([album_path], stmt_to_album);

    result.unwrap()
}

pub fn album_with_tracks(album_id: &i32) -> AlbumWithTracks {
    let conn = db_connect();

    let mut stmt = conn
        .prepare("SELECT * FROM tracks WHERE albums_id = ?1")
        .unwrap();
    let tracks = stmt
        .query_map([album_id], stmt_to_track)
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
        "INSERT INTO albums (artists_id, artist, name, cover_path, type, duration, track_count, year, path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    );
    let result = stmt.unwrap().execute((
        album.artists_id,
        album.artist,
        album.name,
        album.cover_path,
        album.album_type,
        album.duration,
        album.track_count,
        album.year,
        album.path,
    ));

    match result {
        Ok(_) => conn.last_insert_rowid() as i32,
        Err(e) => panic!("Error inserting album: {}", e),
    }
}

pub fn update_album_type(album_id: &i32, album_type: &str, duration_count: &(i32, i32)) {
    let conn = db_connect();
    let mut stmt = conn
        .prepare("UPDATE albums SET type = ?1, duration = ?2, track_count = ?3 WHERE ID = ?4")
        .unwrap();
    let result = stmt.execute((album_type, duration_count.0, duration_count.1, album_id));

    match result {
        Ok(_) => (),
        Err(e) => panic!("Error updating album type: {}", e),
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
        artist: row.get(2)?,
        name: row.get(3)?,
        cover_path: row.get(4)?,
        album_type: row.get(5)?,
        duration: row.get(6)?,
        track_count: row.get(7)?,
        year: row.get(8)?,
        path: row.get(9)?,
    })
}
