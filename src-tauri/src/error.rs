use crate::{db, player};
use specta::Type;

#[derive(thiserror::Error, Debug, serde::Serialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum FrontendError {
    #[error("io error: {0}")]
    IoError(String),
    #[error("metadata error: {0}")]
    MetadataError(String),
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("player error: {0}")]
    PlayerError(String),
    #[error("souvlaki error: {0}")]
    SouvlakiError(String),
}

impl From<std::io::Error> for FrontendError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error.to_string())
    }
}

impl From<audio_metadata::MetadataError> for FrontendError {
    fn from(error: audio_metadata::MetadataError) -> Self {
        Self::MetadataError(error.to_string())
    }
}

impl From<db::DatabaseError> for FrontendError {
    fn from(error: db::DatabaseError) -> Self {
        Self::DatabaseError(error.to_string())
    }
}

impl From<player::PlayerError> for FrontendError {
    fn from(error: player::PlayerError) -> Self {
        Self::PlayerError(error.to_string())
    }
}

impl From<souvlaki::Error> for FrontendError {
    fn from(error: souvlaki::Error) -> Self {
        Self::SouvlakiError(error.to_string())
    }
}
