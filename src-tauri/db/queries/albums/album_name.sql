SELECT
    ar.id AS artist_id,
    ar.name AS artist_name,
    a.*
FROM albums a
JOIN album_artists aa ON a.id = aa.album_id
JOIN artists ar ON aa.artist_id = ar.id
WHERE ar.id = ?1 AND a.name = ?2;