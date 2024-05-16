use dirs;
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

    let cover_path = config_path() + "/covers";
    if !std::path::Path::new(&cover_path).exists() {
        std::fs::create_dir(cover_path).expect("Error creating covers directory");
    }

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
            name        TEXT    NOT NULL,
            cover_path  TEXT    NOT NULL,
            year        INTEGER NOT NULL,
            path        TEXT    NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS tracks (
            id          INTEGER NOT NULL PRIMARY KEY,
            album       TEXT    NOT NULL,
            albums_id   INTEGER NOT NULL REFERENCES albums(id),
            artist      TEXT    NOT NULL,
            name        TEXT    NOT NULL,
            path        TEXT    NOT NULL UNIQUE
        ); 
        CREATE TABLE IF NOT EXISTS playlists (
            id          INTEGER NOT NULL PRIMARY KEY,
            name        TEXT    NOT NULL,
            description TEXT    NOT NULL,
            cover_path  TEXT    NOT NULL
        );
        COMMIT;
    ",
    )
    .expect("Error creating tables");
}

pub fn db_connect() -> r2d2::PooledConnection<SqliteConnectionManager> {
    POOL.get().expect("Error getting connection from pool")
}

// TODO: Cross-platform support, rather than just writing ~/.config/ check if Tauri
// has a better way to do this
fn db_path() -> String {
    let home_dir = dirs::home_dir().unwrap();
    return home_dir.to_str().unwrap().to_string() + "/.config/sodapop/db.sqlite";
}

pub fn config_path() -> String {
    let home_dir = dirs::home_dir().unwrap();
    home_dir.to_str().unwrap().to_string() + "/.config/sodapop"
}
