use rusqlite::{Result, ToSql, types::FromSql};

use crate::{
    AlbumType, Albums, Artists, NewAlbum, NewArtist, NewPlaylist, NewTrack, Playlists, Search,
    Tracks,
};

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

pub trait HasArtists {
    /// Return an Option<u32> of `artist_id`
    fn get_artist_id(&self) -> Option<u32>;
}

impl<'a> HasArtists for NewAlbum<'a> {
    fn get_artist_id(&self) -> Option<u32> {
        Some(self.artist_id)
    }
}

pub trait Hashable {
    /// Converts the struct into a hash
    fn make_hash(&self) -> String;
}

impl<'a> Hashable for NewArtist<'a> {
    fn make_hash(&self) -> String {
        blake3::hash(self.name.to_owned().as_bytes())
            .to_hex()
            .to_string()
    }
}

impl<'a> Hashable for NewAlbum<'a> {
    fn make_hash(&self) -> String {
        let formatted_string = format!(
            "{}:{}:{}:{:?}",
            self.name, self.duration, self.track_count, self.album_type
        );
        blake3::hash(formatted_string.as_bytes())
            .to_hex()
            .to_string()
    }
}

impl<'a> Hashable for NewTrack<'a> {
    fn make_hash(&self) -> String {
        let formatted_string = format!(
            "{}:{}:{}:{}",
            self.album_name, self.name, self.artist_name, self.duration
        );
        blake3::hash(formatted_string.as_bytes())
            .to_hex()
            .to_string()
    }
}

impl<'a> Hashable for NewPlaylist<'a> {
    fn make_hash(&self) -> String {
        let formatted_string = format!("{}:{}", self.name, self.description);
        blake3::hash(formatted_string.as_bytes())
            .to_hex()
            .to_string()
    }
}

pub trait Insertable {
    /// Name of struct in database
    fn table_name() -> &'static str;

    /// Struct to parameters to insert into database
    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql>;
}

impl<'a> Insertable for NewArtist<'a> {
    fn table_name() -> &'static str {
        "artists"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.name]
    }
}

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

impl<'a> Insertable for NewPlaylist<'a> {
    fn table_name() -> &'static str {
        "playlists"
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.name, &self.description, &self.cover_path]
    }
}

pub trait Queryable: Sized {
    /// Turn rusqlite row into given struct
    fn from_row(row: &rusqlite::Row) -> Result<Self>;

    /// Name of struct in database
    fn table_name() -> &'static str;
}

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
            hash: row.get(10)?,
        })
    }

    fn table_name() -> &'static str {
        "tracks"
    }
}

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
