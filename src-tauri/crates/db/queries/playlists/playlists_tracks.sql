SELECT
    t.id,
    t.album_id,
    t.artist_id,
    t.album_name,
    t.artist_name,
    t.name,
    t.number,
    t.duration,
    t.cover_path,
    t.path

FROM
    playlist_tracks pt
    JOIN tracks t ON pt.track_id = t.id
WHERE
    pt.playlist_id = ?1

ORDER BY t.id DESC