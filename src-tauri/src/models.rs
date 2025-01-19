use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Clone, Type)]
pub struct Artists {
    pub id: u32,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct Albums {
    pub id: u32,
    pub artists_id: u32,
    pub artist: String,
    pub name: String,
    pub cover_path: String,
    pub album_type: String,
    pub duration: u32,
    pub track_count: u32,
    pub year: u16,
    pub path: String,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct Tracks {
    pub id: u32,
    pub duration: u32,
    pub album: String,
    pub albums_id: u32,
    pub artist: String,
    pub artists_id: u32,
    pub name: String,
    pub path: String,
    pub cover_path: String,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct Playlists {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub cover_path: String,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct PlaylistWithTracks {
    pub playlist: Playlists,
    pub tracks: Vec<Tracks>,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct AlbumWithTracks {
    pub album: Albums,
    pub tracks: Vec<Tracks>,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct ArtistWithAlbums {
    pub artist: Artists,
    pub albums: Vec<AlbumWithTracks>,
}
