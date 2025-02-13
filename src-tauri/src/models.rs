use rusqlite::Result;
use serde::Serialize;
use specta::Type;

pub trait NeedForDatabase: Sized {
    fn from_row(row: &rusqlite::Row) -> Result<Self>;
    fn table_name() -> &'static str;
    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql>;
}

#[derive(Debug, Serialize, Type)]
pub struct Artists {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Type)]
pub struct Albums {
    pub id: u32,
    pub artist_id: u32,
    pub artist_name: String,
    pub name: String,
    pub year: u16,
    pub album_type: String,
    pub track_count: u32,
    pub duration: u32,
    pub cover_path: String,
    pub path: String,
}

#[derive(Debug, Serialize, Type)]
pub struct Tracks {
    pub id: u32,
    pub album_id: u32,
    pub artist_id: u32,
    pub album_name: String,
    pub artist_name: String,
    pub name: String,
    pub duration: u32,
    pub cover_path: String,
    pub path: String,
}

#[derive(Debug, Serialize, Type)]
pub struct Playlists {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub cover_path: String,
}

#[derive(Debug, Serialize, Type)]
pub struct PlaylistWithTracks {
    pub playlist: Playlists,
    pub tracks: Vec<Tracks>,
}

#[derive(Debug, Serialize, Type)]
pub struct AlbumWithTracks {
    pub album: Albums,
    pub tracks: Vec<Tracks>,
}

#[derive(Debug, Serialize, Type)]
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
            artist_id: row.get(1)?,
            artist_name: row.get(2)?,
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
            &self.artist_name,
            &self.name,
            &self.year,
            &self.album_type,
            &self.track_count,
            &self.duration,
            &self.cover_path,
            &self.path,
        ]
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
