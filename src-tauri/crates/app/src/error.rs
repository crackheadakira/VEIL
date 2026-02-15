#[derive(thiserror::Error, Debug, serde::Serialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum VeilError {
    #[error("[IO] {0}")]
    Io(String),

    #[error("[Metadata] {0}")]
    Metadata(String),

    #[error("[Database] {0}")]
    Database(String),

    #[error("[Player] {0}")]
    Player(String),

    #[error("[Standard] {0}")]
    Standard(String),

    #[error("[LastFM] {0}")]
    LastFMError(String),

    #[error("[JSON] {0}")]
    SerdeJson(String),

    #[error("[Tauri] {0}")]
    TauriError(String),

    #[error("[Anyhow] {0}")]
    AnyhowError(String),
}

impl From<std::io::Error> for VeilError {
    fn from(error: std::io::Error) -> Self {
        let msg = error.to_string();
        logging::error!("{error:?}");

        Self::Io(msg)
    }
}

impl From<metadata_audio::Error> for VeilError {
    fn from(error: metadata_audio::Error) -> Self {
        let msg = error.to_string();
        logging::error!("{error:?}");

        Self::Metadata(msg)
    }
}

impl From<db::Error> for VeilError {
    fn from(error: db::Error) -> Self {
        let msg = error.to_string();
        logging::error!("{error:?}");

        Self::Database(msg)
    }
}

impl From<media_controls::Error> for VeilError {
    fn from(error: media_controls::Error) -> Self {
        let msg = error.to_string();
        logging::error!("{error:?}");

        Self::Player(msg)
    }
}

impl From<lastfm::Error> for VeilError {
    fn from(error: lastfm::Error) -> Self {
        let msg = error.to_string();
        logging::error!("{error:?}");

        Self::LastFMError(msg)
    }
}

impl From<Box<dyn std::error::Error>> for VeilError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        let msg = error.to_string();
        logging::error!("{error:?}");

        Self::Standard(msg)
    }
}

impl From<serde_json::Error> for VeilError {
    fn from(error: serde_json::Error) -> Self {
        let msg = error.to_string();
        logging::error!("{error:?}");

        Self::SerdeJson(msg)
    }
}

impl From<anyhow::Error> for VeilError {
    fn from(error: anyhow::Error) -> Self {
        let mut msg = error.to_string();

        for cause in error.chain().skip(1) {
            msg.push_str(&format!("\nCaused by: {}", cause));
        }

        logging::error!("{error:?}");

        VeilError::AnyhowError(msg)
    }
}
