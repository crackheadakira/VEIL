UPDATE playlists
SET
    name = ?1,
    description = ?2,
    cover_path = ?3
WHERE
    id = ?4