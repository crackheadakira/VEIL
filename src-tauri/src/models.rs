use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Artists {
    pub id: i32,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Albums {
    pub id: i32,
    pub artists_id: i32,
    pub name: String,
    pub cover_path: String,
    pub year: i32,
    pub path: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Tracks {
    pub id: i32,
    pub album: String,
    pub albums_id: i32,
    pub artist: String,
    pub name: String,
    pub path: String,
}

pub struct Playlists {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub cover_path: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AlbumWithTracks {
    pub album: Albums,
    pub tracks: Vec<Tracks>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ArtistWithAlbums {
    pub artist: Artists,
    pub albums: Vec<AlbumWithTracks>,
}
