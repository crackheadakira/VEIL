CREATE TABLE IF NOT EXISTS artists (
    id          INTEGER NOT NULL PRIMARY KEY,
    name        TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS albums (
    id          INTEGER NOT NULL PRIMARY KEY,
    artist_id   INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    artist_name TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    year        INTEGER NOT NULL,
    type        TEXT    NOT NULL,
    track_count INTEGER NOT NULL,
    duration    INTEGER NOT NULL,
    cover_path  TEXT    NOT NULL,
    path        TEXT    NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS tracks (
    id          INTEGER NOT NULL PRIMARY KEY,
    album_id    INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    artist_id   INTEGER NOT NULL REFERENCES artists(id),
    album_name  TEXT    NOT NULL,
    artist_name TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    duration    INTEGER NOT NULL,
    cover_path  TEXT    NOT NULL,
    path        TEXT    NOT NULL UNIQUE
); 

CREATE TABLE IF NOT EXISTS playlists (
    id          INTEGER NOT NULL PRIMARY KEY,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL,
    cover_path  TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_tracks (
    id          INTEGER NOT NULL PRIMARY KEY,
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE
);

/*CREATE TABLE IF NOT EXISTS album_artists (
    album_id    INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    artist_id   INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    PRIMARY KEY (album_id, artist_id)
);*/