use std::{fs::create_dir, path::Path};

use crate::models::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Error, Result, Row};

pub struct Database {
    pub pool: Pool<SqliteConnectionManager>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Self {
        let data_path = data_path();
        if !Path::new(&data_path).exists() {
            create_dir(&data_path).expect("Error creating data directory");
        }

        let manager = SqliteConnectionManager::file(get_db_path());
        let pool = Pool::new(manager).unwrap();
        let conn = pool.get().unwrap();

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
            PRAGMA journal_size_limit = 6144000;
            PRAGMA synchronous = NORMAL;",
        )
        .expect("Error setting PRAGMA");

        conn.execute_batch(
            "
        BEGIN;
        CREATE TABLE IF NOT EXISTS artists (
            id          INTEGER NOT NULL PRIMARY KEY,
            name        TEXT    NOT NULL,
            path        TEXT    NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS albums (
            id          INTEGER NOT NULL PRIMARY KEY,
            artists_id  INTEGER NOT NULL REFERENCES artists(id),
            artist      TEXT    NOT NULL,
            name        TEXT    NOT NULL,
            cover_path  TEXT    NOT NULL,
            type        TEXT    NOT NULL,
            duration    INTEGER NOT NULL,
            track_count INTEGER NOT NULL,
            year        INTEGER NOT NULL,
            path        TEXT    NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS tracks (
            id          INTEGER NOT NULL PRIMARY KEY,
            album       TEXT    NOT NULL,
            albums_id   INTEGER NOT NULL REFERENCES albums(id),
            artist      TEXT    NOT NULL,
            artists_id  INTEGER NOT NULL REFERENCES artists(id),
            name        TEXT    NOT NULL,
            duration    INTEGER NOT NULL,
            path        TEXT    NOT NULL UNIQUE,
            cover_path  TEXT    NOT NULL
        ); 
        CREATE TABLE IF NOT EXISTS playlists (
            id          INTEGER NOT NULL PRIMARY KEY,
            name        TEXT    NOT NULL,
            description TEXT    NOT NULL,
            cover_path  TEXT    NOT NULL
        );
        CREATE TABLE IF NOT EXISTS playlist_tracks (
            id          INTEGER NOT NULL PRIMARY KEY,
            playlists_id INTEGER NOT NULL REFERENCES playlists(id),
            tracks_id   INTEGER NOT NULL REFERENCES tracks(id)
        );
        COMMIT;
        ",
        )
        .expect("Error creating tables");

        drop(conn);

        Self { pool }
    }

    // TRACK

    pub fn all_tracks(&self) -> Vec<Tracks> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM tracks").unwrap();
        let result = stmt.query_map([], stmt_to_track).unwrap().collect();

        match result {
            Ok(tracks) => tracks,
            Err(e) => panic!("Error fetching all tracks: {}", e),
        }
    }

    pub fn get_all_tracks_path(&self) -> Vec<String> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare("SELECT path FROM tracks").unwrap();
        let result = stmt.query_map([], |row| row.get(0)).unwrap().collect();

