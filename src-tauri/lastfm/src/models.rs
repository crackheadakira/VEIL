use std::fmt;

use serde::{Deserialize, Serialize};

pub enum APIMethod {
    ArtistGetInfo,
    AuthGetSession,
    AuthGetToken,
    UserGetInfo,
}

impl APIMethod {
    pub fn need_sig(&self) -> bool {
        match self {
            Self::AuthGetSession | Self::AuthGetToken => true,
            _ => false,
        }
    }

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
    size: String,
    #[serde(alias = "#text")]
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIError {
    pub error: i64,
    pub message: String,
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Error {}: {}", self.error, self.message)
    }
}

impl std::error::Error for APIError {}
