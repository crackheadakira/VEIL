SELECT COUNT(*)
FROM
    playlist_tracks pt
    JOIN tracks t ON pt.track_id = t.id
WHERE
    pt.playlist_id = ?1