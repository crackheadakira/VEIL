use rusqlite::Result;
use serde::Serialize;
use specta::Type;

pub trait NeedForDatabase: Sized {
    fn from_row(row: &rusqlite::Row) -> Result<Self>;
    fn table_name() -> &'static str;
    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql>;
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct Artists {
    pub id: u32,
    pub name: String,
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
pub struct Features {
    pub id: u32,
    pub track_id: u32,
    pub feature_id: u32,
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
pub struct TrackWithFeatures {
    pub track: Tracks,
    pub features: Vec<Artists>,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct AlbumWithTracks {
    pub album: Albums,
    pub tracks: Vec<TrackWithFeatures>,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct ArtistWithAlbums {
    pub artist: Artists,
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
}

impl NeedForDatabase for Albums {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Albums {
            id: row.get(0)?,
            artists_id: row.get(1)?,
            artist: row.get(2)?,
            name: row.get(3)?,
            cover_path: row.get(4)?,
            album_type: row.get(5)?,
            duration: row.get(6)?,
            track_count: row.get(7)?,
            year: row.get(8)?,
            path: row.get(9)?,
        })
    }

    fn table_name() -> &'static str {
        "albums"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.artists_id,
            &self.artist,
            &self.name,
            &self.cover_path,
            &self.album_type,
            &self.duration,
            &self.track_count,
            &self.year,
            &self.path,
        ]
    }
}

impl NeedForDatabase for Tracks {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Tracks {
            id: row.get(0)?,
            album: row.get(1)?,
            albums_id: row.get(2)?,
            artist: row.get(3)?,
            artists_id: row.get(4)?,
            name: row.get(5)?,
            duration: row.get(6)?,
            path: row.get(7)?,
            cover_path: row.get(8)?,
        })
    }

    fn table_name() -> &'static str {
        "tracks"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.album,
            &self.albums_id,
            &self.artist,
            &self.artists_id,
            &self.name,
            &self.duration,
            &self.path,
            &self.cover_path,
        ]
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
}

impl NeedForDatabase for Features {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Features {
            id: row.get(0)?,
            track_id: row.get(1)?,
            feature_id: row.get(2)?,
        })
    }

    fn table_name() -> &'static str {
        "features"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.track_id, &self.feature_id]
    }
}
