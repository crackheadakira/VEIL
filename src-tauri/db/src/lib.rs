pub mod models;

use include_dir::{include_dir, Dir};
use models::*;
use once_cell::sync::Lazy;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Error, OptionalExtension};
use std::{collections::HashMap, fs::create_dir, path::PathBuf};

fn collect_sql_files(dir: &Dir, queries: &mut HashMap<String, String>) {
    for file in dir.files() {
        if file.path().extension().map(|e| e == "sql").unwrap_or(false) {
            if let Some(content) = file.contents_utf8() {
                let file_name = file
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                queries.insert(file_name, content.to_string());
            }
        }
    }

    // Recursively check for subdirectories
    for subdir in dir.dirs() {
        collect_sql_files(subdir, queries);
    }
}

static QUERY_DIR: Dir = include_dir!("queries");

static QUERIES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut queries = HashMap::new();

    collect_sql_files(&QUERY_DIR, &mut queries);

    queries
});

fn get_query(name: &str) -> &str {
    QUERIES
        .get(name)
        .unwrap_or_else(|| panic!("Query '{}' not found", name))
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error(transparent)]
    R2D2Error(#[from] r2d2::Error),
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
}

pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Instanties a connection with the database, creates a new sqlite file if it doesn't exist
    pub fn new(path: PathBuf) -> Self {
        if !path.exists() {
            create_dir(&path).expect("Error creating data directory");
        }

        let manager = SqliteConnectionManager::file(path.join("db.sqlite"));
        let pool = Pool::new(manager).unwrap();
        let conn = pool.get().unwrap();

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
            PRAGMA journal_size_limit = 6144000;
            PRAGMA synchronous = NORMAL;",
        )
        .expect("Error setting PRAGMA");

        conn.execute_batch(get_query("schema"))
            .expect("Error creating tables");

        drop(conn);

        Self { pool }
    }

    /// Writes WAL data to database
    pub fn shutdown(&mut self) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;

        Ok(())
    }

    /// Get all values from table for `T`
    pub fn all<T: NeedForDatabase>(&self) -> Result<Vec<T>, DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = match T::table_name() {
            "artists" => get_query("artists_all"),
            "albums" => get_query("albums_all"),
            "tracks" => get_query("tracks_all"),
            "playlists" => get_query("playlists_all"),
            _ => unreachable!("Invalid table name"),
        };

        let mut stmt = conn.prepare(stmt_to_call)?;
        let result = stmt
            .query_map([], T::from_row)?
            .collect::<Result<Vec<T>, Error>>()?;

        Ok(result)
    }

    /// Get value from `T` table where id is same
    pub fn by_id<T: NeedForDatabase>(&self, id: &u32) -> Result<T, DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = match T::table_name() {
            "artists" => get_query("artists_id"),
            "albums" => get_query("albums_id"),
            "tracks" => get_query("tracks_id"),
            "playlists" => get_query("playlists_id"),
            _ => unreachable!("Invalid table name"),
        };
        let mut stmt = conn.prepare_cached(stmt_to_call)?;

        let result = stmt.query_row([id], T::from_row)?;

        Ok(result)
    }

    /// Insert value to `T` table
    pub fn insert<T: NeedForDatabase>(&self, data_to_pass: T) -> Result<(), DatabaseError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let stmt_to_call = match T::table_name() {
            "artists" => get_query("artists_insert"),
            "albums" => get_query("albums_insert"),
            "tracks" => get_query("tracks_insert"),
            "playlists" => get_query("playlists_insert"),
            _ => unreachable!("Invalid table name"),
        };

        let album_id = {
            let mut stmt = tx.prepare_cached(stmt_to_call)?;
            let params = data_to_pass.to_params();
            stmt.execute(rusqlite::params_from_iter(params))?;

            tx.last_insert_rowid()
        };

        if T::table_name() == "albums" {
            let mut stmt = tx.prepare_cached(get_query("album_artists_insert"))?;
            stmt.execute((album_id, data_to_pass.get_artist_id()))?;
        }

        tx.commit()?;

        Ok(())
    }

    /// Delete value from `T` table where id is same
    pub fn delete<T: NeedForDatabase>(&self, id: u32) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = format!("DELETE FROM {} WHERE id = ?1", T::table_name());
        let mut stmt = conn.prepare_cached(&stmt_to_call)?;
        stmt.execute([id])?;

        Ok(())
    }

    /// Return latest record from `T` table
    pub fn latest<T: NeedForDatabase>(&self) -> Result<T, DatabaseError> {
        let conn = self.pool.get()?;
        let stmt_to_call = match T::table_name() {
            "artists" => get_query("artists_latest"),
            "albums" => get_query("albums_latest"),
            "tracks" => get_query("tracks_latest"),
            "playlists" => get_query("playlists_latest"),
            _ => unreachable!("Invalid table name"),
        };

        let mut stmt = conn.prepare_cached(stmt_to_call)?;
        let result = stmt.query_row([], T::from_row)?;

        Ok(result)
    }

    /// Counts how many values `T` table has
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

    /// Checks if value exists in `T` table
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

    /// Update track duration
    pub fn update_duration(
        &self,
        track_id: &u32,
        album_id: &u32,
        duration: &u32,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(get_query("tracks_update_duration"))?;
        stmt.execute((duration, track_id))?;

        let (new_duration, track_count) = self.get_album_duration(album_id)?;
        let new_album_type = AlbumType::get(track_count, new_duration);
        self.update_album_type(album_id, new_album_type, &(new_duration, track_count))?;

        Ok(())
    }

    /// Get album from database where `album_name` matches
    pub fn album_by_name(
        &self,
        album_name: &str,
        artist_id: &u32,
    ) -> Result<Albums, DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(get_query("album_name"))?;
        let result = stmt.query_row((artist_id, album_name), Albums::from_row)?;

        Ok(result)
    }

    /// Get all albums from artist where `artist_id` matches
    pub fn albums_by_artist_id(&self, artist_id: &u32) -> Result<Vec<Albums>, DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(get_query("albums_artist_id"))?;
        let result = stmt
            .query_map([artist_id], Albums::from_row)?
            .collect::<Result<Vec<Albums>, Error>>()?;

        Ok(result)
    }

    /// Get duration of all tracks of given album
    pub fn get_album_duration(&self, album_id: &u32) -> Result<(u32, u32), DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(get_query("albums_duration"))?;

        let result = stmt.query_row([album_id], |row| Ok((row.get(0)?, row.get(1)?)))?;

        Ok(result)
    }

    pub fn album_with_tracks(&self, album_id: &u32) -> Result<AlbumWithTracks, DatabaseError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM tracks WHERE album_id = ?1")?;

        let tracks = stmt
            .query_map([album_id], Tracks::from_row)?
            .collect::<Result<Vec<Tracks>, Error>>()?;

        let album = self.by_id::<Albums>(album_id)?;

        Ok(AlbumWithTracks { album, tracks })
    }

    pub fn update_album_type(
        &self,
        album_id: &u32,
        album_type: AlbumType,
        duration_count: &(u32, u32),
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        conn.execute(
            get_query("albums_update_type"),
            (album_type, duration_count.0, duration_count.1, album_id),
        )?;

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
        let mut stmt = conn.prepare(get_query("playlists_tracks"))?;

        let tracks = stmt
            .query_map([playlist_id], Tracks::from_row)?
            .collect::<Result<Vec<Tracks>, Error>>()?;

        Ok(PlaylistWithTracks { playlist, tracks })
    }

    pub fn update_playlist(&self, playlist: &Playlists) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        conn.execute(
            get_query("playlists_update"),
            playlist.to_params().as_slice(),
        )?;

        Ok(())
    }

    pub fn insert_track_to_playlist(
        &self,
        playlist_id: &u32,
        track_id: &u32,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        conn.execute(get_query("playlists_insert_track"), [playlist_id, track_id])?;

        Ok(())
    }

    pub fn delete_track_from_playlist(
        &self,
        playlist_id: &u32,
        track_id: &u32,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;

        conn.execute(get_query("playlists_delete_track"), [playlist_id, track_id])?;

        Ok(())
    }

    pub fn album_exists(
        &self,
        album_name: &str,
        album_year: u16,
    ) -> Result<Option<Albums>, DatabaseError> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(get_query("artist_album"))?;
        let result = stmt
            .query_row((album_name, album_year), Albums::from_row)
            .optional()?;

        Ok(result)
    }
}
