use crate::{db, player};
use specta::Type;

#[derive(thiserror::Error, Debug, serde::Serialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum FrontendError {
    #[error("io error: {0}")]
    Io(String),
    #[error("metadata error: {0}")]
    Metadata(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("player error: {0}")]
    Player(String),
    #[error("souvlaki error: {0}")]
    Souvlaki(String),
}

impl From<std::io::Error> for FrontendError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<audio_metadata::MetadataError> for FrontendError {
    fn from(error: audio_metadata::MetadataError) -> Self {
        Self::Metadata(error.to_string())
    }
}

impl From<db::DatabaseError> for FrontendError {
    fn from(error: db::DatabaseError) -> Self {
        Self::Database(error.to_string())
    }
}

impl From<player::PlayerError> for FrontendError {
    fn from(error: player::PlayerError) -> Self {
        Self::Player(error.to_string())
    }
}

impl From<souvlaki::Error> for FrontendError {
    fn from(error: souvlaki::Error) -> Self {
        Self::Souvlaki(error.to_string())
    }
}
