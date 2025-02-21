use rusqlite::{types::FromSql, Result, ToSql};

#[cfg(feature = "serialization")]
use serde::Serialize;
#[cfg(feature = "serialization")]
use specta::Type;

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
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

impl From<String> for AlbumType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Single" => AlbumType::Single,
            "EP" => AlbumType::EP,
            "Album" => AlbumType::Album,
            _ => AlbumType::Unknown,
        }
    }
}

impl FromSql for AlbumType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Text(text) => match text {
                b"Single" => Ok(AlbumType::Single),
                b"EP" => Ok(AlbumType::EP),
                b"Album" => Ok(AlbumType::Album),
                _ => Ok(AlbumType::Unknown),
            },
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl ToSql for AlbumType {
    fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>> {
        let text = match self {
            AlbumType::Unknown => "Unknown",
            AlbumType::Single => "Single",
            AlbumType::EP => "EP",
            AlbumType::Album => "Album",
        };

        Ok(text.into())
    }
}

pub trait NeedForDatabase: Sized {
    /// Turn rusqlite row into given struct
    fn from_row(row: &rusqlite::Row) -> Result<Self>;
    /// Name of struct in database
    fn table_name() -> &'static str;
    /// Struct to parameters to insert into database
    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql>;
    /// Return an Option<u32> of artist_id
    fn get_artist_id(&self) -> Option<u32>;
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
pub struct Artists {
    /// ID of artist in database
    pub id: u32,
    /// Name of artist
    pub name: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
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
    /// Path to album cover in Sodapop local app data
    pub cover_path: String,
    /// Path to album folder from where it was imported
    pub path: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
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
    /// Track duration
    pub duration: u32,
    /// Path to album cover in Sodapop local app data
    pub cover_path: String,
    /// Path to track file
    pub path: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
pub struct Playlists {
    /// ID of playlist in database
    pub id: u32,
    /// Playlist name
    pub name: String,
    /// Playlist description
    pub description: String,
    /// Path to playlist cover in Sodapop local app data
    pub cover_path: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
pub struct PlaylistWithTracks {
    pub playlist: Playlists,
    /// All tracks belonging to playlist
    pub tracks: Vec<Tracks>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
pub struct AlbumWithTracks {
    pub album: Albums,
    /// All tracks belonging to album
    pub tracks: Vec<Tracks>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
pub struct ArtistWithAlbums {
    pub artist: Artists,
    /// All albums belonging to artist
    pub albums: Vec<AlbumWithTracks>,
}

impl NeedForDatabase for Artists {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Artists {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    }

    fn table_name() -> &'static str {
        "artists"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.name]
    }

    fn get_artist_id(&self) -> Option<u32> {
        Some(self.id)
    }
}

impl NeedForDatabase for Albums {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Albums {
            artist_id: row.get(0)?,
            artist_name: row.get(1)?,
            id: row.get(2)?,
            name: row.get(3)?,
            year: row.get(4)?,
            album_type: row.get(5)?,
            track_count: row.get(6)?,
            duration: row.get(7)?,
            cover_path: row.get(8)?,
            path: row.get(9)?,
        })
    }

    fn table_name() -> &'static str {
        "albums"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.artist_id,
            &self.name,
            &self.year,
            &self.album_type,
            &self.track_count,
            &self.duration,
            &self.cover_path,
            &self.path,
        ]
    }

    fn get_artist_id(&self) -> Option<u32> {
        Some(self.artist_id)
    }
}

impl NeedForDatabase for Tracks {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Tracks {
            id: row.get(0)?,
            album_id: row.get(1)?,
            artist_id: row.get(2)?,
            album_name: row.get(3)?,
            artist_name: row.get(4)?,
            name: row.get(5)?,
            duration: row.get(6)?,
            cover_path: row.get(7)?,
            path: row.get(8)?,
        })
    }

    fn table_name() -> &'static str {
        "tracks"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.album_id,
            &self.artist_id,
            &self.album_name,
            &self.artist_name,
            &self.name,
            &self.duration,
            &self.cover_path,
            &self.path,
        ]
    }

    fn get_artist_id(&self) -> Option<u32> {
        Some(self.artist_id)
    }
}

impl NeedForDatabase for Playlists {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Playlists {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            cover_path: row.get(3)?,
        })
    }

    fn table_name() -> &'static str {
        "playlists"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.name, &self.description, &self.cover_path]
    }

    fn get_artist_id(&self) -> Option<u32> {
        None
    }
}