        match result {
            Ok(paths) => paths,
            Err(e) => panic!("Error fetching all tracks path: {}", e),
        }
    }

    pub fn track_by_album_id(&self, track_name: &str, album_id: &u32) -> Option<Tracks> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare_cached("SELECT * FROM tracks WHERE (name, albums_id) = (?1, ?2)")
            .unwrap();

        match stmt.query_row((track_name, album_id), stmt_to_track) {
            Ok(track) => Some(track),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => panic!("Error fetching track: {}", e),
        }
    }

    pub fn get_track_by_id(&self, track_id: &u32) -> Tracks {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM tracks WHERE id = ?1").unwrap();
        let result = stmt.query_row([track_id], stmt_to_track);

        match result {
            Ok(track) => track,
            Err(e) => panic!("Error fetching track: {}", e),
        }
    }

    pub fn new_track(&self, track: Tracks) -> u32 {
        let conn = self.pool.get().unwrap();
        let stmt = conn.prepare_cached(
            "INSERT INTO tracks (duration, album, albums_id, artist, artists_id, name, path, cover_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        );
        let result = stmt.unwrap().execute((
            track.duration,
            track.album,
            track.albums_id,
            track.artist,
            track.artists_id,
            track.name,
            track.path,
            track.cover_path,
        ));

        match result {
            Ok(_) => conn.last_insert_rowid() as u32,
            Err(e) => panic!("Error inserting track: {}", e),
        }
    }

    pub fn delete_track(&self, track_path: &str) {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare("DELETE FROM tracks WHERE path = ?1").unwrap();
        let result = stmt.execute([track_path]);

        match result {
            Ok(_) => (),
            Err(e) => panic!("Error deleting track: {}", e),
        }
    }

    // ALBUM

    pub fn all_albums(&self) -> Vec<Albums> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM albums").unwrap();
        let result = stmt.query_map([], stmt_to_album).unwrap().collect();

        match result {
            Ok(albums) => albums,
            Err(e) => panic!("Error fetching all albums: {}", e),
        }
    }

    // This returns an option due to it's usage in metadata.rs
    pub fn spec_album_by_artist_id(&self, album_name: &str, artist_id: &u32) -> Option<Albums> {
        let conn = self.pool.get().unwrap();
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

    pub fn album_by_artist_id(&self, artist_id: &u32) -> Vec<Albums> {
        let conn = self.pool.get().unwrap();
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

    pub fn album_by_id(&self, album_id: &u32) -> Albums {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare_cached("SELECT * FROM albums WHERE ID = ?1")
            .unwrap();
        let result = stmt.query_row([album_id], stmt_to_album);

        result.unwrap()
    }

    pub fn get_album_duration(&self, album_id: &u32) -> (u32, u32) {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("SELECT SUM(duration), COUNT(*) FROM tracks WHERE albums_id = ?1")
            .unwrap();

        let result = stmt.query_row([album_id], |row| Ok((row.get(0)?, row.get(1)?)));

        match result {
            Ok((duration, tracks)) => (duration, tracks),
            Err(e) => panic!("Error fetching album duration: {}", e),
        }
    }

    pub fn album_by_path(&self, album_path: &str) -> Albums {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("SELECT * FROM albums WHERE path = ?1")
            .unwrap();
        let result = stmt.query_row([album_path], stmt_to_album);

        result.unwrap()
    }

    pub fn album_with_tracks(&self, album_id: &u32) -> AlbumWithTracks {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("SELECT * FROM tracks WHERE albums_id = ?1")
            .unwrap();
        let tracks = stmt
            .query_map([album_id], stmt_to_track)
            .unwrap()
            .collect::<Result<Vec<Tracks>>>()
            .unwrap();

        let album = self.album_by_id(album_id);

        AlbumWithTracks { album, tracks }
    }

    pub fn album_tracks_length(&self, album_id: &u32) -> u32 {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM tracks WHERE albums_id = ?1")
            .unwrap();
        let result = stmt.query_row([album_id], |row| row.get(0));

        match result {
            Ok(length) => length,
            Err(e) => panic!("Error fetching album tracks length: {}", e),
        }
    }

    pub fn new_album(&self, album: Albums) -> u32 {
        let conn = self.pool.get().unwrap();
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
            Ok(_) => conn.last_insert_rowid() as u32,
            Err(e) => panic!("Error inserting album: {}", e),
        }
    }

    pub fn update_album_type(&self, album_id: &u32, album_type: &str, duration_count: &(u32, u32)) {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("UPDATE albums SET type = ?1, duration = ?2, track_count = ?3 WHERE ID = ?4")
            .unwrap();
        let result = stmt.execute((album_type, duration_count.0, duration_count.1, album_id));

        match result {
            Ok(_) => (),
            Err(e) => panic!("Error updating album type: {}", e),
        }
    }

    pub fn delete_album(&self, album_id: u32) {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("DELETE FROM albums WHERE ID = ?1 OR path = ?1")
            .unwrap();
        let result = stmt.execute([album_id]);

        match result {
            Ok(_) => (),
            Err(e) => panic!("Error deleting artist: {}", e),
        }
    }

    // ARTIST

    pub fn all_artists(&self) -> Vec<Artists> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM artists").unwrap();
        let result: Result<Vec<Artists>> = stmt.query_map([], stmt_to_artist).unwrap().collect();

        match result {
            Ok(artists) => artists,
            Err(e) => panic!("Error fetching all artists: {}", e),
        }
    }

    pub fn artist_albums_length(&self, artist_id: &u32) -> u32 {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM albums WHERE artists_id = ?1")
            .unwrap();
        let result = stmt.query_row([artist_id], |row| row.get(0));

        match result {
            Ok(count) => count,
            Err(e) => panic!("Error fetching artist albums length: {}", e),
        }
    }

    pub fn artist_by_name(&self, name: &str) -> Option<Artists> {
        let conn = self.pool.get().unwrap();
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

    pub fn artist_by_id(&self, id: &u32) -> Artists {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare_cached("SELECT * FROM artists WHERE ID = ?1")
            .unwrap();
        let result = stmt.query_row([id], stmt_to_artist);

        result.unwrap()
    }

    pub fn artist_with_albums(&self, id: &u32) -> ArtistWithAlbums {
        let artist = self.artist_by_id(id);
        let albums = self.album_by_artist_id(id);

        let albums_with_tracks = albums
            .iter()
            .map(|album| self.album_with_tracks(&album.id))
            .collect();

        ArtistWithAlbums {
            artist,
            albums: albums_with_tracks,
        }
    }

    pub fn new_artist(&self, artist: &str, path: &str) -> u32 {
        let conn = self.pool.get().unwrap();
        let stmt = conn.prepare_cached("INSERT INTO artists (name, path) VALUES (?1, ?2)");
        let result = stmt.unwrap().execute((artist, path));

        match result {
            Ok(_) => conn.last_insert_rowid() as u32,
            Err(e) => panic!("Error inserting artist: {}", e),
        }
    }

    pub fn delete_artist(&self, artist_id: u32) {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("DELETE FROM artists WHERE ID = ?1 OR name = ?1 OR path = ?1")
            .unwrap();
        let result = stmt.execute([artist_id]);

        match result {
            Ok(_) => (),
            Err(e) => panic!("Error deleting artist: {}", e),
        }
    }

    // PLAYLIST

    pub fn get_all_playlists(&self) -> Vec<Playlists> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM playlists").unwrap();
        let playlists = stmt.query_map([], stmt_to_playlist).unwrap().collect();

        match playlists {
            Ok(playlists) => playlists,
            Err(e) => panic!("Error fetching all playlists: {}", e),
        }
    }

    pub fn new_playlist(&self, playlist: Playlists) -> u32 {
        let conn = self.pool.get().unwrap();
        conn.execute(
            "INSERT INTO playlists (name, description, cover_path) VALUES (?1, ?2, ?3)",
            [&playlist.name, &playlist.description, &playlist.cover_path],
        )
        .expect("Error inserting new playlist");

        conn.last_insert_rowid() as u32
    }

    fn get_playlist_by_id(&self, id: &u32) -> Playlists {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("SELECT * FROM playlists WHERE id = ?1")
            .unwrap();
        let playlist = stmt.query_row([id], stmt_to_playlist);

        match playlist {
            Ok(playlist) => playlist,
            Err(e) => panic!("Error fetching playlist by id: {}", e),
        }
    }

    pub fn get_playlist_with_tracks(&self, playlist_id: &u32) -> PlaylistWithTracks {
        let conn = self.pool.get().unwrap();
        let playlist = self.get_playlist_by_id(playlist_id);
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.album, t.albums_id, t.artist, t.name, t.path
            FROM playlist_tracks pt
            JOIN tracks t ON pt.tracks_id = t.id
            WHERE pt.playlists_id = ?1;",
            )
            .unwrap();
        let tracks = stmt
            .query_map([playlist_id], stmt_to_track)
            .unwrap()
            .collect();

        match tracks {
            Ok(tracks) => PlaylistWithTracks { playlist, tracks },
            Err(e) => panic!("Error fetching playlist with tracks: {}", e),
        }
    }

    pub fn update_playlist(&self, playlist: &Playlists) {
        let conn = self.pool.get().unwrap();
        conn.execute(
            "UPDATE playlists SET name = ?1, description = ?2, cover_path = ?3 WHERE id = ?4",
            [&playlist.name, &playlist.description, &playlist.cover_path],
        )
        .expect("Error updating playlist");
    }

    pub fn insert_track_to_playlist(&self, playlist_id: &u32, track_id: &u32) {
        let conn = self.pool.get().unwrap();
        conn.execute(
            "INSERT INTO playlist_tracks (playlists_id, tracks_id) VALUES (?1, ?2)",
            [playlist_id, track_id],
        )
        .expect("Error inserting track to playlist");
    }

    pub fn delete_playlist(&self, id: &u32) {
        let conn = self.pool.get().unwrap();
        conn.execute("DELETE FROM playlists WHERE id = ?1", [id])
            .expect("Error deleting playlist");
    }
}

fn get_db_path() -> String {
    data_path() + "/db.sqlite"
}

pub fn data_path() -> String {
    let home_dir = dirs::data_local_dir().unwrap();
    home_dir.to_str().unwrap().to_string() + "/Sodapop-Reimagined"
}

fn stmt_to_track(row: &Row) -> Result<Tracks, Error> {
    Ok(Tracks {
        id: row.get(0)?,
        album: row.get(1)?,
        albums_id: row.get(2)?,
        artist: row.get(3)?,
        artists_id: row.get(4)?,
        name: row.get(5)?,
        duration: row.get(6)?,
        path: row.get(7)?,
        cover_path: row.get(8)?,
    })
}

fn stmt_to_album(row: &Row) -> Result<Albums, Error> {
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

fn stmt_to_artist(row: &Row) -> Result<Artists, Error> {
    Ok(Artists {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
    })
}

fn stmt_to_playlist(row: &Row) -> Result<Playlists, Error> {
    Ok(Playlists {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        cover_path: row.get(3)?,
    })
}
