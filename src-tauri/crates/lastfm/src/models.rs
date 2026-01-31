use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum APIMethod {
    AlbumGetInfo,
    AuthGetSession,
    AuthGetToken,
    UserGetInfo,
    TrackUpdateNowPlaying,
    TrackScrobble,
}

impl APIMethod {
    /// If the method needs an API signature
    pub fn need_sig(&self) -> bool {
        matches!(
            self,
            Self::AuthGetSession
                | Self::AuthGetToken
                | Self::TrackScrobble
                | Self::TrackUpdateNowPlaying
        )
    }

    /// If the method needs authentication
    pub fn need_auth(&self) -> bool {
        matches![self, |Self::TrackScrobble| Self::TrackUpdateNowPlaying]
    }

    /// Get the method as a method string to pass to last.fm
    pub fn as_query(&self) -> &'static str {
        match self {
            Self::AlbumGetInfo => "album.getInfo",
            Self::AuthGetSession => "auth.getSession",
            Self::AuthGetToken => "auth.getToken",
            Self::UserGetInfo => "user.getInfo",
            Self::TrackUpdateNowPlaying => "track.updateNowPlaying",
            Self::TrackScrobble => "track.scrobble",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The size of the image (i.e "small", "medium", "large")
    pub size: String,
    /// The URL of the image
    #[serde(alias = "#text")]
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIError {
    /// The error code
    pub error: i64,
    /// The error message
    pub message: String,
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Error {}: {}", self.error, self.message)
    }
}

impl std::error::Error for APIError {}

#[derive(Debug)]
pub struct TrackData {
    pub artist: String,
    pub name: String,
    pub album: Option<String>,
    pub timestamp: Option<i64>,
}
