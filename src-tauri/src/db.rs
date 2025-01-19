use lazy_static::*;
use r2d2_sqlite::SqliteConnectionManager;

lazy_static! {
    static ref POOL: r2d2::Pool<SqliteConnectionManager> = {
        let manager = SqliteConnectionManager::file(db_path());
        r2d2::Pool::new(manager).expect("Error creating connection pool")
    };
}

pub fn init() {
    let conn = db_connect();
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
}

pub fn db_connect() -> r2d2::PooledConnection<SqliteConnectionManager> {
    POOL.get().expect("Error getting connection")
}

pub fn db_path() -> String {
    data_path() + "/db.sqlite"
}

pub fn data_path() -> String {
    let home_dir = dirs::data_local_dir().unwrap();
    home_dir.to_str().unwrap().to_string() + "/sodapop-reimagined"
}
