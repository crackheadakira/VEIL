INSERT INTO albums (artist_id, artist_name, name, year, type, track_count, duration, cover_path, path)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) RETURNING id