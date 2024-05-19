use crate::db::db_connect;
use crate::models::Tracks;
use rusqlite::{Error, Result, Row};

pub fn all_tracks() -> Vec<Tracks> {
    let conn = db_connect();

    let mut stmt = conn.prepare("SELECT * FROM tracks").unwrap();
    let result = stmt
        .query_map([], |row| stmt_to_track(row))
        .unwrap()
        .collect();

    match result {
        Ok(tracks) => tracks,
        Err(e) => panic!("Error fetching all tracks: {}", e),
    }
}

pub fn get_all_tracks_path() -> Vec<String> {
    let conn = db_connect();

    let mut stmt = conn.prepare("SELECT path FROM tracks").unwrap();
    let result = stmt.query_map([], |row| row.get(0)).unwrap().collect();

    match result {
        Ok(paths) => paths,
        Err(e) => panic!("Error fetching all tracks path: {}", e),
    }
}

pub fn track_by_album_id(track_name: &str, album_id: &i32) -> Option<Tracks> {
    let conn = db_connect();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM tracks WHERE (name, albums_id) = (?1, ?2)")
        .unwrap();
    let result: Result<Tracks> = stmt.query_row((track_name, album_id), |row| stmt_to_track(row));

    match result {
        Ok(track) => Some(track),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => panic!("Error fetching track: {}", e),
    }
}

pub fn get_track_by_id(track_id: &i32) -> Tracks {
    let conn = db_connect();

    let mut stmt = conn.prepare("SELECT * FROM tracks WHERE id = ?1").unwrap();
    let result = stmt.query_row([track_id], |row| stmt_to_track(row));

    match result {
        Ok(track) => track,
        Err(e) => panic!("Error fetching track: {}", e),
    }
}

pub fn new_track(track: Tracks) -> i64 {
    let conn = db_connect();
    let stmt = conn.prepare_cached(
        "INSERT INTO tracks (duration, album, albums_id, artist, name, path, cover_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    );
    let result = stmt.unwrap().execute((
        track.duration,
        track.album,
        track.albums_id,
        track.artist,
        track.name,
        track.path,
        track.cover_path,
    ));

    match result {
        Ok(_) => conn.last_insert_rowid(),
        Err(e) => panic!("Error inserting track: {}", e),
    }
}

pub fn delete_track(track_path: &str) {
    let conn = db_connect();
    let mut stmt = conn.prepare("DELETE FROM tracks WHERE path = ?1").unwrap();
    let result = stmt.execute([track_path]);

    match result {
        Ok(_) => (),
        Err(e) => panic!("Error deleting track: {}", e),
    }
}

pub fn stmt_to_track(row: &Row) -> Result<Tracks, Error> {
    Ok(Tracks {
        id: row.get(0)?,
        album: row.get(1)?,
        albums_id: row.get(2)?,
        artist: row.get(3)?,
        name: row.get(4)?,
        duration: row.get(5)?,
        path: row.get(6)?,
        cover_path: row.get(7)?,
    })
}
