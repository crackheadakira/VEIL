#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rusqlite")]
mod rusqlite_impl;

pub mod traits;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum AlbumType {
    Unknown,
    Single,
    EP,
    Album,
}

impl AlbumType {
    pub fn get(tracks: u32, duration: u32) -> Self {
        if duration == 0 || tracks == 0 {
            Self::Unknown
        } else if tracks < 3 && duration < 1800 {
            Self::Single
        } else if tracks <= 6 && duration < 1800 {
            Self::EP
        } else {
            Self::Album
        }
    }
}

impl std::fmt::Display for AlbumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            AlbumType::Unknown => "Unknown",
            AlbumType::Single => "Single",
            AlbumType::EP => "EP",
            AlbumType::Album => "Album",
        };
        write!(f, "{}", label)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Artists {
    /// ID of artist in database
    pub id: u32,
    /// Name of artist
    pub name: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Albums {
    /// ID of album in database
    pub id: u32,

    /// ID of artist in database
    pub artist_id: u32,

    /// Name of artist
    pub artist_name: String,

    /// Name of album
    pub name: String,

    /// Year album was published
    pub year: u16,

    /// Album type
    pub album_type: AlbumType,

    /// Amount of tracks in album
    pub track_count: u32,

    /// Album duration
    pub duration: u32,

    /// Path to album cover in VEIL local app data
    pub cover_path: String,

    /// Path to album folder from where it was imported
    pub path: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Tracks {
    /// ID of track in database
    pub id: u32,

    /// ID of album in database
    pub album_id: u32,

    /// ID of artist in database
    pub artist_id: u32,

    /// Album name
    pub album_name: String,

    /// Artist name
    pub artist_name: String,

    /// Track name
    pub name: String,

    /// Track number in album
    pub number: i32,

    /// Track duration
    pub duration: u32,

    /// Path to album cover in VEIL local app data
    pub cover_path: String,

    /// Path to track file
    pub path: String,

    /// Hash of the metadata
    pub hash: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Playlists {
    /// ID of playlist in database
    pub id: u32,

    /// Playlist name
    pub name: String,

    /// Playlist description
    pub description: String,

    /// Path to playlist cover in VEIL local app data
    pub cover_path: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Search {
    /// ID of the search item
    pub search_id: u32,

    /// Name of the search item
    pub title: String,

    /// Type of the search item
    pub search_type: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct PlaylistWithTracks {
    pub playlist: Playlists,

    /// All tracks belonging to playlist
    pub tracks: Vec<Tracks>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct AlbumWithTracks {
    pub album: Albums,

    /// All tracks belonging to album
    pub tracks: Vec<Tracks>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct ArtistWithAlbums {
    pub artist: Artists,

    /// All albums belonging to artist
    pub albums: Vec<AlbumWithTracks>,
}

pub struct NewArtist<'a> {
    pub name: &'a str,
}

pub struct NewAlbum<'a> {
    /// ID of artist in database
    pub artist_id: u32,

    /// Name of artist
    pub artist_name: &'a str,

    /// Name of album
    pub name: &'a str,

    /// Year album was published
    pub year: u16,

    /// Album type
    pub album_type: &'a AlbumType,

    /// Amount of tracks in album
    pub track_count: u32,

    /// Album duration
    pub duration: u32,

    /// Path to album cover in VEIL local app data
    pub cover_path: &'a str,

    /// Path to album folder from where it was imported
    pub path: &'a str,
}

pub struct NewTrack<'a> {
    /// ID of album in database
    pub album_id: u32,

    /// ID of artist in database
    pub artist_id: u32,

    /// Album name
    pub album_name: &'a str,

    /// Artist name
    pub artist_name: &'a str,

    /// Track name
    pub name: &'a str,

    /// Track number in album
    pub number: i32,

    /// Track duration
    pub duration: u32,

    /// Path to album cover in VEIL local app data
    pub cover_path: &'a str,

    /// Path to track file
    pub path: &'a str,
}

pub struct NewPlaylist<'a> {
    /// Playlist name
    pub name: &'a str,

    /// Playlist description
    pub description: &'a str,

    /// Path to playlist cover in VEIL local app data
    pub cover_path: &'a str,
}
