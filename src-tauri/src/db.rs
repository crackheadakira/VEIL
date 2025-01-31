use std::{fs::create_dir, path::Path};

use crate::{commands::music_folder::get_album_type, models::*};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Error;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error(transparent)]
    R2D2Error(#[from] r2d2::Error),
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
}

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
            name        TEXT    NOT NULL
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

    pub fn shutdown(&mut self) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        // write WAL to disk
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;

        Ok(())
    }

    pub fn all<T: NeedForDatabase>(&self) -> Result<Vec<T>, DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = match T::table_name() {
            "artists" => "SELECT * FROM tracks",
            "albums" => "SELECT * FROM albums",
            "tracks" => "SELECT * FROM tracks",
            "playlists" => "SELECT * FROM playlists",
            _ => unreachable!("Invalid table name"),
        };

        let mut stmt = conn.prepare(stmt_to_call)?;
        let result = stmt
            .query_map([], T::from_row)?
            .collect::<Result<Vec<T>, Error>>()?;

        Ok(result)
    }

    pub fn by_id<T: NeedForDatabase>(&self, id: &u32) -> Result<T, DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = format!("SELECT * FROM {} WHERE id = ?1", T::table_name());
        let mut stmt = conn.prepare_cached(&stmt_to_call)?;

        let result = stmt.query_row([id], T::from_row)?;

        Ok(result)
    }

    pub fn insert<T: NeedForDatabase>(&self, data_to_pass: T) -> Result<u32, DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = match T::table_name() {
            "artists" => "INSERT INTO artists (name) VALUES (?1) RETURNING id",
            "albums" => "INSERT INTO albums (artists_id, artist, name, cover_path, type, duration, track_count, year, path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) RETURNING id",
            "tracks" => "INSERT INTO tracks (album, albums_id, artist, artists_id, name, duration, path, cover_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) RETURNING id",
            "playlists" => "INSERT INTO playlists (name, description, cover_path) VALUES (?1, ?2, ?3) RETURNING id",
            _ => unreachable!("Invalid table name"),
        };

        let mut stmt = conn.prepare_cached(stmt_to_call)?;
        let params = data_to_pass.to_params();
        let id = stmt.query_row(params.as_slice(), |row| row.get(0))?;

        Ok(id)
    }

    pub fn delete<T: NeedForDatabase>(&self, id: u32) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = format!("DELETE FROM {} WHERE id = ?1", T::table_name());
        let mut stmt = conn.prepare_cached(&stmt_to_call)?;
        stmt.execute([id])?;

        Ok(())
    }

    pub fn count<T: NeedForDatabase>(
        &self,
        id: u32,
        call_where: &str,
    ) -> Result<u32, DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = format!(
            "SELECT COUNT(*) FROM {} WHERE {} = ?1",
            T::table_name(),
            call_where
        );
        let mut stmt = conn.prepare_cached(&stmt_to_call)?;
        let result = stmt.query_row([id], |row| row.get(0))?;

        Ok(result)
    }

    pub fn exists<T: NeedForDatabase>(
        &self,
        field_to_view: &str,
        field_data: &str,
    ) -> Result<bool, DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = format!(
            "SELECT 1 FROM {} WHERE {} = ?1",
            T::table_name(),
            field_to_view
        );
        let mut stmt = conn.prepare(&stmt_to_call)?;
        let result = stmt.exists([field_data])?;

        Ok(result)
    }

    pub fn update_duration(
        &self,
        track_id: &u32,
        album_id: &u32,
        duration: &u32,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("UPDATE tracks SET duration = ?1 WHERE id = ?2")?;
        stmt.execute((duration, track_id))?;

        let (new_duration, track_count) = self.get_album_duration(album_id)?;
        let new_album_type = get_album_type(track_count, new_duration);
        self.update_album_type(album_id, &new_album_type, &(new_duration, track_count))?;

        Ok(())
    }

    // TRACK
    pub fn track_by_album_id(
        &self,
        track_name: &str,
        album_id: &u32,
    ) -> Result<Tracks, DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare_cached("SELECT * FROM tracks WHERE (name, albums_id) = (?1, ?2)")?;
        let result = stmt.query_row((track_name, album_id), Tracks::from_row)?;

        Ok(result)
    }

    // ALBUM
    pub fn album_by_name(
        &self,
        album_name: &str,
        artist_id: &u32,
    ) -> Result<Albums, DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare_cached("SELECT * FROM albums WHERE (name, artists_id) = (?1, ?2)")?;
        let result = stmt.query_row((album_name, artist_id), Albums::from_row)?;

        Ok(result)
    }

    pub fn albums_by_artist_id(&self, artist_id: &u32) -> Result<Vec<Albums>, DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM albums WHERE artists_id = ?1")?;
        let result = stmt
            .query_map([artist_id], Albums::from_row)?
            .collect::<Result<Vec<Albums>, Error>>()?;

        Ok(result)
    }

    pub fn get_album_duration(&self, album_id: &u32) -> Result<(u32, u32), DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT SUM(duration), COUNT(*) FROM tracks WHERE albums_id = ?1")?;

        let result = stmt.query_row([album_id], |row| Ok((row.get(0)?, row.get(1)?)))?;

        Ok(result)
    }

    pub fn album_with_tracks(&self, album_id: &u32) -> Result<AlbumWithTracks, DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM tracks WHERE albums_id = ?1")?;

        let tracks = stmt
            .query_map([album_id], Tracks::from_row)?
            .collect::<Result<Vec<Tracks>, Error>>()?;

        let album = self.by_id::<Albums>(album_id)?;

        Ok(AlbumWithTracks { album, tracks })
    }

    pub fn update_album_type(
        &self,
        album_id: &u32,
        album_type: &str,
        duration_count: &(u32, u32),
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "UPDATE albums SET type = ?1, duration = ?2, track_count = ?3 WHERE ID = ?4",
        )?;
        stmt.execute((album_type, duration_count.0, duration_count.1, album_id))?;

        Ok(())
    }

    // ARTIST

    pub fn artist_by_name(&self, name: &str) -> Result<Artists, DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached("SELECT * FROM artists WHERE name = ?1")?;
        let result = stmt.query_row([name], Artists::from_row)?;

        Ok(result)
    }

    pub fn artist_with_albums(&self, id: &u32) -> Result<ArtistWithAlbums, DatabaseError> {
        let artist = self.by_id::<Artists>(id)?;
        let albums = self.albums_by_artist_id(id)?;

        let albums_with_tracks = albums
            .iter()
            .map(|album| self.album_with_tracks(&album.id))
            .collect::<Result<Vec<AlbumWithTracks>, DatabaseError>>()?;

        Ok(ArtistWithAlbums {
            artist,
            albums: albums_with_tracks,
        })
    }

    // PLAYLIST

    pub fn get_playlist_with_tracks(
        &self,
        playlist_id: &u32,
    ) -> Result<PlaylistWithTracks, DatabaseError> {
        let conn = self.pool.get()?;
        let playlist = self.by_id::<Playlists>(playlist_id)?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.album, t.albums_id, t.artist, t.name, t.path
            FROM playlist_tracks pt
            JOIN tracks t ON pt.tracks_id = t.id
            WHERE pt.playlists_id = ?1;",
        )?;

        let tracks = stmt
            .query_map([playlist_id], Tracks::from_row)?
            .collect::<Result<Vec<Tracks>, Error>>()?;

        Ok(PlaylistWithTracks { playlist, tracks })
    }

    pub fn update_playlist(&self, playlist: &Playlists) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "UPDATE playlists SET name = ?1, description = ?2, cover_path = ?3 WHERE id = ?4",
        )?;
        stmt.execute(playlist.to_params().as_slice())?;

        Ok(())
    }

    pub fn insert_track_to_playlist(
        &self,
        playlist_id: &u32,
        track_id: &u32,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO playlist_tracks (playlists_id, tracks_id) VALUES (?1, ?2)",
            [playlist_id, track_id],
        )?;

        Ok(())
    }
}

fn get_db_path() -> String {
    data_path() + "/db.sqlite"
}

pub fn data_path() -> String {
    let home_dir = dirs::data_local_dir().unwrap();
    home_dir.to_str().unwrap().to_string() + "/Sodapop-Reimagined"
}
