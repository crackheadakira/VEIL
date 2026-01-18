CREATE TABLE IF NOT EXISTS artists (
    id          INTEGER NOT NULL PRIMARY KEY,
    name        TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS albums (
    id          INTEGER NOT NULL PRIMARY KEY,
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
    number      INTEGER NOT NULL,
    duration    INTEGER NOT NULL,
    cover_path  TEXT    NOT NULL,
    path        TEXT    NOT NULL UNIQUE,
    hash        TEXT    NOT NULL UNIQUE
); 

CREATE TABLE IF NOT EXISTS playlists (
    id          INTEGER NOT NULL PRIMARY KEY,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL,
    cover_path  TEXT    NOT NULL,
    track_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS playlist_tracks (
    id          INTEGER NOT NULL PRIMARY KEY,
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS album_artists (
    album_id    INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    artist_id   INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    PRIMARY KEY (album_id, artist_id)
);

-- Add support for searching

CREATE VIRTUAL TABLE IF NOT EXISTS search
USING FTS5(title, type, type_id);

-- Insert Triggers

CREATE TRIGGER IF NOT EXISTS albums_insert_search
AFTER INSERT ON albums
BEGIN
    INSERT INTO search (title, type, type_id)
    VALUES (NEW.name, 'album', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS artists_insert_search
AFTER INSERT ON artists
BEGIN
    INSERT INTO search (title, type, type_id)
    VALUES (NEW.name, 'artist', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS playlists_insert_search
AFTER INSERT ON playlists
BEGIN
    INSERT INTO search (title, type, type_id)
    VALUES (NEW.name, 'playlist', NEW.id);
END;

-- Delete Triggers

CREATE TRIGGER IF NOT EXISTS albums_delete_search
AFTER DELETE ON albums
BEGIN
    DELETE FROM search WHERE type = 'album' AND type_id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS artists_delete_search
AFTER DELETE ON artists
BEGIN
    DELETE FROM search WHERE type = 'artist' AND type_id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS playlists_delete_search
AFTER DELETE ON playlists
BEGIN
    DELETE FROM search WHERE type = 'playlist' AND type_id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS playlist_track_insert
AFTER INSERT ON playlist_tracks
FOR EACH ROW
BEGIN
    UPDATE playlists
    SET track_count = track_count + 1
    WHERE id = NEW.playlist_id;
END;

CREATE TRIGGER IF NOT EXISTS playlist_track_delete
AFTER DELETE ON playlist_tracks
FOR EACH ROW
BEGIN
    UPDATE playlists
    SET track_count = track_count - 1
    WHERE id = OLD.playlist_id;
END;