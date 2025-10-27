use specta::Type;
use tauri_specta::Event;

#[derive(thiserror::Error, Debug, serde::Serialize, Type, Event, Clone)]
#[serde(tag = "type", content = "data")]
pub enum FrontendError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Metadata error: {0}")]
    Metadata(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Player error: {0}")]
    Player(String),
    #[error("Standard error: {0}")]
    Standard(String),
    #[error("LastFM error: {0}")]
    LastFMError(String),
    #[error("Serde JSON: {0}")]
    SerdeJson(String),
    #[error("Tauri error: {0}")]
    TauriError(String),
    #[error("Anyhow error: {0}")]
    AnyhowError(String),
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

impl From<media_controls::PlayerError> for FrontendError {
    fn from(error: media_controls::PlayerError) -> Self {
        Self::Player(error.to_string())
    }
}

impl From<lastfm::LastFMError> for FrontendError {
    fn from(error: lastfm::LastFMError) -> Self {
        Self::LastFMError(error.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for FrontendError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Standard(error.to_string())
    }
}

impl From<serde_json::Error> for FrontendError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerdeJson(error.to_string())
    }
}

impl From<tauri::Error> for FrontendError {
    fn from(error: tauri::Error) -> Self {
        Self::TauriError(error.to_string())
    }
}

impl From<anyhow::Error> for FrontendError {
    fn from(error: anyhow::Error) -> Self {
        Self::AnyhowError(error.to_string())
    }
}
