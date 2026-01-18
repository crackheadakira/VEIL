SELECT
    t.*,
    ar.id AS artist_id,
    ar.name AS artist_name,
    a.id AS album_id,
    a.name AS album_name
FROM tracks t
JOIN albums a ON t.album_id = a.id
JOIN artists ar ON t.artist_id = ar.id
ORDER BY t.id ASC
LIMIT ?1
OFFSET ?2;