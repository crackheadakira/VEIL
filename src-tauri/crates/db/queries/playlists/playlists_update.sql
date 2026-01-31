UPDATE playlists
SET
    name = COALESCE(?1, name),
    description = COALESCE(?2, description),
    cover_path = COALESCE(?3, cover_path)
WHERE id = ?4;