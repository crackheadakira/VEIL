use crate::*;

impl From<&str> for AlbumType {
    fn from(value: &str) -> Self {
        match value {
            "Single" => AlbumType::Single,
            "EP" => AlbumType::EP,
            "Album" => AlbumType::Album,
            _ => AlbumType::Unknown,
        }
    }
}

impl From<String> for AlbumType {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[cfg(feature = "rusqlite")]
use rusqlite::{Result, ToSql, types::FromSql};

#[cfg(feature = "rusqlite")]
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

#[cfg(feature = "rusqlite")]
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

#[cfg(feature = "rusqlite")]
pub trait Queryable: Sized {
    /// Turn rusqlite row into given struct
    fn from_row(row: &rusqlite::Row) -> Result<Self>;

    /// Name of struct in database
    fn table_name() -> &'static str;
}

pub trait HasArtists {
    /// Return an Option<u32> of `artist_id`
    fn get_artist_id(&self) -> Option<u32>;
}

#[cfg(feature = "rusqlite")]
pub trait Insertable {
    /// Name of struct in database
    fn table_name() -> &'static str;

    /// Struct to parameters to insert into database
    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql>;
}

#[cfg(feature = "rusqlite")]
impl<'a> Insertable for NewArtist<'a> {
    fn table_name() -> &'static str {
        "artists"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.name]
    }
}

#[cfg(feature = "rusqlite")]
impl<'a> Insertable for NewAlbum<'a> {
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
}

#[cfg(feature = "rusqlite")]
impl<'a> Insertable for NewTrack<'a> {
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
            &self.number,
            &self.duration,
            &self.cover_path,
            &self.path,
        ]
    }
}

#[cfg(feature = "rusqlite")]
impl<'a> Insertable for NewPlaylist<'a> {
    fn table_name() -> &'static str {
        "playlists"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.name, &self.description, &self.cover_path]
    }
}

#[cfg(feature = "rusqlite")]
impl Queryable for Artists {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    }

    fn table_name() -> &'static str {
        "artists"
    }
}

#[cfg(feature = "rusqlite")]
impl Queryable for Albums {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Self {
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
}

#[cfg(feature = "rusqlite")]
impl<'a> HasArtists for NewAlbum<'a> {
    fn get_artist_id(&self) -> Option<u32> {
        Some(self.artist_id)
    }
}

#[cfg(feature = "rusqlite")]
impl Queryable for Tracks {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            album_id: row.get(1)?,
            artist_id: row.get(2)?,
            album_name: row.get(3)?,
            artist_name: row.get(4)?,
            name: row.get(5)?,
            number: row.get(6)?,
            duration: row.get(7)?,
            cover_path: row.get(8)?,
            path: row.get(9)?,
        })
    }

    fn table_name() -> &'static str {
        "tracks"
    }
}

#[cfg(feature = "rusqlite")]
impl Queryable for Playlists {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            cover_path: row.get(3)?,
        })
    }

    fn table_name() -> &'static str {
        "playlists"
    }
}

#[cfg(feature = "rusqlite")]
impl Queryable for Search {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Self {
            title: row.get(0)?,
            search_type: row.get(1)?,
            search_id: row.get(2)?,
        })
    }

    fn table_name() -> &'static str {
        "search"
    }
}
