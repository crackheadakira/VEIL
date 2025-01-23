use std::{fs::create_dir, path::Path};

use crate::{get_album_type, models::*};
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

    pub fn all<T>(&self) -> Vec<T>
    where
        T: NeedForDatabase,
    {
        let conn = self.pool.get().unwrap();
        let stmt_to_call = match T::table_name() {
            "artists" => "SELECT * FROM tracks",
            "albums" => "SELECT * FROM albums",
            "tracks" => "SELECT * FROM tracks",
            "playlists" => "SELECT * FROM playlists",
            _ => panic!("Invalid table name"),
        };

        let mut stmt = conn.prepare(stmt_to_call).unwrap();
        let result = stmt.query_map([], T::from_row).unwrap().collect();

        match result {
            Ok(items) => items,
            Err(e) => panic!("Error fetching all {}: {}", T::table_name(), e),
        }
    }

    pub fn by_id<T>(&self, id: &u32) -> T
    where
        T: NeedForDatabase,
    {
        let conn = self.pool.get().unwrap();
        let stmt_to_call = format!("SELECT * FROM {} WHERE id = ?1", T::table_name());
        let mut stmt = conn.prepare_cached(&stmt_to_call).unwrap();

        let result = stmt.query_row([id], T::from_row);

        match result {
            Ok(item) => item,
            Err(e) => panic!("Error fetching {} by id: {}", T::table_name(), e),
        }
    }

    pub fn insert<T>(&self, data_to_pass: T) -> u32
    where
        T: NeedForDatabase,
    {
        let conn = self.pool.get().unwrap();
        let stmt_to_call = match T::table_name() {
            "artists" => "INSERT INTO artists (name, path) VALUES (?1, ?2)",
            "albums" => "INSERT INTO albums (artists_id, artist, name, cover_path, type, duration, track_count, year, path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            "tracks" => "INSERT INTO tracks (album, albums_id, artist, artists_id, name, duration, path, cover_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            "playlists" => "INSERT INTO playlists (name, description, cover_path) VALUES (?1, ?2, ?3)",
            _ => panic!("Invalid table name"),
        };

        let stmt = conn.prepare_cached(stmt_to_call);
        let params = data_to_pass.to_params();
        let result = stmt.unwrap().execute(&params[..]);

        match result {
            Ok(_) => conn.last_insert_rowid() as u32,
            Err(e) => panic!("Error inserting {}: {}", T::table_name(), e),
        }
    }

    pub fn delete<T>(&self, id: u32)
    where
        T: NeedForDatabase,
    {
        let conn = self.pool.get().unwrap();
        let stmt_to_call = format!("DELETE FROM {} WHERE id = ?1", T::table_name());
        let stmt = conn.prepare_cached(&stmt_to_call);
        let result = stmt.unwrap().execute([id]);

        match result {
            Ok(_) => (),
            Err(e) => panic!("Error deleting {}: {}", T::table_name(), e),
        }
    }

    pub fn count<T>(&self, id: u32, call_where: &str) -> u32
    where
        T: NeedForDatabase,
    {
        let conn = self.pool.get().unwrap();
        let stmt_to_call = format!(
            "SELECT COUNT(*) FROM {} WHERE {} = ?1",
            T::table_name(),
            call_where
        );
        let mut stmt = conn.prepare_cached(&stmt_to_call).unwrap();
        let result = stmt.query_row([id], |row| row.get(0));

        match result {
            Ok(count) => count,
            Err(e) => panic!("Error counting {}: {}", T::table_name(), e),
        }
    }

    pub fn update_duration(&self, track_id: &u32, album_id: &u32, duration: &u32) {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("UPDATE tracks SET duration = ?1 WHERE id = ?2")
            .unwrap();
        let result = stmt.execute((duration, track_id));

        match result {
            Ok(_) => {
                let mut album = self.by_id::<Albums>(album_id);
                album.duration += duration;
                let new_album_type = get_album_type(album.track_count, album.duration);
                self.update_album_type(
                    album_id,
                    &new_album_type,
                    &(album.duration, album.track_count),
                );
            }
            Err(e) => panic!("Error updating duration: {}", e),
        }
    }

    // TRACK
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

    // ALBUM

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

        let album = self.by_id::<Albums>(album_id);

        AlbumWithTracks { album, tracks }
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

    // ARTIST

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

    pub fn artist_with_albums(&self, id: &u32) -> ArtistWithAlbums {
        let artist = self.by_id::<Artists>(id);
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

    // PLAYLIST

    pub fn get_playlist_with_tracks(&self, playlist_id: &u32) -> PlaylistWithTracks {
        let conn = self.pool.get().unwrap();
        let playlist = self.by_id::<Playlists>(playlist_id);
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

    pub fn update_playlist(&self, playlist: &Playlists) -> Result<usize> {
        let conn = self.pool.get().unwrap();
        conn.execute(
            "UPDATE playlists SET name = ?1, description = ?2, cover_path = ?3 WHERE id = ?4",
            [&playlist.name, &playlist.description, &playlist.cover_path],
        )
    }

    pub fn insert_track_to_playlist(&self, playlist_id: &u32, track_id: &u32) -> Result<usize> {
        let conn = self.pool.get().unwrap();
        conn.execute(
            "INSERT INTO playlist_tracks (playlists_id, tracks_id) VALUES (?1, ?2)",
            [playlist_id, track_id],
        )
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
