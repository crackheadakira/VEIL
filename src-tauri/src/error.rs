use crate::player;
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
    #[error("standard error: {0}")]
    Standard(String),
}

impl From<std::io::Error> for FrontendError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<metadata_audio::MetadataError> for FrontendError {
    fn from(error: metadata_audio::MetadataError) -> Self {
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

impl From<Box<dyn std::error::Error>> for FrontendError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Standard(error.to_string())
    }
}
