use super::track::stmt_to_track;
use crate::db::db_connect;
use crate::models::{PlaylistWithTracks, Playlists};
use rusqlite::{Error, Result, Row};

pub fn get_all_playlists() -> Vec<Playlists> {
    let conn = db_connect();
    let mut stmt = conn.prepare("SELECT * FROM playlists").unwrap();
    let playlists = stmt
        .query_map([], |row| stmt_to_playlist(row))
        .unwrap()
        .collect();

    match playlists {
        Ok(playlists) => playlists,
        Err(e) => panic!("Error fetching all playlists: {}", e),
    }
}

pub fn new_playlist(playlist: Playlists) -> i32 {
    let conn = db_connect();
    conn.execute(
        "INSERT INTO playlists (name, description, cover_path) VALUES (?1, ?2, ?3)",
        &[&playlist.name, &playlist.description, &playlist.cover_path],
    )
    .expect("Error inserting new playlist");

    conn.last_insert_rowid() as i32
}

fn get_playlist_by_id(id: &i32) -> Playlists {
    let conn = db_connect();
    let mut stmt = conn
        .prepare("SELECT * FROM playlists WHERE id = ?1")
        .unwrap();
    let playlist = stmt.query_row(&[id], |row| stmt_to_playlist(row));

    match playlist {
        Ok(playlist) => playlist,
        Err(e) => panic!("Error fetching playlist by id: {}", e),
    }
}

pub fn get_playlist_with_tracks(playlist_id: &i32) -> PlaylistWithTracks {
    let playlist = get_playlist_by_id(playlist_id);
    let conn = db_connect();
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.album, t.albums_id, t.artist, t.name, t.path
        FROM playlist_tracks pt
        JOIN tracks t ON pt.tracks_id = t.id
        WHERE pt.playlists_id = ?1;",
        )
        .unwrap();
    let tracks = stmt
        .query_map(&[playlist_id], |row| stmt_to_track(row))
        .unwrap()
        .collect();

    match tracks {
        Ok(tracks) => PlaylistWithTracks { playlist, tracks },
        Err(e) => panic!("Error fetching playlist with tracks: {}", e),
    }
}

pub fn update_playlist(playlist: Playlists) {
    let conn = db_connect();
    conn.execute(
        "UPDATE playlists SET name = ?1, description = ?2, cover_path = ?3 WHERE id = ?4",
        &[&playlist.name, &playlist.description, &playlist.cover_path],
    )
    .expect("Error updating playlist");
}

pub fn insert_track_to_playlist(playlist_id: &i32, track_id: &i32) {
    let conn = db_connect();
    conn.execute(
        "INSERT INTO playlist_tracks (playlists_id, tracks_id) VALUES (?1, ?2)",
        &[playlist_id, track_id],
    )
    .expect("Error inserting track to playlist");
}

pub fn delete_playlist(id: &i32) {
    let conn = db_connect();
    conn.execute("DELETE FROM playlists WHERE id = ?1", &[id])
        .expect("Error deleting playlist");
}

pub fn stmt_to_playlist(row: &Row) -> Result<Playlists, Error> {
    Ok(Playlists {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        cover_path: row.get(3)?,
    })
}
