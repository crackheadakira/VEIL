use std::fmt;

use serde::{Deserialize, Serialize};

pub enum APIMethod {
    ArtistGetInfo,
    AuthGetSession,
    AuthGetToken,
    UserGetInfo,
}

impl APIMethod {
    /// If the method needs an API signature
    pub fn need_sig(&self) -> bool {
        match self {
            Self::AuthGetSession | Self::AuthGetToken => true,
            _ => false,
        }
    }

    /// Get the method as a method string to pass to last.fm
    pub fn as_query(&self) -> String {
        let result = match self {
            Self::ArtistGetInfo => "artist.getInfo",
            Self::AuthGetSession => "auth.getSession",
            Self::AuthGetToken => "auth.getToken",
            Self::UserGetInfo => "user.GetInfo",
        };

        String::from(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The size of the image (i.e "small", "medium", "large")
    size: String,
    /// The URL of the image
    #[serde(alias = "#text")]
    text: String,
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
