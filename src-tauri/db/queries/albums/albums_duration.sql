SELECT 
    SUM(duration),
    COUNT(*)
FROM tracks
WHERE album_id = ?1