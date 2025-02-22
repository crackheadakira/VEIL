SELECT
    ar.id AS artist_id,
    ar.name AS artist_name,
    a.id AS album_id,
    a.name AS album_name,
    a.year,
    a.type,
    a.track_count,
    a.duration,
    a.cover_path,
    a.path
FROM albums a
JOIN album_artists aa ON a.id = aa.album_id
JOIN artists ar ON aa.artist_id = ar.id
WHERE a.id = (
    SELECT id FROM albums
    ORDER BY rowid DESC
    LIMIT 1
);