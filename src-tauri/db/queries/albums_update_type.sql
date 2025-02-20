UPDATE albums
SET
    type = ?1,
    duration = ?2,
    track_count = ?3
WHERE ID = ?4