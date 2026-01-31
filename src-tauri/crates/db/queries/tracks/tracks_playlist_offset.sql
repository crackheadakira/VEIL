SELECT
    t.*,
    ar.id AS artist_id,
    ar.name AS artist_name,
    a.id AS album_id,
    a.name AS album_name
FROM playlist_tracks pt
JOIN tracks t ON pt.track_id = t.id
JOIN albums a ON t.album_id = a.id
JOIN artists ar ON t.artist_id = ar.id
WHERE pt.playlist_id = ?1
ORDER BY t.id ASC
LIMIT ?2
OFFSET ?3;