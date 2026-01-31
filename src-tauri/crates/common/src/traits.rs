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
pub use crate::rusqlite_impl::*;
