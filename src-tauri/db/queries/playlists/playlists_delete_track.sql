DELETE FROM playlist_tracks
WHERE ROWID IN (
    SELECT MIN(ROWID) as row_id
    FROM playlist_tracks
    WHERE playlist_id = ?1 AND track_id = ?2
)